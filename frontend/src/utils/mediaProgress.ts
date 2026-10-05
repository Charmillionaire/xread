const STORAGE_PREFIX = 'reader-media-progress:'

export interface MediaProgress {
  position: number
  duration: number
  updatedAt: number
}

type StorageLike = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>

function storageKey(bookUrl: string, chapterUrl: string) {
  return `${STORAGE_PREFIX}${bookUrl}::${chapterUrl}`
}

function defaultStorage(): StorageLike | null {
  try {
    return typeof localStorage !== 'undefined' ? localStorage : null
  } catch {
    return null
  }
}

/**
 * 读取某章节的播放进度。接近结尾（剩余不足 5 秒）视为已播完，返回 null 从头开始，
 * 避免自动连播后再回来卡在结尾。
 */
export function readMediaProgress(
  bookUrl: string | undefined,
  chapterUrl: string | undefined,
  storage: StorageLike | null = defaultStorage(),
): MediaProgress | null {
  if (!bookUrl || !chapterUrl || !storage) return null
  const raw = storage.getItem(storageKey(bookUrl, chapterUrl))
  if (!raw) return null
  try {
    const parsed = JSON.parse(raw) as MediaProgress
    if (typeof parsed?.position !== 'number' || !Number.isFinite(parsed.position)) return null
    if (parsed.position < 1) return null
    if (parsed.duration > 0 && parsed.duration - parsed.position < 5) return null
    return parsed
  } catch {
    return null
  }
}

export function writeMediaProgress(
  bookUrl: string | undefined,
  chapterUrl: string | undefined,
  position: number,
  duration: number,
  storage: StorageLike | null = defaultStorage(),
): void {
  if (!bookUrl || !chapterUrl || !storage) return
  if (!Number.isFinite(position) || position < 1) return
  const payload: MediaProgress = {
    position,
    duration: Number.isFinite(duration) ? duration : 0,
    updatedAt: Date.now(),
  }
  try {
    storage.setItem(storageKey(bookUrl, chapterUrl), JSON.stringify(payload))
  } catch {
    /* 存储不可用时静默忽略 */
  }
}

export function clearMediaProgress(
  bookUrl: string | undefined,
  chapterUrl: string | undefined,
  storage: StorageLike | null = defaultStorage(),
): void {
  if (!bookUrl || !chapterUrl || !storage) return
  try {
    storage.removeItem(storageKey(bookUrl, chapterUrl))
  } catch {
    /* ignore */
  }
}

/** 进度落盘节流：默认 5 秒一次，避免 timeupdate 高频写存储。 */
export function createMediaProgressThrottle(intervalMs = 5000) {
  // 首次调用必须放行，否则进度会一直等到第二次 timeupdate 才落盘。
  let last: number | null = null
  return function shouldWrite(now = Date.now()) {
    if (last !== null && now - last < intervalMs) return false
    last = now
    return true
  }
}
