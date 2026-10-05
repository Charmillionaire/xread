import { describe, expect, it } from 'vitest'
import { parseMediaContent, buildMediaProxyUrl } from './mediaContent'

describe('parseMediaContent', () => {
  it('detects audio direct links', () => {
    const r = parseMediaContent('https://v5-novelapp.fqnovelvod.com/abc/?mime_type=audio_mpeg&qs=13')
    expect(r?.kind).toBe('audio')
  })

  it('detects video direct links', () => {
    const r = parseMediaContent('http://dj.qingtian618.com/video/cached/123/720p?key=abc&url=https%3A%2F%2Fx')
    expect(r?.kind).toBe('video')
  })

  it('strips the drama banner prefix', () => {
    const r = parseMediaContent('【右上角刷新】开启播放(下一集请切换下一章刷新)\n播放直链：\nhttp://dj.qingtian618.com/video/cached/1/720p?key=a')
    expect(r?.kind).toBe('video')
    expect(r?.url).toBe('http://dj.qingtian618.com/video/cached/1/720p?key=a')
  })

  it('returns null for plain text', () => {
    expect(parseMediaContent('第一章 正文内容，没有任何链接')).toBeNull()
  })
})

describe('buildMediaProxyUrl', () => {
  it('encodes the target url and source', () => {
    const u = buildMediaProxyUrl('http://x/v?a=1&b=2', '光遇聚合', 'tok', 'http://h')
    expect(u.startsWith('http://h/reader3/bookSourceProxy?')).toBe(true)
    expect(u).toContain('bookSourceUrl=')
    expect(u).toContain('accessToken=tok')
    expect(decodeURIComponent(u)).toContain('url=http://x/v?a=1&b=2')
  })
})
