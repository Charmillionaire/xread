export interface MediaContent {
  kind: 'audio' | 'video'
  url: string
}

const URL_PATTERN = /https?:\/\/[^\s"'<>]+/g

// 注意：fqnovelvod 的音频直链路径里也带 "/video/tos/"，因此必须靠 mime_type 优先判定音频。
const AUDIO_HINT =
  /(mime_type=audio|audio_mpeg|audio_mpeg4|audio\/mpeg|\.mp3(\?|$)|\.m4a(\?|$)|\.aac(\?|$)|\.flac(\?|$)|\.wav(\?|$))/i
const VIDEO_HINT =
  /(mime_type=video|video_mp4|video\/mp4|\.mp4(\?|$)|\.m3u8(\?|$)|\.flv(\?|$)|\.mkv(\?|$)|\.ts(\?|$)|dj\.qingtian618\.com\/video\/|\/video\/cached\/)/i

/**
 * 书源的听书/短剧章节返回的是媒体直链（可能是纯直链，也可能带「播放直链：」这样的说明前缀）。
 * 这里把最后一条 http(s) 链接识别出来，并判断是音频还是视频。
 */
export function parseMediaContent(raw: string | null | undefined): MediaContent | null {
  const text = (raw || '').trim()
  if (!text) return null
  const matches = text.match(URL_PATTERN)
  if (!matches || !matches.length) return null
  const url = matches[matches.length - 1].replace(/[),.;]+$/, '')
  if (!/^https?:\/\//i.test(url)) return null

  const isAudio = AUDIO_HINT.test(url)
  const isVideo = VIDEO_HINT.test(url)
  if (isAudio && !isVideo) return { kind: 'audio', url }
  if (isVideo) return { kind: 'video', url }
  return null
}

/** 通过书源代理转发媒体地址，避免跨域与防盗链问题。 */
export function buildMediaProxyUrl(
  mediaUrl: string,
  bookSourceUrl: string | undefined,
  accessToken: string | null,
  origin?: string,
) {
  const base = origin || (typeof window !== 'undefined' ? window.location.origin : '')
  const params = new URLSearchParams()
  params.set('url', mediaUrl)
  if (bookSourceUrl) params.set('bookSourceUrl', bookSourceUrl)
  if (accessToken) params.set('accessToken', accessToken)
  return `${base}/reader3/bookSourceProxy?${params.toString()}`
}
