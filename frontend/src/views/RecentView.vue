<template>
  <div class="recent-view">
    <div class="recent-content">
      <header class="stat-header">
        <div class="stat-header-text">
          <h1 class="stat-date">{{ todayDateText }}</h1>
          <p class="stat-sub">已持续累计每日阅读时长</p>
        </div>
        <div class="stat-header-actions">
          <button class="icon-btn" title="更多" @click.stop="showActions = !showActions">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <circle cx="5" cy="12" r="2" />
              <circle cx="12" cy="12" r="2" />
              <circle cx="19" cy="12" r="2" />
            </svg>
          </button>
          <div v-if="showActions" class="actions-menu">
            <button @click="openGoalEditor">设置阅读目标</button>
            <button class="danger" :disabled="!shelfStore.recentBooks.length" @click="handleClearRecent">
              清空最近阅读
            </button>
          </div>
        </div>
      </header>

      <section class="card goal-card">
        <div class="card-head">
          <span class="card-title">阅读目标</span>
          <button class="icon-btn small" title="编辑目标" @click="openGoalEditor">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 20h9" />
              <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
            </svg>
          </button>
        </div>
        <div class="goal-body">
          <div class="goal-avatar">{{ avatarText }}</div>
          <div class="goal-info">
            <div class="goal-line">
              <span class="goal-label">今日阅读</span>
              <strong class="goal-value">{{ todayText }}</strong>
            </div>
            <div class="goal-target">目标 {{ goalText }}</div>
            <div class="goal-bar">
              <i :style="{ width: goalPercent + '%' }" />
            </div>
          </div>
        </div>
        <div class="goal-foot">
          <span>累计 {{ totalText }}</span>
          <span>已读书籍 {{ bookCount }} 本</span>
        </div>
      </section>

      <section class="stat-grid">
        <div class="stat-cell">
          <div class="stat-cell-value">{{ todayText }}</div>
          <div class="stat-cell-label">今日</div>
        </div>
        <div class="stat-cell">
          <div class="stat-cell-value">{{ monthText }}</div>
          <div class="stat-cell-label">本月</div>
        </div>
        <div class="stat-cell">
          <div class="stat-cell-value">{{ totalText }}</div>
          <div class="stat-cell-label">累计</div>
        </div>
        <div class="stat-cell">
          <div class="stat-cell-value">{{ activeDaysText }}</div>
          <div class="stat-cell-label">活跃日</div>
        </div>
      </section>

      <section class="card">
        <div class="card-head">
          <span class="card-title">阅读热力图</span>
          <span class="card-meta">最近 {{ heatmapWeekCount }} 周</span>
        </div>
        <div class="heatmap-scroll">
          <div class="heatmap-inner">
            <div class="heatmap-months">
              <span v-for="(week, wi) in heatmapWeeks" :key="wi" class="heatmap-month-slot">
                <em v-if="week.monthLabel">{{ week.monthLabel }}</em>
              </span>
            </div>
            <div class="heatmap">
              <div v-for="(week, wi) in heatmapWeeks" :key="wi" class="heatmap-week">
                <span
                  v-for="day in week.days"
                  :key="day.date"
                  class="heatmap-cell"
                  :class="'level-' + day.level"
                  :title="day.title"
                />
              </div>
            </div>
          </div>
        </div>
        <div class="heatmap-legend">
          <span>少</span>
          <i class="heatmap-cell level-0" />
          <i class="heatmap-cell level-1" />
          <i class="heatmap-cell level-2" />
          <i class="heatmap-cell level-3" />
          <i class="heatmap-cell level-4" />
          <span>多</span>
        </div>
      </section>

      <section class="card">
        <div class="card-head">
          <span class="card-title">最近在读</span>
          <span class="card-meta">{{ recentList.length }} 本</span>
        </div>
        <ul v-if="recentList.length" class="recent-list">
          <li v-for="item in recentList" :key="item.key" class="recent-item">
            <div class="recent-main" @click="openRecent(item.book)">
              <div class="recent-name">{{ item.name }}</div>
              <div class="recent-sub">
                <span class="recent-chapter">{{ item.durChapterTitle || '未开始阅读' }}</span>
                <span class="dot">·</span>
                <span>{{ relativeTime(item.recentReadAt) }}</span>
                <span class="dot">·</span>
                <span class="recent-time">{{ formatDuration(item.seconds) }}</span>
              </div>
            </div>
            <div class="recent-btns">
              <button
                v-if="item.recentKind !== 'rss'"
                class="mini-btn"
                @click.stop="openDetail(item.book)"
              >
                详情
              </button>
              <button class="mini-btn danger" @click.stop="removeRecent(item.book)">删除</button>
            </div>
          </li>
        </ul>
        <p v-else class="empty-text">暂无最近阅读</p>
      </section>

      <section v-if="ranking.length" class="card">
        <div class="card-head">
          <span class="card-title">阅读时长排行</span>
        </div>
        <ul class="rank-list">
          <li v-for="(item, i) in ranking" :key="item.bookUrl" class="rank-item" @click="openRankBook(item)">
            <span class="rank-no" :class="{ 'top-3': i < 3 }">{{ i + 1 }}</span>
            <img v-if="coverOf(item.coverUrl)" class="rank-cover" :src="coverOf(item.coverUrl)" alt="" />
            <div v-else class="rank-cover placeholder">{{ (item.bookName || '?').charAt(0) }}</div>
            <div class="rank-info">
              <div class="rank-name">{{ item.bookName }}</div>
              <div class="rank-author">{{ item.author || '佚名' }}</div>
            </div>
            <span class="rank-time">{{ formatDuration(item.seconds) }}</span>
          </li>
        </ul>
      </section>

      <section v-if="dailyList.length" class="card">
        <div class="card-head">
          <span class="card-title">按日摘要</span>
        </div>
        <ul class="daily-list">
          <li v-for="item in dailyList" :key="item.date" class="daily-item">
            <div class="daily-date">
              <strong>{{ item.dateText }}</strong>
              <span class="daily-weekday">{{ item.weekday }}</span>
            </div>
            <span class="daily-time">{{ item.timeText }}</span>
          </li>
        </ul>
      </section>
    </div>

    <div v-if="showGoalEditor" class="goal-mask" @click.self="showGoalEditor = false">
      <div class="goal-dialog">
        <h3>每日阅读目标</h3>
        <div class="goal-options">
          <button
            v-for="opt in goalOptions"
            :key="opt.value"
            :class="{ active: draftGoal === opt.value }"
            @click="draftGoal = opt.value"
          >
            {{ opt.label }}
          </button>
        </div>
        <div class="goal-dialog-actions">
          <button class="ghost" @click="showGoalEditor = false">取消</button>
          <button class="primary" @click="saveGoal">保存</button>
        </div>
      </div>
    </div>

    <BookDetailModal v-model="showDetail" :book="selectedBook" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import BookDetailModal from '../components/BookDetailModal.vue'
import { getCoverUrl } from '../api/bookshelf'
import { useAppStore } from '../stores/app'
import { useBookshelfStore } from '../stores/bookshelf'
import { useReaderStore } from '../stores/reader'
import {
  formatDuration,
  getStatsSummary,
  setDailyGoal,
  type BookReadingRecord,
} from '../utils/readingStats'
import type { Book, SearchBook } from '../types'

const DAY_MS = 24 * 60 * 60 * 1000
const heatmapWeekCount = 16

interface DayCell {
  date: string
  level: number
  title: string
}

const router = useRouter()
const appStore = useAppStore()
const shelfStore = useBookshelfStore()
const readerStore = useReaderStore()

const showActions = ref(false)
const showGoalEditor = ref(false)
const showDetail = ref(false)
const selectedBook = ref<Book | SearchBook | null>(null)
const openingBookUrl = ref('')
const summary = ref(getStatsSummary())
const draftGoal = ref(summary.value.dailyGoalMinutes)

const goalOptions = [
  { label: '15分钟', value: 15 },
  { label: '30分钟', value: 30 },
  { label: '1小时', value: 60 },
  { label: '2小时', value: 120 },
  { label: '3小时', value: 180 },
  { label: '5小时', value: 300 },
]

function dateKeyOf(d: Date) {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function parseDateKey(key: string) {
  const [y, m, d] = key.split('-').map(Number)
  return new Date(y, (m || 1) - 1, d || 1)
}

const todayDateText = computed(() => {
  const now = new Date()
  const weekday = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'][now.getDay()]
  return `${now.getFullYear()}年${String(now.getMonth() + 1).padStart(2, '0')}月${String(now.getDate()).padStart(2, '0')}日 ${weekday}`
})

const avatarText = computed(() => appStore.userInfo?.username?.charAt(0)?.toUpperCase() || 'U')

const todayText = computed(() => formatDuration(summary.value.todaySeconds))
const monthText = computed(() => formatDuration(summary.value.monthSeconds))
const totalText = computed(() => formatDuration(summary.value.totalSeconds))
const activeDaysText = computed(() => `${summary.value.activeDays}天`)
const goalText = computed(() => formatDuration(summary.value.dailyGoalMinutes * 60))

const goalPercent = computed(() => {
  const target = summary.value.dailyGoalMinutes * 60
  if (target <= 0) return 0
  return Math.min(100, Math.round((summary.value.todaySeconds / target) * 100))
})

const bookCount = computed(() => Object.keys(summary.value.bookRecords).length)

function levelOf(seconds: number) {
  if (seconds <= 0) return 0
  if (seconds < 300) return 1
  if (seconds < 900) return 2
  if (seconds < 1800) return 3
  return 4
}

const heatmapWeeks = computed(() => {
  const records = summary.value.dailyRecords
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const endOfWeek = new Date(today.getTime() + (6 - today.getDay()) * DAY_MS)
  let cursor = endOfWeek.getTime() - (heatmapWeekCount * 7 - 1) * DAY_MS

  const weeks: Array<{ days: DayCell[]; monthLabel?: string }> = []
  let lastMonth = -1

  for (let w = 0; w < heatmapWeekCount; w += 1) {
    const days: DayCell[] = []
    let weekMonth = -1
    for (let d = 0; d < 7; d += 1) {
      const cur = new Date(cursor)
      if (d === 0) weekMonth = cur.getMonth()
      const key = dateKeyOf(cur)
      const seconds = records[key] || 0
      days.push({
        date: key,
        level: levelOf(seconds),
        title: `${key} · ${formatDuration(seconds)}`,
      })
      cursor += DAY_MS
    }
    const week: { days: DayCell[]; monthLabel?: string } = { days }
    if (weekMonth !== lastMonth) {
      week.monthLabel = `${weekMonth + 1}月`
      lastMonth = weekMonth
    }
    weeks.push(week)
  }
  return weeks
})

const recentList = computed(() =>
  shelfStore.recentBooks.slice(0, 8).map((book) => {
    const record = summary.value.bookRecords[book.bookUrl]
    return {
      key: `${book.origin || ''}::${book.bookUrl || ''}`,
      book: book as Book,
      name: book.name,
      durChapterTitle: book.durChapterTitle,
      recentReadAt: book.durChapterTime || record?.lastReadAt || 0,
      seconds: record?.seconds || 0,
      recentKind: book.recentKind,
    }
  }),
)

const ranking = computed(() =>
  Object.values(summary.value.bookRecords)
    .filter((item) => item.seconds > 0)
    .sort((a, b) => b.seconds - a.seconds)
    .slice(0, 5),
)

function coverOf(url?: string) {
  return url ? getCoverUrl(url) : ''
}

function relativeDayText(d: Date) {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const target = new Date(d)
  target.setHours(0, 0, 0, 0)
  const diff = Math.round((today.getTime() - target.getTime()) / DAY_MS)
  if (diff === 0) return '今天'
  if (diff === 1) return '昨天'
  if (diff === 2) return '前天'
  return ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'][d.getDay()]
}

const dailyList = computed(() =>
  Object.entries(summary.value.dailyRecords)
    .filter(([, seconds]) => seconds > 0)
    .sort((a, b) => (a[0] < b[0] ? 1 : -1))
    .slice(0, 30)
    .map(([date, seconds]) => {
      const d = parseDateKey(date)
      return {
        date,
        dateText: `${d.getFullYear()}年${String(d.getMonth() + 1).padStart(2, '0')}月${String(d.getDate()).padStart(2, '0')}日`,
        weekday: relativeDayText(d),
        timeText: formatDuration(seconds),
      }
    }),
)

function relativeTime(ts: number) {
  if (!ts) return '未记录'
  const diff = Date.now() - ts
  if (diff < 60 * 1000) return '刚刚'
  if (diff < 60 * 60 * 1000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < DAY_MS) return `${Math.floor(diff / 3600000)}小时前`
  if (diff < 2 * DAY_MS) return '昨天'
  if (diff < 30 * DAY_MS) return `${Math.floor(diff / DAY_MS)}天前`
  const d = new Date(ts)
  return `${d.getMonth() + 1}月${d.getDate()}日`
}

function openGoalEditor() {
  showActions.value = false
  draftGoal.value = summary.value.dailyGoalMinutes
  showGoalEditor.value = true
}

function saveGoal() {
  setDailyGoal(draftGoal.value)
  summary.value = getStatsSummary()
  showGoalEditor.value = false
}

function openDetail(book: Book) {
  selectedBook.value = book
  showDetail.value = true
}

async function openRecent(book: Book) {
  await handleBookClick(book)
}

async function handleBookClick(book: Book | SearchBook) {
  const currentBook = book as Book
  if (
    currentBook.recentKind === 'rss' &&
    currentBook.rssSourceUrl &&
    (currentBook.rssLink || currentBook.bookUrl)
  ) {
    await router.push({
      name: 'rss-article',
      query: {
        source: currentBook.rssSourceUrl,
        link: currentBook.rssLink || currentBook.bookUrl,
        title: currentBook.name || '',
        pubDate: currentBook.rssPubDate || '',
        origin: currentBook.author || '',
      },
    })
    return
  }
  if (!currentBook.origin || !currentBook.bookUrl) return
  if (openingBookUrl.value === currentBook.bookUrl) return

  openingBookUrl.value = currentBook.bookUrl
  const targetIndex = currentBook.durChapterIndex || 0

  try {
    await shelfStore.moveBookToFront(currentBook.bookUrl).catch(() => undefined)
    const loadBookTask = readerStore.loadBook(currentBook)
    await router.push('/reader')
    await loadBookTask
    await readerStore.loadChapter(targetIndex)
  } finally {
    openingBookUrl.value = ''
  }
}

async function openRankBook(item: BookReadingRecord) {
  const match =
    shelfStore.books.find((book) => book.bookUrl === item.bookUrl) ||
    shelfStore.recentBooks.find((book) => book.bookUrl === item.bookUrl)
  if (match) {
    await handleBookClick(match)
    return
  }
  appStore.showToast('该书不在书架中，无法直接打开', 'warning')
}

async function removeRecent(book: Book) {
  await shelfStore.removeRecentBook(book).catch(() => undefined)
  summary.value = getStatsSummary()
}

async function handleClearRecent() {
  showActions.value = false
  await shelfStore.clearAllRecentBooks().catch(() => undefined)
  appStore.showToast('已清空最近阅读记录', 'success')
}

onMounted(async () => {
  summary.value = getStatsSummary()
  await shelfStore.fetchBooks().catch(() => undefined)
  await shelfStore.refreshRecentBooks().catch(() => undefined)
})
</script>

<style scoped>
.recent-view {
  height: 100%;
  min-height: 0;
  overflow: hidden;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.recent-view::-webkit-scrollbar {
  display: none;
}

.recent-content {
  height: 100%;
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding: var(--space-6) var(--space-6) var(--space-12);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.recent-content::-webkit-scrollbar {
  display: none;
}

.stat-header {
  position: relative;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.stat-date {
  font-size: var(--text-2xl);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.stat-sub {
  margin-top: 4px;
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
}

.stat-header-actions {
  position: relative;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.icon-btn svg {
  width: 18px;
  height: 18px;
}

.icon-btn.small {
  width: 28px;
  height: 28px;
}

.icon-btn.small svg {
  width: 15px;
  height: 15px;
}

.icon-btn:hover {
  color: var(--color-primary);
  border-color: var(--color-primary-border);
}

.actions-menu {
  position: absolute;
  right: 0;
  top: 40px;
  z-index: var(--z-dropdown);
  min-width: 160px;
  padding: 6px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
}

.actions-menu button {
  padding: 9px 12px;
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  color: var(--color-text);
  text-align: left;
  background: transparent;
  border: none;
  cursor: pointer;
}

.actions-menu button:hover:not(:disabled) {
  background: var(--color-bg-hover);
}

.actions-menu button.danger {
  color: var(--color-danger);
}

.actions-menu button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.card {
  padding: var(--space-4);
  border-radius: 16px;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-xs);
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.card-title {
  font-size: var(--text-base);
  font-weight: 600;
}

.card-meta {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.goal-body {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.goal-avatar {
  flex-shrink: 0;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  color: var(--color-text-inverse);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-dark));
}

.goal-info {
  flex: 1;
  min-width: 0;
}

.goal-line {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-2);
}

.goal-label {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.goal-value {
  font-size: var(--text-lg);
  font-weight: 700;
}

.goal-target {
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.goal-bar {
  margin-top: 8px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-bg-sunken);
  overflow: hidden;
}

.goal-bar i {
  display: block;
  height: 100%;
  border-radius: var(--radius-full);
  background: linear-gradient(90deg, var(--color-primary-light), var(--color-primary));
  transition: width var(--duration-normal) var(--ease-out);
}

.goal-foot {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--color-divider);
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.stat-cell {
  padding: var(--space-4);
  border-radius: 16px;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-xs);
}

.stat-cell-value {
  font-size: var(--text-lg);
  font-weight: 700;
}

.stat-cell-label {
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.heatmap-scroll {
  overflow-x: auto;
  padding-bottom: 4px;
}

.heatmap-inner {
  min-width: 480px;
}

.heatmap-months {
  display: flex;
  margin-bottom: 6px;
}

.heatmap-month-slot {
  position: relative;
  flex: 1;
  height: 14px;
}

.heatmap-month-slot em {
  position: absolute;
  left: 0;
  font-style: normal;
  font-size: 10px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
}

.heatmap {
  display: flex;
  gap: 3px;
}

.heatmap-week {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
}

.heatmap-cell {
  display: block;
  width: 100%;
  aspect-ratio: 1;
  border-radius: 3px;
  background: var(--color-bg-sunken);
}

.heatmap-cell.level-1 {
  background: rgba(244, 63, 94, 0.22);
}

.heatmap-cell.level-2 {
  background: rgba(244, 63, 94, 0.42);
}

.heatmap-cell.level-3 {
  background: rgba(244, 63, 94, 0.65);
}

.heatmap-cell.level-4 {
  background: var(--color-primary);
}

.heatmap-legend {
  margin-top: var(--space-3);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  font-size: 10px;
  color: var(--color-text-tertiary);
}

.heatmap-legend .heatmap-cell {
  width: 11px;
  height: 11px;
  aspect-ratio: auto;
}

.recent-list,
.rank-list,
.daily-list {
  display: flex;
  flex-direction: column;
}

.recent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 10px 0;
  border-bottom: 1px solid var(--color-divider);
}

.recent-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.recent-main {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.recent-name {
  font-size: var(--text-base);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-sub {
  margin-top: 3px;
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
  overflow: hidden;
  white-space: nowrap;
}

.recent-chapter {
  max-width: 46%;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-time {
  color: var(--color-primary);
}

.recent-btns {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.mini-btn {
  padding: 5px 10px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.mini-btn.danger {
  color: var(--color-danger);
}

.mini-btn:hover {
  border-color: var(--color-primary-border);
}

.rank-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 9px 0;
  cursor: pointer;
  border-bottom: 1px solid var(--color-divider);
}

.rank-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.rank-no {
  width: 20px;
  flex-shrink: 0;
  text-align: center;
  font-weight: 700;
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
}

.rank-no.top-3 {
  color: var(--color-primary);
}

.rank-cover {
  width: 34px;
  height: 46px;
  border-radius: 5px;
  object-fit: cover;
  flex-shrink: 0;
  background: var(--color-bg-sunken);
}

.rank-cover.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
}

.rank-info {
  flex: 1;
  min-width: 0;
}

.rank-name {
  font-size: var(--text-sm);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rank-author {
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rank-time {
  flex-shrink: 0;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-primary);
}

.daily-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 0;
  border-bottom: 1px solid var(--color-divider);
}

.daily-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.daily-date {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.daily-date strong {
  font-size: var(--text-sm);
  font-weight: 600;
}

.daily-weekday {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}

.daily-time {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-primary);
}

.empty-text {
  padding: var(--space-6) 0;
  text-align: center;
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
}

.goal-mask {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-5);
}

.goal-dialog {
  width: 100%;
  max-width: 340px;
  padding: var(--space-5);
  border-radius: var(--radius-lg);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-lg);
}

.goal-dialog h3 {
  font-size: var(--text-lg);
  font-weight: 600;
  margin-bottom: var(--space-4);
}

.goal-options {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
}

.goal-options button {
  padding: 10px 0;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
}

.goal-options button.active {
  border-color: var(--color-primary-border);
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 600;
}

.goal-dialog-actions {
  margin-top: var(--space-5);
  display: flex;
  gap: var(--space-3);
}

.goal-dialog-actions button {
  flex: 1;
  padding: 10px 0;
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  cursor: pointer;
}

.goal-dialog-actions .ghost {
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}

.goal-dialog-actions .primary {
  border: none;
  background: var(--color-primary);
  color: var(--color-text-inverse);
  font-weight: 600;
}

@media (max-width: 640px) {
  .recent-content {
    padding: var(--space-4) var(--space-4) var(--space-10);
  }

  .stat-date {
    font-size: var(--text-xl);
  }

  .stat-cell {
    padding: var(--space-3);
  }

  .stat-cell-value {
    font-size: var(--text-base);
  }
}
</style>
