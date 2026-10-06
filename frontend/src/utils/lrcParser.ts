export interface LrcLine {
  time: number
  text: string
}

/**
 * 解析 LRC 格式歌词/字幕文本
 * 支持 [mm:ss.xx]、[mm:ss] 等格式，返回按时间排序的字幕行列表
 */
export function parseLrc(raw: string | null | undefined): LrcLine[] {
  if (!raw) return []
  const lines = raw.split(/\r?\n/)
  const result: LrcLine[] = []
  const timeRegex = /\[(\d{1,2}):(\d{1,2}(?:\.\d{1,3})?)\]/g

  for (const line of lines) {
    timeRegex.lastIndex = 0
    let match: RegExpExecArray | null
    const times: number[] = []
    while ((match = timeRegex.exec(line)) !== null) {
      const min = parseFloat(match[1])
      const sec = parseFloat(match[2])
      times.push(min * 60 + sec)
    }
    const text = line.replace(/\[\d{1,2}:\d{1,2}(?:\.\d{1,3})?\]/g, '').trim()
    // 过滤掉包含 http(s):// 的直链行
    if (text && !/^https?:\/\//i.test(text)) {
      for (const t of times) {
        result.push({ time: t, text })
      }
    }
  }

  return result.sort((a, b) => a.time - b.time)
}
