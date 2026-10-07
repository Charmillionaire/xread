<template>
  <div class="explore-view">
    <!-- 顶部标题 -->
    <header class="explore-header">
      <h2>发现</h2>
    </header>

    <!-- 筛选控件：下拉切换（类型/频道/平台/字数/更新/排序等） -->
    <div v-if="store.filterKinds.length" class="filter-bar">
      <label v-for="kind in store.filterKinds" :key="kind.paramKey || kind.title" class="filter-chip">
        <span class="filter-chip-label">{{ kind.title }}</span>
        <select
          :value="store.variables[kind.paramKey || kind.title] || kind.default || ''"
          @change="onVariableChange(kind, $event)"
        >
          <option v-for="opt in optionsFor(kind)" :key="opt" :value="opt">{{ opt }}</option>
        </select>
      </label>
    </div>

    <!-- 榜单 / 分类标签 + 操作按钮 -->
    <div v-if="store.rankingKinds.length || store.buttonKinds.length" class="ranking-bar">
      <button
        v-for="kind in store.rankingKinds"
        :key="kind.url || kind.title"
        class="ranking-chip"
        :class="{ active: store.activeCategoryUrl === kind.url }"
        @click="onCategoryClick(kind)"
      >
        {{ kind.title }}
      </button>
      <span
        v-for="kind in store.buttonKinds"
        :key="'btn-' + kind.title"
        class="action-chip"
        :title="`由书源提供：${kind.title}`"
      >
        {{ kind.title }}
      </span>
    </div>

    <!-- 书籍列表 -->
    <div class="content-panel" ref="scrollContainer" @scroll="handleScroll">
      <div v-if="store.books.length > 0" class="book-list">
        <article
          v-for="(book, index) in store.books"
          :key="book.bookUrl + '-' + index"
          class="book-item"
          @click="handleBookClick(book)"
        >
          <img
            v-if="coverOf(book)"
            class="book-cover"
            :src="coverOf(book)"
            :alt="book.name"
            loading="lazy"
            @error="markCoverFailed(book.bookUrl)"
          />
          <div v-else class="book-cover placeholder">{{ book.name.charAt(0) }}</div>

          <div class="book-info">
            <h3 class="book-name">{{ book.name }}</h3>
            <p class="book-author">作者：{{ book.author || '佚名' }}</p>

            <div class="book-tags">
              <span v-if="book.kind" class="tag kind">{{ book.kind }}</span>
              <span v-if="book.wordCount" class="tag">{{ book.wordCount }}</span>
              <span v-if="book.originName" class="tag origin">{{ book.originName }}</span>
            </div>

            <p v-if="book.lastChapter" class="book-latest">
              <span>最新：</span>{{ book.lastChapter }}
            </p>
            <p v-if="book.intro" class="book-intro">{{ book.intro }}</p>
          </div>

          <button class="shelf-btn" @click.stop="handleAddToShelf(book)">加书架</button>
        </article>
      </div>

      <div class="loading-state" v-if="store.loading">
        <div class="spinner"></div>
        加载中...
      </div>

      <div class="end-state" v-else-if="!store.hasMore && store.books.length > 0">没有更多了</div>

      <div class="error-state" v-if="store.error">{{ store.error }}</div>

      <div
        class="empty-state"
        v-if="!store.loading && !store.error && store.books.length === 0"
      >
        {{ store.activeCategoryUrl ? '暂无数据' : '请选择书源或榜单' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useExploreStore } from '../stores/explore'
import { useReaderStore } from '../stores/reader'
import { getCoverUrl, saveBook } from '../api/bookshelf'
import { useAppStore } from '../stores/app'
import type { Book, SearchBook, ExploreKind } from '../types'

const store = useExploreStore()
const readerStore = useReaderStore()
const appStore = useAppStore()
const router = useRouter()

const scrollContainer = ref<HTMLElement>()
const openingBookUrl = ref('')
const failedCovers = ref<Set<string>>(new Set())

onMounted(async () => {
  await store.init()
})

function onVariableChange(kind: ExploreKind, event: Event) {
  const key = kind.paramKey || kind.title
  store.setVariable(key, (event.target as HTMLSelectElement).value)
}

/**
 * 下拉候选项。部分控件（如「平台」）的候选由书源云端配置动态提供，
 * 首次拿不到 chars 时至少保留当前值，避免下拉框空白无法展示。
 */
function optionsFor(kind: ExploreKind) {
  const chars = kind.chars ?? []
  if (chars.length > 0) return chars
  const current = store.variables[kind.paramKey || kind.title] || kind.default || ''
  return current ? [current] : []
}

function onCategoryClick(kind: ExploreKind) {
  if (!kind.url) return
  store.setCategory(kind.url)
}

function handleScroll() {
  const el = scrollContainer.value
  if (!el) return
  const { scrollTop, scrollHeight, clientHeight } = el
  if (scrollTop + clientHeight >= scrollHeight - 120) {
    store.fetchMore()
  }
}

function coverOf(book: SearchBook) {
  if (failedCovers.value.has(book.bookUrl)) return ''
  return book.coverUrl ? getCoverUrl(book.coverUrl) : ''
}

function markCoverFailed(bookUrl: string) {
  const next = new Set(failedCovers.value)
  next.add(bookUrl)
  failedCovers.value = next
}

async function handleBookClick(book: Book | SearchBook) {
  const b = book as Book
  if (!b.origin || !b.bookUrl) return
  if (openingBookUrl.value === b.bookUrl) return

  openingBookUrl.value = b.bookUrl
  const targetIndex = b.durChapterIndex || 0

  try {
    const loadBookTask = readerStore.loadBook(b)
    await router.push('/reader')
    await loadBookTask
    await readerStore.loadChapter(targetIndex)
  } finally {
    openingBookUrl.value = ''
  }
}

async function handleAddToShelf(book: Book | SearchBook) {
  try {
    await saveBook({
      name: book.name,
      author: book.author,
      bookUrl: book.bookUrl,
      origin: book.origin,
      coverUrl: book.coverUrl,
    })
    appStore.showToast(`"${book.name}" 已加入书架`, 'success')
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  }
}
</script>

<style scoped>
.explore-view {
  height: 100%;
  min-height: 0;
  max-width: var(--content-max-width);
  margin: 0 auto;
  width: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg);
  overflow: hidden;
}

.explore-header {
  padding: var(--space-5) var(--space-6) var(--space-3);
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
}

.explore-header h2 {
  font-size: var(--text-2xl);
  font-weight: 700;
  margin: 0;
  color: var(--color-text);
  letter-spacing: -0.02em;
  text-align: center;
}

/* 筛选控件 */
.filter-bar {
  padding: 0 var(--space-6) var(--space-3);
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
}

.filter-chip-label {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
  white-space: nowrap;
}

.filter-chip select {
  border: none;
  background: transparent;
  color: var(--color-text);
  font-size: var(--text-sm);
  font-weight: 600;
  outline: none;
  cursor: pointer;
  max-width: 160px;
}

/* 榜单标签栏 */
.ranking-bar {
  padding: 0 var(--space-6) var(--space-3);
  display: flex;
  gap: var(--space-2);
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  flex-shrink: 0;
}

.ranking-bar::-webkit-scrollbar {
  display: none;
}

.ranking-chip {
  flex-shrink: 0;
  padding: 8px 16px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.ranking-chip:hover {
  color: var(--color-primary);
  border-color: var(--color-primary-border);
}

.ranking-chip.active {
  border-color: var(--color-primary-border);
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 600;
}

.action-chip {
  flex-shrink: 0;
  padding: 8px 16px;
  border-radius: var(--radius-full);
  border: 1px dashed var(--color-border);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
  white-space: nowrap;
}

/* 列表区 */
.content-panel {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0 var(--space-6) var(--space-10);
  position: relative;
}

.book-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.book-item {
  position: relative;
  display: flex;
  gap: var(--space-4);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-xs);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.book-item:hover {
  border-color: var(--color-primary-border);
  box-shadow: var(--shadow-sm);
}

.book-cover {
  flex-shrink: 0;
  width: 92px;
  height: 126px;
  border-radius: var(--radius-md);
  object-fit: cover;
  background: var(--color-bg-sunken);
}

.book-cover.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-2xl);
  color: var(--color-text-tertiary);
}

.book-info {
  flex: 1;
  min-width: 0;
  padding-right: 76px;
}

.book-name {
  font-size: var(--text-lg);
  font-weight: 700;
  line-height: var(--leading-tight);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.book-author {
  margin-top: 4px;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.book-tags {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  padding: 3px 9px;
  border-radius: var(--radius-sm);
  background: var(--color-bg-sunken);
  color: var(--color-text-secondary);
  font-size: var(--text-xs);
}

.tag.kind {
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.tag.origin {
  background: rgba(74, 144, 217, 0.12);
  color: var(--color-accent);
}

.book-latest {
  margin-top: 8px;
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.book-latest span {
  color: var(--color-text-tertiary);
}

.book-intro {
  margin-top: 6px;
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
  color: var(--color-text-tertiary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.shelf-btn {
  position: absolute;
  top: var(--space-4);
  right: var(--space-4);
  padding: 6px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.shelf-btn:hover {
  color: var(--color-primary);
  border-color: var(--color-primary-border);
}

.loading-state,
.end-state,
.error-state,
.empty-state {
  text-align: center;
  padding: 20px 0;
  color: var(--color-text-tertiary);
  font-size: var(--text-sm);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
}

.error-state {
  color: var(--color-danger);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 640px) {
  .explore-header,
  .filter-summary,
  .filter-bar,
  .ranking-bar,
  .content-panel {
    padding-left: var(--space-4);
    padding-right: var(--space-4);
  }

  .book-cover {
    width: 78px;
    height: 106px;
  }

  .book-info {
    padding-right: 0;
  }

  .shelf-btn {
    position: static;
    align-self: flex-start;
    margin-left: auto;
  }
}
</style>
