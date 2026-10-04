import { computed, ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import { useReaderAutoPlayback } from './useReaderAutoPlayback'

describe('useReaderAutoPlayback', () => {
  it('starts speech when chapter HTML contains br elements without paragraphs', () => {
    const chapterText = fakeElement('第一段\n第二段', [], '第一段<br>第二段')
    const scrollContainer = fakeElement('')
    const originalHtml = chapterText.innerHTML
    const startTTS = vi.fn()
    const store = {
      speechConfig: {
        provider: 'openai',
        openaiRequestMode: 'chunked',
      },
      currentIndex: 0,
      hasNext: false,
      hasPrev: false,
      isPaused: false,
      isAutoScrolling: false,
      startTTS,
      preloadOpenAITTS: vi.fn(),
      stopTTS: vi.fn(),
    } as unknown as Parameters<typeof useReaderAutoPlayback>[0]

    const playback = useReaderAutoPlayback(
      store,
      computed(() => ({
        autoPageMode: 'pixel',
        clickAction: 'none',
        scrollPixel: 1,
        pageSpeed: 1_000,
        fontSize: 16,
        lineHeight: 1.5,
      })),
      computed(() => false),
      ref(scrollContainer),
      ref(chapterText),
      vi.fn(),
      vi.fn(),
    )

    playback.startSpeech()

    expect(startTTS).toHaveBeenCalledTimes(1)
    expect(startTTS.mock.calls[0]?.[0]).toContain('第一段')
    expect(chapterText.innerHTML).toBe(originalHtml)
  })

  it('keeps paragraph elements as separate speech targets when they exist', () => {
    const firstParagraph = fakeElement('第一段')
    const secondParagraph = fakeElement('第二段')
    const chapterText = fakeElement('第一段\n第二段', [firstParagraph, secondParagraph])
    const scrollContainer = fakeElement('')
    const startTTS = vi.fn()
    const store = {
      speechConfig: {
        provider: 'system',
        openaiRequestMode: 'chunked',
      },
      currentIndex: 0,
      hasNext: false,
      hasPrev: false,
      isPaused: false,
      isAutoScrolling: false,
      startTTS,
      preloadOpenAITTS: vi.fn(),
      stopTTS: vi.fn(),
    } as unknown as Parameters<typeof useReaderAutoPlayback>[0]

    const playback = useReaderAutoPlayback(
      store,
      computed(() => ({
        autoPageMode: 'pixel',
        clickAction: 'none',
        scrollPixel: 1,
        pageSpeed: 1_000,
        fontSize: 16,
        lineHeight: 1.5,
      })),
      computed(() => false),
      ref(scrollContainer),
      ref(chapterText),
      vi.fn(),
      vi.fn(),
    )

    playback.startSpeech()

    expect(startTTS).toHaveBeenCalledTimes(1)
    expect(startTTS.mock.calls[0]?.[0]).toBe('第一段')
  })
})

function fakeElement(innerText: string, paragraphs: HTMLElement[] = [], innerHTML = '') {
  return {
    innerText,
    innerHTML,
    offsetTop: 0,
    offsetHeight: 20,
    scrollTop: 0,
    classList: {
      add: vi.fn(),
      remove: vi.fn(),
    },
    querySelector: vi.fn(() => null),
    querySelectorAll: vi.fn((selector: string) => selector === 'p' ? paragraphs : []),
    scrollTo: vi.fn(),
  } as unknown as HTMLElement
}
