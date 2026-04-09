import type { ComputedRef, Ref } from 'vue'
import type { useReaderStore } from '../stores/reader'

type ReaderStore = ReturnType<typeof useReaderStore>
const OPENAI_SPEECH_CHUNK_CHAR_LIMIT = 70
const OPENAI_PRELOAD_CHUNK_LIMIT = 5

interface AutoPlaybackConfig {
  autoPageMode: string
  clickAction: string
  scrollPixel: number
  pageSpeed: number
  fontSize: number
  lineHeight: number
}

export function useReaderAutoPlayback(
  store: ReaderStore,
  config: ComputedRef<AutoPlaybackConfig>,
  isContinuousMode: ComputedRef<boolean>,
  scrollContainerRef: Ref<HTMLElement | undefined>,
  chapterTextRef: Ref<HTMLElement | undefined>,
  nextChapter: () => void | Promise<void>,
  prevChapter: () => void | Promise<void>,
) {
  let autoScrollId: number | null = null
  let autoParagraphTimer: number | null = null
  let autoReadingParagraphIndex = -1
  let autoReadingProcessing = false
  let speechRestartTimer: number | null = null
  let isSpeechTransitioning = false
  let currentSpeechParagraph: HTMLElement | null = null
  let currentSpeechChunks: string[] = []
  let currentSpeechChunkIndex = 0

  function isSafariSpeechDelayBrowser() {
    if (typeof navigator === 'undefined') return false
    const ua = navigator.userAgent || ''
    return /Safari/i.test(ua) && !/Chrome|Chromium|CriOS|Edg|EdgiOS|Android/i.test(ua)
  }

  function paragraphPreview(paragraph: HTMLElement | null) {
    return paragraph?.innerText.trim().slice(0, 40) || ''
  }

  function logSpeech(message: string, payload?: unknown) {
    void message
    void payload
  }

  function getFilteredParagraphs() {
    const roots = isContinuousMode.value
      ? Array.from(scrollContainerRef.value?.querySelectorAll('.chapter-text[data-role="continuous"]') || []) as HTMLElement[]
      : (chapterTextRef.value ? [chapterTextRef.value] : [])
    if (!roots.length) return [] as HTMLElement[]
    const allElements = roots.flatMap((root) => Array.from(root.querySelectorAll('p')) as HTMLElement[])
    const list: HTMLElement[] = []
    let lastText = ''
    allElements.forEach((el) => {
      const text = el.innerText.trim()
      if (text && text !== lastText) {
        list.push(el)
        lastText = text
      }
    })
    return list
  }

  function getCurrentParagraph() {
    const reading = chapterTextRef.value?.querySelector('.reading') as HTMLElement | null
    if (reading) return reading

    const container = scrollContainerRef.value
    if (!container) return null

    const list = getFilteredParagraphs()
    for (const paragraph of list) {
      const top = paragraph.offsetTop - container.scrollTop
      const bottom = top + paragraph.offsetHeight
      if (bottom > 40) {
        return paragraph
      }
    }

    return list[0] || null
  }

  function getPrevParagraph() {
    const current = getCurrentParagraph()
    return getPrevParagraphFrom(current)
  }

  function getPrevParagraphFrom(current: HTMLElement | null) {
    const list = getFilteredParagraphs()
    const index = current ? list.indexOf(current) : -1
    if (index > 0) return list[index - 1]
    return null
  }

  function getNextParagraph() {
    const current = getCurrentParagraph()
    return getNextParagraphFrom(current)
  }

  function getNextParagraphFrom(current: HTMLElement | null) {
    const list = getFilteredParagraphs()
    const index = current ? list.indexOf(current) : -1
    if (index >= 0 && index < list.length - 1) return list[index + 1]
    return null
  }

  function splitLongSentence(sentence: string) {
    const chunks: string[] = []
    let remaining = sentence.trim()
    while (remaining.length > OPENAI_SPEECH_CHUNK_CHAR_LIMIT) {
      let splitIndex = Math.max(
        remaining.lastIndexOf('，', OPENAI_SPEECH_CHUNK_CHAR_LIMIT),
        remaining.lastIndexOf('、', OPENAI_SPEECH_CHUNK_CHAR_LIMIT),
        remaining.lastIndexOf(',', OPENAI_SPEECH_CHUNK_CHAR_LIMIT),
        remaining.lastIndexOf(' ', OPENAI_SPEECH_CHUNK_CHAR_LIMIT),
      )
      if (splitIndex <= 0) {
        splitIndex = OPENAI_SPEECH_CHUNK_CHAR_LIMIT
      }
      chunks.push(remaining.slice(0, splitIndex).trim())
      remaining = remaining.slice(splitIndex).trim()
    }
    if (remaining) chunks.push(remaining)
    return chunks
  }

  function buildParagraphSpeechChunks(paragraph: HTMLElement | null) {
    const rawText = paragraph?.innerText.trim() || ''
    if (!rawText) return [] as string[]

    const sentences = rawText
      .replace(/\n+/g, '\n')
      .split(/(?<=[。！？!?；;])/)
      .map((item) => item.trim())
      .filter(Boolean)

    const chunks: string[] = []
    let current = ''

    const pushCurrent = () => {
      const normalized = current.trim()
      if (normalized) chunks.push(normalized)
      current = ''
    }

    for (const sentence of (sentences.length ? sentences : [rawText])) {
      if (sentence.length > OPENAI_SPEECH_CHUNK_CHAR_LIMIT) {
        pushCurrent()
        chunks.push(...splitLongSentence(sentence))
        continue
      }
      const next = current ? `${current}${sentence}` : sentence
      if (next.length > OPENAI_SPEECH_CHUNK_CHAR_LIMIT) {
        pushCurrent()
        current = sentence
      } else {
        current = next
      }
    }

    pushCurrent()
    return chunks.length ? chunks : [rawText]
  }

  function resetSpeechChunkState() {
    currentSpeechParagraph = null
    currentSpeechChunks = []
    currentSpeechChunkIndex = 0
  }

  function ensureSpeechChunkState(paragraph: HTMLElement) {
    if (store.speechConfig.provider !== 'openai') {
      return {
        text: paragraph.innerText.trim(),
        nextParagraph: getNextParagraphFrom(paragraph),
      }
    }

    if (currentSpeechParagraph !== paragraph) {
      currentSpeechParagraph = paragraph
      currentSpeechChunks = buildParagraphSpeechChunks(paragraph)
      currentSpeechChunkIndex = 0
    }

    return {
      text: currentSpeechChunks[currentSpeechChunkIndex] || '',
      nextParagraph: currentSpeechChunkIndex < currentSpeechChunks.length - 1 ? paragraph : getNextParagraphFrom(paragraph),
    }
  }

  function getUpcomingSpeechChunks(startParagraph: HTMLElement | null) {
    const chunks: string[] = []

    if (store.speechConfig.provider === 'openai' && currentSpeechParagraph && currentSpeechChunks.length) {
      for (let i = currentSpeechChunkIndex + 1; i < currentSpeechChunks.length && chunks.length < OPENAI_PRELOAD_CHUNK_LIMIT; i += 1) {
        chunks.push(currentSpeechChunks[i])
      }
    }

    let cursor = startParagraph
    while (cursor && chunks.length < OPENAI_PRELOAD_CHUNK_LIMIT) {
      const paragraphChunks = buildParagraphSpeechChunks(cursor)
      for (const chunk of paragraphChunks) {
        if (chunks.length >= OPENAI_PRELOAD_CHUNK_LIMIT) break
        chunks.push(chunk)
      }
      const list = getFilteredParagraphs()
      const index = list.indexOf(cursor)
      cursor = index >= 0 ? (list[index + 1] || null) : null
    }

    return chunks
  }

  function clearReadingClass() {
    scrollContainerRef.value?.querySelectorAll('.reading').forEach((el) => el.classList.remove('reading'))
  }

  function showParagraph(paragraph: HTMLElement | null, smooth = true) {
    const container = scrollContainerRef.value
    if (!container || !paragraph) return

    const targetTop = Math.max(0, paragraph.offsetTop - 24)
    container.scrollTo({
      top: targetTop,
      behavior: smooth ? 'smooth' : 'auto',
    })
  }

  function markReadingParagraph(paragraph: HTMLElement | null) {
    clearReadingClass()
    if (paragraph) {
      paragraph.classList.add('reading')
    }
  }

  function runAutoScroll() {
    if (!store.isAutoScrolling || !scrollContainerRef.value) return

    const container = scrollContainerRef.value
    const speed = Math.max(1, config.value.scrollPixel) * (config.value.pageSpeed / 1000) * 0.5

    container.scrollTop += speed

    if (container.scrollTop + container.clientHeight >= container.scrollHeight - 2) {
      if (config.value.clickAction === 'auto' && store.hasNext) {
        void nextChapter()
      } else {
        stopAutoScroll()
      }
    } else {
      autoScrollId = requestAnimationFrame(runAutoScroll)
    }
  }

  function runAutoParagraph() {
    if (!store.isAutoScrolling) return
    if (autoReadingProcessing) return

    const list = getFilteredParagraphs()
    if (!list.length) return

    autoReadingProcessing = true

    if (autoReadingParagraphIndex < 0) {
      const current = getCurrentParagraph()
      autoReadingParagraphIndex = current ? Math.max(0, list.indexOf(current)) : 0
    }

    if (autoReadingParagraphIndex >= list.length) {
      autoReadingParagraphIndex = -1
      autoReadingProcessing = false
      if (store.hasNext) {
        Promise.resolve(nextChapter()).then(() => {
          window.setTimeout(() => {
            if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
              runAutoParagraph()
            }
          }, 300)
        })
      } else {
        stopAutoScroll()
      }
      return
    }

    const current = list[autoReadingParagraphIndex]
    markReadingParagraph(current)
    showParagraph(current)

    const estimatedLineCount = Math.max(1, Math.ceil(current.offsetHeight / (config.value.fontSize * config.value.lineHeight)))
    const delayTime = Math.max(300, config.value.pageSpeed * estimatedLineCount)

    autoReadingProcessing = false
    autoParagraphTimer = window.setTimeout(() => {
      autoReadingParagraphIndex += 1
      runAutoParagraph()
    }, delayTime)
  }

  function startAutoScroll() {
    if (config.value.autoPageMode === 'paragraph') {
      if (autoParagraphTimer) return
      runAutoParagraph()
      return
    }
    if (autoScrollId) return
    runAutoScroll()
  }

  function stopAutoScroll() {
    store.isAutoScrolling = false
    autoReadingParagraphIndex = -1
    autoReadingProcessing = false
    if (autoScrollId) {
      cancelAnimationFrame(autoScrollId)
      autoScrollId = null
    }
    if (autoParagraphTimer) {
      clearTimeout(autoParagraphTimer)
      autoParagraphTimer = null
    }
    if (!store.isSpeaking) {
      clearReadingClass()
    }
  }

  function restartSpeechTarget(paragraph: HTMLElement | null, interruptCurrent = true) {
    logSpeech('restartSpeechTarget', {
      interruptCurrent,
      paragraph: paragraphPreview(paragraph),
      isSpeechTransitioning,
    })
    if (!paragraph) {
      store.stopTTS()
      resetSpeechChunkState()
      return
    }
    if (isSpeechTransitioning) return
    isSpeechTransitioning = true
    resetSpeechChunkState()
    if (interruptCurrent) {
      store.stopTTS(false)
    }
    if (speechRestartTimer) {
      clearTimeout(speechRestartTimer)
    }
    const restartDelay = !interruptCurrent && store.speechConfig.provider === 'system'
      ? ((isSafariSpeechDelayBrowser() && !store.systemTtsNativeEventsReliable) ? 160 : 40)
      : 150
    speechRestartTimer = window.setTimeout(() => {
      if (store.isPaused) {
        isSpeechTransitioning = false
        return
      }
      isSpeechTransitioning = false
      startSpeech(paragraph, interruptCurrent)
    }, restartDelay)
  }

  function continueSpeechTarget(paragraph: HTMLElement | null, resetChunks = true) {
    logSpeech('continueSpeechTarget', {
      resetChunks,
      paragraph: paragraphPreview(paragraph),
      hasNextChapter: store.hasNext,
    })
    if (speechRestartTimer) {
      clearTimeout(speechRestartTimer)
    }

    const continueDelay = store.speechConfig.provider === 'system'
      ? ((isSafariSpeechDelayBrowser() && !store.systemTtsNativeEventsReliable) ? 160 : 40)
      : 120

    if (paragraph) {
      isSpeechTransitioning = true
      if (resetChunks) {
        resetSpeechChunkState()
      }
      speechRestartTimer = window.setTimeout(() => {
        if (store.isPaused) {
          isSpeechTransitioning = false
          return
        }
        isSpeechTransitioning = false
        startSpeech(paragraph, false)
      }, continueDelay)
      return
    }

    if (!store.hasNext) {
      store.stopTTS()
      clearReadingClass()
      return
    }

    isSpeechTransitioning = true
    if (resetChunks) {
      resetSpeechChunkState()
    }
    Promise.resolve(nextChapter())
      .then(() => {
        speechRestartTimer = window.setTimeout(() => {
          if (store.isPaused) {
            isSpeechTransitioning = false
            return
          }
          isSpeechTransitioning = false
          startSpeech(getFilteredParagraphs()[0] || null, false)
        }, continueDelay)
      })
      .catch(() => {
        isSpeechTransitioning = false
      })
  }

  function startSpeech(paragraph?: HTMLElement | null, interruptCurrent = true) {
    const current = paragraph || getCurrentParagraph()
    logSpeech('startSpeech', {
      interruptCurrent,
      paragraph: paragraphPreview(current),
      currentIndex: store.currentIndex,
    })
    if (!current?.innerText.trim()) {
      if (interruptCurrent) {
        speechNext()
      } else {
        continueSpeechTarget(getNextParagraph())
      }
      return
    }

    markReadingParagraph(current)
    showParagraph(current)
    const chunk = ensureSpeechChunkState(current)
    if (!chunk.text.trim()) {
      if (interruptCurrent) {
        speechNext(chunk.nextParagraph)
      } else {
        continueSpeechTarget(chunk.nextParagraph)
      }
      return
    }
    const nextParagraph = chunk.nextParagraph
    logSpeech('speak chunk', {
      interruptCurrent,
      provider: store.speechConfig.provider,
      text: chunk.text.slice(0, 60),
      nextParagraph: paragraphPreview(nextParagraph),
      chunkIndex: currentSpeechChunkIndex,
      chunkCount: currentSpeechChunks.length,
    })
    store.startTTS(chunk.text, {
      onEnd: () => {
        logSpeech('chunk onEnd', {
          provider: store.speechConfig.provider,
          currentParagraph: paragraphPreview(current),
          nextParagraph: paragraphPreview(nextParagraph),
          chunkIndex: currentSpeechChunkIndex,
          chunkCount: currentSpeechChunks.length,
        })
        if (store.speechConfig.provider === 'openai' && currentSpeechParagraph === current && currentSpeechChunkIndex < currentSpeechChunks.length - 1) {
          currentSpeechChunkIndex += 1
          continueSpeechTarget(current, false)
          return
        }
        continueSpeechTarget(nextParagraph)
      },
      onError: () => {
        logSpeech('chunk onError', {
          currentParagraph: paragraphPreview(current),
          nextParagraph: paragraphPreview(nextParagraph),
        })
        resetSpeechChunkState()
        clearReadingClass()
      },
    }, interruptCurrent)
    const preloadTexts = getUpcomingSpeechChunks(nextParagraph)
    if (preloadTexts.length) {
      window.setTimeout(() => {
        void store.preloadOpenAITTS(preloadTexts)
      }, 0)
    }
  }

  function speechPrev() {
    logSpeech('speechPrev', {
      currentParagraph: paragraphPreview(getCurrentParagraph()),
      hasPrevChapter: store.hasPrev,
    })
    resetSpeechChunkState()
    const prev = getPrevParagraph()
    if (prev) {
      restartSpeechTarget(prev)
      return
    }
    if (!store.hasPrev) {
      store.stopTTS()
      return
    }
    store.stopTTS(false)
    Promise.resolve(prevChapter()).then(() => {
      window.setTimeout(() => {
        const list = getFilteredParagraphs()
        restartSpeechTarget(list[list.length - 1] || null)
      }, 120)
    })
  }

  function speechNext(forcedNext?: HTMLElement | null, interruptCurrent = true) {
    logSpeech('speechNext', {
      interruptCurrent,
      forcedNext: paragraphPreview(forcedNext || null),
      currentParagraph: paragraphPreview(getCurrentParagraph()),
      hasNextChapter: store.hasNext,
    })
    resetSpeechChunkState()
    const next = forcedNext ?? getNextParagraph()
    if (next) {
      restartSpeechTarget(next, interruptCurrent)
      return
    }
    if (!store.hasNext) {
      store.stopTTS()
      clearReadingClass()
      return
    }
    if (interruptCurrent) {
      store.stopTTS(false)
    }
    Promise.resolve(nextChapter()).then(() => {
      window.setTimeout(() => {
        restartSpeechTarget(getFilteredParagraphs()[0] || null)
      }, 120)
    })
  }

  function restartSpeechFromCurrentParagraph() {
    logSpeech('restartSpeechFromCurrentParagraph', {
      currentParagraph: paragraphPreview(getCurrentParagraph()),
      isSpeechTransitioning,
    })
    if (isSpeechTransitioning) return
    isSpeechTransitioning = true
    resetSpeechChunkState()
    store.stopTTS(false)
    if (speechRestartTimer) {
      clearTimeout(speechRestartTimer)
    }
    speechRestartTimer = window.setTimeout(() => {
      if (store.isPaused) {
        isSpeechTransitioning = false
        return
      }
      isSpeechTransitioning = false
      startSpeech()
    }, 150)
  }

  function cancelSpeechTransition() {
    if (speechRestartTimer) {
      clearTimeout(speechRestartTimer)
      speechRestartTimer = null
    }
    isSpeechTransitioning = false
  }

  function resetAutoParagraphIndex() {
    autoReadingParagraphIndex = -1
  }

  function handleContentChanged() {
    autoReadingParagraphIndex = -1
    if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
      if (autoParagraphTimer) {
        clearTimeout(autoParagraphTimer)
        autoParagraphTimer = null
      }
      window.setTimeout(() => {
        if (store.isAutoScrolling && config.value.autoPageMode === 'paragraph') {
          runAutoParagraph()
        }
      }, 100)
    }
  }

  function disposeAutoPlayback() {
    cancelSpeechTransition()
    stopAutoScroll()
  }

  return {
    getCurrentParagraph,
    clearReadingClass,
    startAutoScroll,
    stopAutoScroll,
    startSpeech,
    speechPrev,
    speechNext,
    restartSpeechFromCurrentParagraph,
    cancelSpeechTransition,
    resetAutoParagraphIndex,
    handleContentChanged,
    disposeAutoPlayback,
  }
}
