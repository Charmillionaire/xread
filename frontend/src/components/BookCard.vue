<template>
  <div
    class="book-card"
    :class="{ 'edit-mode': editMode, 'selected': selected, 'dragging': dragging }"
    @click="handleCardClick"
  >
    <!-- Cover -->
    <div class="card-cover" @click.stop="handleCoverClick">
      <button
        v-if="showDeleteAction && !editMode"
        class="card-delete-btn"
        title="删除最近阅读"
        @click.stop="$emit('delete', book)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
      <img
        v-if="coverSrc"
        :src="coverSrc"
        :alt="book.name"
        class="cover-img"
        loading="lazy"
        @error="coverFailed = true"
      />
      <div v-else class="cover-placeholder">
        <span class="cover-title">{{ book.name }}</span>
        <span class="cover-author">{{ book.author }}</span>
      </div>

      <!-- Unread badge -->
      <div v-if="unreadCount > 0 && !editMode" class="unread-badge">
        {{ unreadCount > 99 ? '99+' : unreadCount }}
      </div>

      <!-- Selection overlay -->
      <div v-if="editMode" class="selection-overlay">
        <div class="checkbox" :class="{ checked: selected }">
          <svg v-if="selected" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M20 6 9 17l-5-5" />
          </svg>
        </div>
      </div>
    </div>

    <!-- Info -->
    <div class="card-info">
      <div class="card-topline">
        <div class="title-block">
          <h3 class="book-name">{{ book.name }}</h3>
          <div class="book-meta">
            <span class="book-author">{{ book.author || '未知作者' }}</span>
            <span v-if="asBook.totalChapterNum" class="meta-dot">·</span>
            <span v-if="asBook.totalChapterNum" class="book-chapters">共{{ asBook.totalChapterNum }}章</span>
          </div>
        </div>
        <button
          v-if="showAiEntry"
          class="ai-entry-btn"
          @click.stop="$emit('ai', book)"
        >
          AI
        </button>
      </div>

      <div v-if="isSearch && (sourceName || sourceGroup)" class="book-source-row">
        <span v-if="sourceName" class="source-chip source-name">{{ sourceName }}</span>
        <span v-if="sourceGroup" class="source-chip source-group">{{ sourceGroup }}</span>
      </div>

      <!-- 发现页/搜索结果：展示标签（状态/评分/分类/更新时间等） -->
      <div v-if="isSearch && kindChips.length > 0" class="book-kind-row">
        <span
          v-for="(chip, idx) in kindChips"
          :key="idx"
          class="kind-chip"
          :class="chip.type"
        >
          {{ chip.text }}
        </span>
      </div>

      <!-- 发现页/搜索结果：展示简介（最多 3 行） -->
      <p v-if="isSearch && bookIntro" class="book-intro">
        {{ bookIntro }}
      </p>

      <div v-if="!isSearch" class="chapter-lines">
        <p v-if="asBook.durChapterTitle" class="book-progress">
          已读：{{ asBook.durChapterTitle }}
        </p>
        <p v-if="latestChapterText" class="book-latest">
          最新：{{ latestChapterText }}
        </p>
      </div>

      <div class="card-footer">
        <div v-if="!isSearch && (browserCachedCount > 0 || serverCachedCount > 0 || groupName)" class="book-cache-row">
          <span v-if="groupName" class="cache-chip group-chip">#{{ groupName }}</span>
          <span v-if="browserCachedCount > 0" class="cache-chip primary">离线 {{ browserCachedCount }} 章</span>
          <span v-if="serverCachedCount > 0" class="cache-chip">服务端 {{ serverCachedCount }} 章</span>
        </div>
        <!-- Search mode: add to shelf -->
        <button
          v-if="isSearch"
          class="add-shelf-btn"
          @click.stop="$emit('addToShelf', book)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <path d="M12 5v14M5 12h14" />
          </svg>
          加入书架
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { getCoverUrl } from '../api/bookshelf'
import { isLocalBook } from '../utils/localBook'
import { useBookshelfStore } from '../stores/bookshelf'
import type { Book, SearchBook } from '../types'

const props = defineProps<{
  book: Book | SearchBook
  editMode?: boolean
  selected?: boolean
  isSearch?: boolean
  dragging?: boolean
  showDeleteAction?: boolean
}>()

const emit = defineEmits<{
  click: [book: Book | SearchBook]
  info: [book: Book | SearchBook]
  delete: [book: Book | SearchBook]
  ai: [book: Book | SearchBook]
  addToShelf: [book: Book | SearchBook]
  select: [book: Book | SearchBook]
}>()

const shelfStore = useBookshelfStore()

const groupName = computed(() => {
  if (props.isSearch) return ''
  const gId = (props.book as Book).group
  if (!gId) return ''
  return shelfStore.groups.find((g) => g.groupId === gId)?.groupName || ''
})

function handleCardClick() {
  if (props.editMode) {
    emit('select', props.book)
  } else {
    emit('click', props.book)
  }
}

function handleCoverClick() {
  if (props.editMode) {
    emit('select', props.book)
  } else {
    emit('info', props.book)
  }
}

const coverFailed = ref(false)

const asBook = computed(() => props.book as Book)
const asSearchBook = computed(() => props.book as SearchBook)

const coverSrc = computed(() => {
  if (coverFailed.value) return ''
  const url = (props.book as Book).customCoverUrl || props.book.coverUrl
  if (!url) return ''
  return getCoverUrl(url)
})

const unreadCount = computed(() => {
  const b = props.book as Book
  if (!b.totalChapterNum || b.durChapterIndex === undefined) return 0
  return Math.max(0, b.totalChapterNum - 1 - b.durChapterIndex)
})

const browserCachedCount = computed(() => isLocalBook(asBook.value) ? 0 : Math.max(0, asBook.value.browserCachedChapterCount || 0))
const serverCachedCount = computed(() => isLocalBook(asBook.value) ? 0 : Math.max(0, asBook.value.cachedChapterCount || 0))
const latestChapterText = computed(() => {
  if (props.isSearch) {
    return asSearchBook.value.lastChapter || asBook.value.latestChapterTitle || ''
  }
  return asBook.value.latestChapterTitle || ''
})
const sourceName = computed(() => {
  if (!props.isSearch) return ''
  return asSearchBook.value.originName || props.book.origin || ''
})
const sourceGroup = computed(() => {
  if (!props.isSearch) return ''
  return asSearchBook.value.originGroup || ''
})
const showAiEntry = computed(() => {
  if (props.isSearch || props.editMode) return false
  const currentBook = props.book as Book
  return currentBook.recentKind !== 'rss'
})

interface KindChip {
  text: string
  type: 'status' | 'score' | 'time' | 'category'
}

/**
 * 将书源的 kind 字段（如 "连载中,9.4,传统玄幻,2024-01-11 18:10:39"）切分为独立标签，
 * 并根据内容分别赋予状态/评分/时间/分类类型。
 */
const kindChips = computed<KindChip[]>(() => {
  if (!props.isSearch) return []
  const raw = asSearchBook.value.kind || ''
  if (!raw.trim()) return []
  const parts = raw
    .split(/[,，、|/]/)
    .map((s) => s.trim())
    .filter(Boolean)

  return parts.map((text) => {
    if (/^(连载|完结|连载中|已完结|全本)$/.test(text)) {
      return { text, type: 'status' }
    }
    if (/^\d+(\.\d+)?分?$/.test(text) && parseFloat(text) <= 10) {
      return { text, type: 'score' }
    }
    if (/^\d{4}-\d{1,2}-\d{1,2}/.test(text)) {
      return { text, type: 'time' }
    }
    return { text, type: 'category' }
  })
})

const bookIntro = computed(() => {
  if (!props.isSearch) return ''
  return asSearchBook.value.intro?.trim() || ''
})
</script>

<style scoped>
.book-card {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr);
  align-items: center;
  gap: 15px;
  height: 100%;
  min-height: 148px;
  padding: 13px 14px;
  border-radius: var(--radius-lg, 14px);
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow:
    var(--glass-shadow),
    var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  cursor: pointer;
  transition:
    background var(--duration-normal) var(--ease-out),
    border-color var(--duration-normal) var(--ease-out),
    box-shadow var(--duration-normal) var(--ease-out),
    transform var(--duration-normal) var(--ease-out);
  position: relative;
  overflow: hidden;
}

.book-card::before {
  content: "";
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 0 0, rgba(244, 63, 94, 0.08), transparent 34%),
    linear-gradient(90deg, rgba(244, 63, 94, 0.035), transparent 42%);
  opacity: 0;
  transition: opacity var(--duration-normal) var(--ease-out);
  pointer-events: none;
}

.book-card:hover {
  background: var(--glass-bg-hover);
  border-color: rgba(255, 255, 255, 0.85);
  box-shadow:
    var(--glass-shadow-hover),
    var(--glass-inset-highlight);
  transform: translateY(-2px);
}

.book-card:hover::before {
  opacity: 1;
}

.book-card:active {
  transform: translateY(0);
}

.card-cover {
  width: 86px;
  height: 118px;
  flex-shrink: 0;
  border-radius: 6px;
  overflow: hidden;
  position: relative;
  background: var(--color-bg-sunken);
  box-shadow:
    0 10px 18px rgba(48, 35, 20, 0.16),
    0 0 0 1px rgba(0, 0, 0, 0.04);
}

.card-cover::after {
  content: "";
  position: absolute;
  inset: 0 auto 0 0;
  width: 13px;
  background: linear-gradient(90deg, rgba(0, 0, 0, 0.16), transparent);
  pointer-events: none;
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-2);
  background:
    linear-gradient(160deg, rgba(244, 63, 94, 0.12), rgba(70, 134, 121, 0.08)),
    var(--color-bg-sunken);
  text-align: center;
  gap: var(--space-1);
}

.cover-title {
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--color-primary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-tight);
}

.cover-author {
  font-size: 10px;
  color: var(--color-text-tertiary);
}

.unread-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-dark));
  color: white;
  font-size: 10px;
  font-weight: 800;
  padding: 2px 7px;
  border-radius: var(--radius-full);
  min-width: 18px;
  text-align: center;
  line-height: 15px;
  box-shadow: 0 6px 14px rgba(244, 63, 94, 0.28);
  z-index: 1;
}

.card-delete-btn {
  position: absolute;
  top: var(--space-1);
  left: var(--space-1);
  z-index: 2;
  width: 28px;
  height: 28px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(12, 14, 18, 0.58);
  color: white;
  backdrop-filter: blur(8px);
  transition: transform var(--duration-fast), background var(--duration-fast);
}

.card-delete-btn:hover {
  background: rgba(185, 44, 44, 0.9);
  transform: scale(1.06);
}

.card-delete-btn svg {
  width: 14px;
  height: 14px;
}

.selection-overlay {
  position: absolute;
  inset: 0;
  background: rgba(var(--color-primary-rgb), 0.1);
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: var(--space-1);
  opacity: 1;
}

.checkbox {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-full);
  border: 2px solid white;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast);
}

.checkbox.checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.checkbox svg {
  width: 14px;
  height: 14px;
  color: white;
}

.book-card.selected {
  border-color: var(--color-primary);
  background: rgba(var(--color-primary-rgb), 0.05);
}

.book-card.dragging {
  opacity: 0.55;
  transform: scale(0.98);
  box-shadow: none;
}

.edit-btn {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast);
}

.edit-btn svg {
  width: 18px;
  height: 18px;
}

.edit-btn.delete {
  background: var(--color-danger);
  color: white;
}

.edit-btn.delete:hover {
  background: #ff4d4f;
  transform: scale(1.1);
}

.card-info {
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 8px;
  align-self: stretch;
  padding: 2px 0;
  position: relative;
  z-index: 1;
}

.card-topline {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 0;
}

.title-block {
  min-width: 0;
  flex: 1;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  flex-wrap: wrap;
  gap: 10px;
  min-width: 0;
  min-height: 25px;
}

.book-name {
  margin: 0;
  font-size: 16px;
  font-weight: 800;
  color: var(--color-text);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-tight);
}

.book-meta {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  min-height: 18px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  margin-top: 4px;
}

.book-author {
  font-size: var(--text-xs);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.meta-dot {
  font-size: 10px;
}

.book-progress,
.book-latest {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: var(--leading-normal);
}

.book-latest {
  color: var(--color-text-tertiary);
}

.chapter-lines {
  display: grid;
  gap: 3px;
  min-height: 35px;
}

.book-source-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-height: 24px;
}

.source-chip {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.source-name {
  background: rgba(244, 63, 94, 0.12);
  color: var(--color-primary);
}

.source-group {
  background: rgba(0, 0, 0, 0.05);
  color: var(--color-text-secondary);
}

.book-kind-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  min-height: 20px;
}

.kind-chip {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 11px;
  line-height: 1.35;
  white-space: nowrap;
  background: var(--color-bg-sunken);
  color: var(--color-text-secondary);
}

.kind-chip.status {
  background: rgba(244, 63, 94, 0.12);
  color: var(--color-primary);
  font-weight: 600;
}

.kind-chip.score {
  background: rgba(234, 179, 8, 0.15);
  color: #ca8a04;
  font-weight: 700;
}

.kind-chip.category {
  background: rgba(59, 130, 246, 0.1);
  color: #2563eb;
}

.kind-chip.time {
  background: rgba(0, 0, 0, 0.04);
  color: var(--color-text-tertiary);
  font-size: 10.5px;
}

.book-intro {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  color: var(--color-text-tertiary);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
}

.book-cache-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
  flex: 1 1 auto;
}

.cache-chip {
  display: inline-flex;
  align-items: center;
  min-height: 25px;
  padding: 0 10px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.04);
  color: var(--color-text-secondary);
  font-size: 11.5px;
  line-height: 1;
  white-space: nowrap;
}

.cache-chip.primary {
  background: rgba(244, 63, 94, 0.13);
  color: var(--color-primary);
}

.ai-entry-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  flex: 0 0 auto;
  min-height: 28px;
  padding: 0 10px;
  margin-left: auto;
  background: rgba(244, 63, 94, 0.075);
  color: var(--color-primary);
  border: 1px solid rgba(244, 63, 94, 0.24);
  border-radius: 7px;
  font-size: 12px;
  font-weight: 800;
  transition: all var(--duration-fast);
  white-space: nowrap;
}

.ai-entry-btn:hover {
  background: rgba(244, 63, 94, 0.16);
  border-color: rgba(244, 63, 94, 0.42);
  transform: translateY(-1px);
}

.add-shelf-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  flex: 0 0 auto;
  min-height: 30px;
  padding: 0 12px;
  background: var(--color-primary);
  color: white;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  transition: all var(--duration-fast);
  align-self: flex-start;
}

.add-shelf-btn:hover {
  background: var(--color-primary-dark);
  transform: scale(1.02);
}

.add-shelf-btn:active {
  transform: scale(0.98);
}

@media (max-width: 520px) {
  .book-card {
    grid-template-columns: 76px minmax(0, 1fr);
    gap: 13px;
    padding: 12px;
  }

  .card-cover {
    width: 76px;
    height: 106px;
  }

  .book-name {
    font-size: 15px;
  }

  .card-footer {
    gap: 7px;
  }

  .ai-entry-btn {
    min-height: 30px;
    padding: 0 11px;
    font-size: 12px;
  }
}
</style>
