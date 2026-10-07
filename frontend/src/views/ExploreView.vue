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

    <!-- 榜单 / 分类标签 -->
    <div v-if="store.rankingKinds.length" class="ranking-bar">
      <button
        v-for="kind in store.rankingKinds"
        :key="kind.url || kind.title"
        class="ranking-chip"
        :class="{ active: store.activeCategoryUrl === kind.url }"
        @click="onCategoryClick(kind)"
      >
        {{ kind.title }}
      </button>
    </div>

    <!-- 书籍列表：与书架一致的卡片网格 -->
    <div class="content-panel" ref="scrollContainer" @scroll="handleScroll">
      <BookGrid
        :books="store.books"
        :is-search="true"
        :loading="store.loading && store.books.length === 0"
        :empty-text="store.activeCategoryUrl ? '暂无数据' : '请选择书源或榜单'"
        @click="handleBookClick"
        @info="handleBookInfo"
        @addToShelf="handleAddToShelf"
      />

      <div class="end-state" v-if="!store.loading && !store.hasMore && store.books.length > 0">
        没有更多了
      </div>

      <div class="error-state" v-if="store.error">{{ store.error }}</div>
    </div>

    <BookDetailModal v-model="showDetail" :book="selectedBook" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useExploreStore } from '../stores/explore'
import { useReaderStore } from '../stores/reader'
import { saveBook } from '../api/bookshelf'
import { useAppStore } from '../stores/app'
import BookGrid from '../components/BookGrid.vue'
import BookDetailModal from '../components/BookDetailModal.vue'
import type { Book, SearchBook, ExploreKind } from '../types'

const store = useExploreStore()
const readerStore = useReaderStore()
const appStore = useAppStore()
const router = useRouter()

const scrollContainer = ref<HTMLElement>()
const openingBookUrl = ref('')
const showDetail = ref(false)
const selectedBook = ref<Book | SearchBook | null>(null)

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

function handleBookInfo(book: Book | SearchBook) {
  selectedBook.value = book
  showDetail.value = true
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

/* 榜单标签栏：标签过多时自动换行，不出现横向滚动条 */
.ranking-bar {
  padding: 0 var(--space-6) var(--space-3);
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
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

/* 列表区：与书架一致，隐藏滚动条 */
.content-panel {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0 var(--space-6) calc(104px + var(--space-6));
  position: relative;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.content-panel::-webkit-scrollbar {
  display: none;
}

.end-state,
.error-state {
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

@media (max-width: 640px) {
  .explore-header,
  .filter-bar,
  .ranking-bar,
  .content-panel {
    padding-left: var(--space-4);
    padding-right: var(--space-4);
  }
}
</style>
