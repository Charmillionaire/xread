import { afterEach, describe, expect, it, vi } from 'vitest'
import { requestOpenAISpeechAudio } from './openaiSpeech'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('openaiSpeech', () => {
  it('routes server configured speech through aiProxy without browser credentials', async () => {
    installLocalStorage()
    localStorage.setItem('accessToken', 'alice-token')
    const fetchMock = vi.fn(async (_url: RequestInfo | URL, _init?: RequestInit) => ({
      ok: true,
      blob: async () => new Blob(['audio'], { type: 'audio/mpeg' }),
      headers: new Headers(),
    }))
    vi.stubGlobal('fetch', fetchMock)

    const blob = await requestOpenAISpeechAudio({
      source: 'server',
      baseUrl: '',
      apiKey: 'browser-key',
      input: '你好',
      model: 'browser-model',
      voice: 'browser-voice',
      format: 'mp3',
      speed: 1,
    })

    expect(blob.type).toBe('audio/mpeg')
    expect(fetchMock).toHaveBeenCalledWith(
      '/reader3/aiProxy',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          Authorization: 'alice-token',
          'Content-Type': 'application/json',
        }),
      }),
    )
    const init = fetchMock.mock.calls[0]?.[1] as unknown as RequestInit
    expect(JSON.parse(String(init.body))).toMatchObject({
      useServerConfig: true,
      kind: 'speech',
      path: '/v1/audio/speech',
      body: {
        input: '你好',
        response_format: 'mp3',
        speed: 1,
      },
    })
    expect(String(init.body)).not.toContain('browser-key')
  })
})

function installLocalStorage() {
  const memory = new Map<string, string>()
  Object.defineProperty(globalThis, 'localStorage', {
    value: {
      getItem: (key: string) => memory.get(key) || null,
      setItem: (key: string, value: string) => memory.set(key, value),
      removeItem: (key: string) => memory.delete(key),
      clear: () => memory.clear(),
    },
    configurable: true,
  })
}
