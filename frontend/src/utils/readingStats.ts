export interface DayReadingRecord {
  date: string // YYYY-MM-DD
  seconds: number
}

export interface BookReadingRecord {
  bookUrl: string
  bookName: string
  author?: string
  coverUrl?: string
  origin?: string
  seconds: number
  lastReadAt: number
}

export interface ReadingStatsState {
  totalSeconds: number
  dailyGoalMinutes: number // 默认目标分钟数，例如 120 (2小时)
  dailyRecords: Record<string, number> // 'YYYY-MM-DD' -> seconds
  bookRecords: Record<string, BookReadingRecord> // bookUrl -> BookReadingRecord
}

const STATS_STORAGE_KEY = 'xread-reading-stats-v2'

function getTodayKey(d = new Date()): string {
  const year = d.getFullYear()
  const month = String(d.getMonth() + 1).padStart(2, '0')
  const date = String(d.getDate()).padStart(2, '0')
  return `${year}-${month}-${date}`
}

export function loadReadingStats(): ReadingStatsState {
  try {
    const raw = localStorage.getItem(STATS_STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      return {
        totalSeconds: Number(parsed.totalSeconds) || 0,
        dailyGoalMinutes: Number(parsed.dailyGoalMinutes) || 120,
        dailyRecords: parsed.dailyRecords || {},
        bookRecords: parsed.bookRecords || {},
      }
    }
  } catch {
    // fallback
  }

  // 尝试从旧版的 reader-stats 迁移一部分数据
  let legacyTotalSeconds = 0
  try {
    const legacy = localStorage.getItem('reader-stats')
    if (legacy) {
      const parsed = JSON.parse(legacy)
      legacyTotalSeconds = Number(parsed.totalSeconds) || 0
    }
  } catch {
    // ignore
  }

  return {
    totalSeconds: legacyTotalSeconds,
    dailyGoalMinutes: 120,
    dailyRecords: {},
    bookRecords: {},
  }
}

export function saveReadingStats(state: ReadingStatsState) {
  try {
    localStorage.setItem(STATS_STORAGE_KEY, JSON.stringify(state))
  } catch {
    // ignore
  }
}

export function recordReadingTime(
  seconds: number,
  bookInfo?: { bookUrl: string; bookName: string; author?: string; coverUrl?: string; origin?: string },
) {
  if (seconds <= 0) return
  const state = loadReadingStats()
  state.totalSeconds += seconds

  const today = getTodayKey()
  state.dailyRecords[today] = (state.dailyRecords[today] || 0) + seconds

  if (bookInfo?.bookUrl) {
    const existing = state.bookRecords[bookInfo.bookUrl] || {
      bookUrl: bookInfo.bookUrl,
      bookName: bookInfo.bookName,
      author: bookInfo.author,
      coverUrl: bookInfo.coverUrl,
      origin: bookInfo.origin,
      seconds: 0,
      lastReadAt: Date.now(),
    }
    existing.seconds += seconds
    existing.lastReadAt = Date.now()
    if (bookInfo.bookName) existing.bookName = bookInfo.bookName
    if (bookInfo.author) existing.author = bookInfo.author
    if (bookInfo.coverUrl) existing.coverUrl = bookInfo.coverUrl
    if (bookInfo.origin) existing.origin = bookInfo.origin
    state.bookRecords[bookInfo.bookUrl] = existing
  }

  saveReadingStats(state)
}

export function setDailyGoal(minutes: number) {
  const state = loadReadingStats()
  state.dailyGoalMinutes = Math.max(1, minutes)
  saveReadingStats(state)
}

export function formatDuration(seconds: number): string {
  if (!seconds || seconds <= 0) return '0秒'
  const s = Math.floor(seconds % 60)
  const m = Math.floor((seconds / 60) % 60)
  const h = Math.floor(seconds / 3600)

  if (h > 0) return `${h}小时${m}分钟`
  if (m > 0) return `${m}分钟${s}秒`
  return `${s}秒`
}

export function formatDurationShort(seconds: number): string {
  if (!seconds || seconds <= 0) return '0秒'
  const m = Math.floor(seconds / 60)
  const h = Math.floor(m / 60)
  const remM = m % 60
  if (h > 0) {
    return remM > 0 ? `${h}小时${remM}分` : `${h}小时`
  }
  if (m > 0) {
    const s = Math.floor(seconds % 60)
    return s > 0 ? `${m}分${s}秒` : `${m}分钟`
  }
  return `${Math.floor(seconds)}秒`
}

export function getStatsSummary() {
  const state = loadReadingStats()
  const today = getTodayKey()
  const todaySeconds = state.dailyRecords[today] || 0

  const now = new Date()
  const currentYear = now.getFullYear()
  const currentMonth = String(now.getMonth() + 1).padStart(2, '0')
  const monthPrefix = `${currentYear}-${currentMonth}`

  let monthSeconds = 0
  let activeDays = 0

  Object.entries(state.dailyRecords).forEach(([dateStr, sec]) => {
    if (sec > 0) {
      activeDays += 1
      if (dateStr.startsWith(monthPrefix)) {
        monthSeconds += sec
      }
    }
  })

  // 如果有总时间但 activeDays 为 0，给保底至少 1 天
  if (state.totalSeconds > 0 && activeDays === 0) {
    activeDays = 1
  }

  return {
    todaySeconds,
    monthSeconds,
    totalSeconds: state.totalSeconds,
    activeDays,
    dailyGoalMinutes: state.dailyGoalMinutes,
    dailyRecords: state.dailyRecords,
    bookRecords: state.bookRecords,
  }
}
