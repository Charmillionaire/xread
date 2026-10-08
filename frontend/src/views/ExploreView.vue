<template>
  <div class="explore-view">
    <!-- 顶部区域与书籍列表同处一个滚动容器，下滑时整体向上滚走 -->
    <div class="content-panel" ref="scrollContainer" @scroll="handleScroll">
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

      <!-- 榜单 / 分类标签（默认最多2行，超出可展开） -->
      <div v-if="store.rankingKinds.length" class="ranking-wrapper">
        <div
          ref="rankingBar"
          class="ranking-bar"
          :class="{ 'is-collapsed': isRankingCollapsed }"
          :style="isRankingCollapsed && rankingCollapsedMax > 0 ? { maxHeight: rankingCollapsedMax + 'px' } : undefined"
        >
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
        <button
          v-if="rankingHasOverflow"
          type="button"
          class="ranking-toggle-btn"
          :title="isRankingExpanded ? '收起标签' : '展开更多标签'"
          @click="toggleRanking()"
        >
          <span>{{ isRankingExpanded ? '收起' : '展开' }}</span>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            class="toggle-icon"
            :class="{ 'is-expanded': isRankingExpanded }"
          >
            <path d="m6 9 6 6 6-6" />
          </svg>
        </button>
      </div>

      <!-- 书籍列表：与书架一致的卡片网格 -->
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
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
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

// 榜单标签：默认只显示两行，超出部分折叠，可用箭头展开
const rankingBar = ref<HTMLElement>()
const isRankingExpanded = ref(false)
const isRankingCollapsed = computed(() => !isRankingExpanded.value)
const rankingCollapsedMax = ref(0)
const rankingNaturalHeight = ref(0)
const rankingHasOverflow = computed(
  () => rankingNaturalHeight.value > rankingCollapsedMax.value + 1,
)

/**
 * 测量标签行高，得出「前两行」的高度作为折叠高度。
 * 标签的换行位置只由宽度决定，因此即使处于折叠态，各行 offsetTop 仍是完整布局值。
 */
function measureRanking() {
  const el = rankingBar.value
  if (!el) return
  const previousMaxHeight = el.style.maxHeight
  // 测量时临时取消高度限制，避免任何意外干扰行位置
  el.style.maxHeight = 'none'

  const chips = Array.from(el.querySelectorAll<HTMLElement>('.ranking-chip'))
  if (chips.length === 0) {
    rankingCollapsedMax.value = 0
    rankingNaturalHeight.value = 0
    el.style.maxHeight = previousMaxHeight
    return
  }

  // 按行归组：key 为行顶端，value 为该行最低底端
  const rows = new Map<number, number>()
  for (const chip of chips) {
    const top = chip.offsetTop
    const bottom = top + chip.offsetHeight
    rows.set(top, Math.max(rows.get(top) ?? top, bottom))
  }
  const tops = Array.from(rows.keys()).sort((a, b) => a - b)
  const firstTop = tops[0]!
  rankingNaturalHeight.value = Math.max(...rows.values()) - firstTop
  rankingCollapsedMax.value = tops.length <= 2
    ? rankingNaturalHeight.value
    : rows.get(tops[1]!)! - firstTop

  el.style.maxHeight = previousMaxHeight
}

function toggleRanking() {
  isRankingExpanded.value = !isRankingExpanded.value
}

onMounted(async () => {
  await store.init()
  await nextTick()
  measureRanking()
  window.addEventListener('resize', measureRanking)
  // 字体加载完成后标签尺寸可能变化，重新测一次。
  document.fonts?.ready.then(measureRanking).catch(() => undefined)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', measureRanking)
})

// 榜单/分类标签变化（切换书源、筛选条件后重新拉取）时需要重新测量
watch(
  () => store.rankingKinds.map((kind) => kind.title).join('|'),
  async () => {
    await nextTick()
    measureRanking()
  },
)

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
  background: transparent;
  overflow: hidden;
}

.explore-header {
  padding: var(--space-5) 0 var(--space-3);
  display: flex;
  justify-content: center;
  align-items: center;
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
  padding: 0 0 var(--space-3);
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: var(--space-2);
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: var(--radius-full);
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  transition: all var(--duration-fast) var(--ease-out);
}

.filter-chip:hover {
  background: var(--glass-bg-hover);
  border-color: rgba(255, 255, 255, 0.85);
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

/* 榜单标签栏：最多显示两行，超出部分折叠，可用箭头展开 */
.ranking-wrapper {
  padding: 0 0 var(--space-3);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
}

.ranking-bar {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  overflow: hidden;
  transition: max-height var(--duration-normal) var(--ease-out);
}

/* 折叠高度优先取 JS 实测值（内联 max-height）；这里只是未测量前的兜底。 */
.ranking-bar.is-collapsed {
  max-height: 76px;
}

/* 展开时不限制高度 */
.ranking-bar:not(.is-collapsed) {
  max-height: none;
}

.ranking-toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  color: var(--color-text-tertiary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.ranking-toggle-btn:hover {
  color: var(--color-primary);
  border-color: rgba(255, 255, 255, 0.85);
  background: var(--glass-bg-hover);
}

.toggle-icon {
  width: 14px;
  height: 14px;
  transition: transform var(--duration-fast) var(--ease-out);
}

.toggle-icon.is-expanded {
  transform: rotate(180deg);
}

.ranking-chip {
  flex-shrink: 0;
  padding: 8px 16px;
  border-radius: var(--radius-full);
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.ranking-chip:hover {
  color: var(--color-primary);
  background: var(--glass-bg-hover);
  border-color: rgba(255, 255, 255, 0.85);
}

.ranking-chip.active {
  border-color: var(--color-primary-border);
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 600;
}

/* 滚动区：顶部筛选区 + 书籍列表一起滚动，隐藏滚动条 */
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
  .content-panel {
    padding-left: var(--space-4);
    padding-right: var(--space-4);
  }
}
</style>
