<template>
  <div class="reader-source" :style="{ background: theme.popup, color: theme.fontColor }">
    <div class="source-header">
      <div class="header-left">
        <h3>切换书源</h3>
        <span class="source-count" v-if="results.length">{{ results.length }} 个结果</span>
      </div>
      <button class="close-btn" @click="store.closePanel()">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12" /></svg>
      </button>
    </div>
    
    <div class="source-list" ref="listRef">
      <!-- Current Source Header -->
      <div class="section-label" v-if="currentSource">当前书源</div>
      <div v-if="currentSource" class="source-item current">
        <div class="source-name">{{ currentSource.originName || currentSource.origin }}</div>
        <div class="source-url">{{ currentSource.origin }}</div>
      </div>

      <div class="section-label">其他可用源</div>
      
      <div v-if="searching" class="loading">
        <div class="spinner"></div>
        正在全网搜索同名书籍...
      </div>
      
      <div v-else-if="!results.length" class="empty">未找到其他书源</div>
      
      <div
        v-else
        v-for="res in results"
        :key="res.bookUrl + res.origin"
        class="source-item"
        @click="handleSwitch(res)"
      >
        <div class="source-main">
          <div class="source-name-row">
            <span class="source-name">{{ res.origin }}</span>
            <span class="source-tag" v-if="res.kind">{{ res.kind }}</span>
          </div>
          <div class="source-author">{{ res.author }}</div>
          <div class="source-chapter" v-if="res.lastChapter">最新: {{ res.lastChapter }}</div>
        </div>
        <div class="source-action">
          <button class="switch-btn">切换</button>
        </div>
      </div>

      <div v-if="results.length" class="load-more-wrap">
        <button class="load-more-btn" :disabled="loadingMore" @click="loadMoreSources">
          {{ loadingMore ? '加载中...' : '加载更多' }}
        </button>
      </div>
    </div>

    <!-- Switching Overlay -->
    <Transition name="fade">
      <div v-if="store.loading" class="switch-overlay" :style="{ background: theme.popup }">
        <div class="spinner"></div>
        <p>正在切换书源，请稍候...</p>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useReaderStore } from '../../stores/reader'
import { useAppStore } from '../../stores/app'
import { getAvailableBookSource, searchBookSourceSSE } from '../../api/search'
import type { SearchBook } from '../../types'

const store = useReaderStore()
const appStore = useAppStore()
const theme = computed(() => store.currentTheme)
const searching = ref(false)
const loadingMore = ref(false)
const results = ref<SearchBook[]>([])
const lastIndex = ref(-1)
let sourceSSE: EventSource | null = null

const currentSource = computed(() => {
  if (!store.book) return null
  return {
    origin: store.book.origin,
    originName: store.book.originName
  }
})

onMounted(() => {
  startSearch()
})

onUnmounted(() => {
  stopLoadMore()
})

async function startSearch() {
  if (!store.book) return
  searching.value = true
  results.value = []
  lastIndex.value = -1

  try {
    const candidates = await getAvailableBookSource({
      url: store.book.bookUrl,
      name: store.book.name,
      author: store.book.author,
    })
    mergeCandidates(candidates)
    lastIndex.value = Math.max(lastIndex.value, candidates.length - 1)
  } catch (e) {
    console.error('getAvailableBookSource failed', e)
  } finally {
    searching.value = false
  }
}

function stopLoadMore() {
  if (sourceSSE) {
    sourceSSE.close()
    sourceSSE = null
  }
}

function mergeCandidates(candidates: SearchBook[]) {
  if (!store.book || !candidates.length) return
  const currentBook = store.book
  candidates.forEach((item) => {
    if (item.origin === currentBook.origin) return
    if (currentBook.author && item.author && item.author !== currentBook.author) return
    const existed = results.value.some((candidate) =>
      candidate.origin === item.origin || (candidate.bookUrl === item.bookUrl && candidate.origin === item.origin),
    )
    if (!existed) {
      results.value.push(item)
    }
  })
}

function loadMoreSources() {
  if (!store.book || loadingMore.value) return

  stopLoadMore()
  loadingMore.value = true
  const beforeCount = results.value.length
  sourceSSE = searchBookSourceSSE({
    concurrentCount: 24,
    url: store.book.bookUrl,
    bookSourceGroup: '',
    lastIndex: lastIndex.value,
  })

  sourceSSE.onmessage = (event) => {
    try {
      const payload = JSON.parse(event.data)
      if (typeof payload.lastIndex === 'number') {
        lastIndex.value = payload.lastIndex
      }
      const batch = Array.isArray(payload?.data) ? payload.data : []
      if (batch.length) {
        mergeCandidates(batch)
      }
    } catch (error) {
      console.error('searchBookSourceSSE message parse failed', error)
    }
  }

  sourceSSE.addEventListener('end', (event) => {
    const afterCount = results.value.length
    try {
      const payload = JSON.parse((event as MessageEvent).data)
      if (typeof payload.lastIndex === 'number') {
        lastIndex.value = payload.lastIndex
      }
    } catch {
      //
    }
    loadingMore.value = false
    stopLoadMore()
    if (afterCount > beforeCount) {
      appStore.showToast(`已新增 ${afterCount - beforeCount} 个书源`, 'success')
    } else {
      appStore.showToast('没有更多书源了', 'warning')
    }
  })

  sourceSSE.addEventListener('error', (event) => {
    console.error('searchBookSourceSSE failed', event)
    loadingMore.value = false
    stopLoadMore()
    appStore.showToast('加载更多书源失败', 'error')
  })
}

async function handleSwitch(res: SearchBook) {
  if (store.loading) return
  try {
    await store.switchSource(res.bookUrl, res.origin)
    store.closePanel()
  } catch (e: any) {
    alert('切换失败: ' + e.message)
  }
}
</script>

<style scoped>
.reader-source {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

.source-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0,0,0,0.06);
  flex-shrink: 0;
}

.header-left { display: flex; align-items: baseline; gap: 8px; }
.source-header h3 { font-size: 16px; margin: 0; }
.source-count { font-size: 11px; opacity: 0.5; }

.close-btn {
  width: 32px; height: 32px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 8px; color: inherit; opacity: 0.6;
  background: transparent; border: none; cursor: pointer;
}

.source-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.section-label {
  padding: 12px 20px 4px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.4;
  font-weight: 600;
}

.source-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid rgba(0,0,0,0.02);
  cursor: pointer;
  transition: background 0.2s;
}

.source-item:hover { background: rgba(0,0,0,0.03); }
.source-item.current { background: rgba(201, 127, 58, 0.04); cursor: default; }

.source-main { flex: 1; min-width: 0; }
.source-name-row { display: flex; align-items: center; gap: 8px; margin-bottom: 2px; }
.source-name { font-weight: 600; font-size: 14px; }
.source-tag { font-size: 10px; opacity: 0.5; border: 1px solid currentColor; padding: 0 3px; border-radius: 3px; }

.source-author { font-size: 11px; opacity: 0.5; margin-bottom: 4px; }
.source-chapter { font-size: 11px; opacity: 0.7; color: var(--color-primary, #c97f3a); }

.source-url { font-size: 11px; opacity: 0.3; }

.switch-btn {
  padding: 4px 12px;
  font-size: 11px;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.7;
}

.source-item:hover .switch-btn {
  background: var(--color-primary, #c97f3a);
  color: white;
  border-color: var(--color-primary, #c97f3a);
  opacity: 1;
}

.loading, .empty {
  padding: 40px 20px;
  text-align: center;
  opacity: 0.5;
  font-size: 14px;
}

.load-more-wrap {
  padding: 16px 20px 24px;
  display: flex;
  justify-content: center;
}

.load-more-btn {
  min-width: 120px;
  padding: 10px 18px;
  border-radius: 999px;
  border: 1px solid rgba(0,0,0,0.08);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.load-more-btn:hover:not(:disabled) {
  background: rgba(201, 127, 58, 0.08);
  border-color: var(--color-primary, #c97f3a);
  color: var(--color-primary, #c97f3a);
}

.load-more-btn:disabled {
  opacity: 0.5;
  cursor: wait;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(0,0,0,0.1);
  border-top-color: var(--color-primary, #c97f3a);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 12px;
}

@keyframes spin { to { transform: rotate(360deg); } }

.switch-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  z-index: 20;
}

.switch-overlay p { margin-top: 16px; font-size: 14px; opacity: 0.8; }
</style>
