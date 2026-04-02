<template>
  <div class="reader-catalog" :style="{ background: theme.popup, color: theme.fontColor }">
    <div class="catalog-header">
      <div class="tabs">
        <div 
          class="tab" 
          :class="{ active: activeTab === 'chapters' }" 
          @click="activeTab = 'chapters'"
        >
          目录
        </div>
        <div 
          class="tab" 
          :class="{ active: activeTab === 'bookmarks' }" 
          @click="activeTab = 'bookmarks'"
        >
          书签
        </div>
      </div>
      <button class="close-btn" @click="store.closePanel()">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12" /></svg>
      </button>
    </div>

    <!-- Chapters List -->
    <div v-show="activeTab === 'chapters'" class="list-container" ref="listRef">
      <div v-if="store.chaptersLoading" class="loading">加载目录中...</div>
      <div
        v-else
        v-for="(chapter, index) in store.chapters"
        :key="index"
        class="list-item"
        :class="{ active: index === store.currentIndex }"
        @click="goToChapter(index)"
      >
        <span class="item-index">{{ index + 1 }}</span>
        <span class="item-title">{{ chapter.title }}</span>
      </div>
    </div>

    <!-- Bookmarks List -->
    <div v-show="activeTab === 'bookmarks'" class="list-container">
      <div v-if="!store.bookmarks.length" class="empty">暂无书签</div>
      <div
        v-else
        v-for="(bm, idx) in store.bookmarks"
        :key="idx"
        class="list-item bookmark-item"
        @click="goToBookmark(bm)"
      >
        <div class="bm-header">
          <span class="bm-chapter">{{ bm.chapterName }}</span>
          <span class="bm-time">{{ formatDate(bm.time) }}</span>
        </div>
        <div class="bm-snippet">{{ bm.bookText }}</div>
        <button class="bm-delete" @click.stop="store.removeBookmark(bm)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /></svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useReaderStore } from '../../stores/reader'
import type { Bookmark } from '../../types'

const store = useReaderStore()
const theme = computed(() => store.currentTheme)
const activeTab = ref<'chapters' | 'bookmarks'>('chapters')
const listRef = ref<HTMLElement>()

onMounted(() => {
  scrollToCurrent()
  store.fetchBookmarks()
})

function scrollToCurrent() {
  nextTick(() => {
    const activeEl = listRef.value?.querySelector('.list-item.active')
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'center' })
    }
  })
}

async function goToChapter(index: number) {
  await store.loadChapter(index)
  store.closePanel()
}

async function goToBookmark(bm: Bookmark) {
  if (bm.chapterIndex !== undefined) {
    await store.loadChapter(bm.chapterIndex)
    // Position scrolling could be added here if needed
    store.closePanel()
  }
}

function formatDate(ts?: number) {
  if (!ts) return ''
  const d = new Date(ts)
  return `${d.getMonth() + 1}-${d.getDate()} ${d.getHours()}:${String(d.getMinutes()).padStart(2, '0')}`
}
</script>

<style scoped>
.reader-catalog {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.catalog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-bottom: 1px solid rgba(0,0,0,0.06);
  flex-shrink: 0;
  height: 56px;
}

.tabs {
  display: flex;
  gap: 20px;
  height: 100%;
}

.tab {
  height: 100%;
  display: flex;
  align-items: center;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  opacity: 0.6;
  border-bottom: 2px solid transparent;
  padding: 0 4px;
}

.tab.active {
  opacity: 1;
  color: var(--color-primary, #c97f3a);
  border-bottom-color: var(--color-primary, #c97f3a);
}

.close-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  color: inherit;
  opacity: 0.6;
  background: transparent;
  border: none;
  cursor: pointer;
}

.list-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.loading, .empty {
  padding: 40px;
  text-align: center;
  opacity: 0.5;
  font-size: 14px;
}

.list-item {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s;
  border-bottom: 1px solid rgba(0,0,0,0.02);
}

.list-item:hover {
  background: rgba(0,0,0,0.03);
}

.list-item.active {
  color: var(--color-primary, #c97f3a);
  background: rgba(201, 127, 58, 0.05);
}

.item-index {
  font-size: 11px;
  opacity: 0.4;
  width: 24px;
  flex-shrink: 0;
}

.item-title {
  font-size: 14px;
  line-height: 1.4;
}

/* Bookmark items */
.bookmark-item {
  flex-direction: column;
  gap: 6px;
  position: relative;
}

.bm-header {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.bm-chapter {
  font-size: 13px;
  font-weight: 600;
  opacity: 0.9;
}

.bm-time {
  font-size: 11px;
  opacity: 0.4;
}

.bm-snippet {
  font-size: 12px;
  opacity: 0.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.5;
}

.bm-delete {
  position: absolute;
  right: 12px;
  bottom: 12px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  background: rgba(255,0,0,0.05);
  color: #ef4444;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: 0.2s;
}

.bookmark-item:hover .bm-delete {
  opacity: 1;
}

.bm-delete svg {
  width: 14px;
  height: 14px;
}
</style>
