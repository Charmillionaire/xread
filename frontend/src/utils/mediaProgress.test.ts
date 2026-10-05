import { describe, expect, it } from 'vitest'
import {
  readMediaProgress,
  writeMediaProgress,
  clearMediaProgress,
  createMediaProgressThrottle,
} from './mediaProgress'

function memoryStorage() {
  const map = new Map<string, string>()
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    removeItem: (k: string) => void map.delete(k),
    size: () => map.size,
  }
}

describe('mediaProgress', () => {
  it('round-trips position for a chapter', () => {
    const s = memoryStorage()
    writeMediaProgress('book-1', 'ch-1', 42.5, 600, s)
    const p = readMediaProgress('book-1', 'ch-1', s)
    expect(p?.position).toBe(42.5)
    expect(p?.duration).toBe(600)
  })

  it('isolates different chapters', () => {
    const s = memoryStorage()
    writeMediaProgress('book-1', 'ch-1', 10, 100, s)
    expect(readMediaProgress('book-1', 'ch-2', s)).toBeNull()
  })

  it('ignores progress too close to the end', () => {
    const s = memoryStorage()
    writeMediaProgress('book-1', 'ch-1', 598, 600, s)
    expect(readMediaProgress('book-1', 'ch-1', s)).toBeNull()
  })

  it('ignores non-positive positions', () => {
    const s = memoryStorage()
    writeMediaProgress('book-1', 'ch-1', 0, 600, s)
    expect(readMediaProgress('book-1', 'ch-1', s)).toBeNull()
  })

  it('survives corrupt payloads', () => {
    const s = memoryStorage()
    s.setItem('reader-media-progress:book-1::ch-1', '{not json')
    expect(readMediaProgress('book-1', 'ch-1', s)).toBeNull()
  })

  it('clears stored progress', () => {
    const s = memoryStorage()
    writeMediaProgress('book-1', 'ch-1', 30, 600, s)
    clearMediaProgress('book-1', 'ch-1', s)
    expect(readMediaProgress('book-1', 'ch-1', s)).toBeNull()
  })

  it('throttles writes by interval', () => {
    const shouldWrite = createMediaProgressThrottle(5000)
    expect(shouldWrite(1000)).toBe(true)
    expect(shouldWrite(2000)).toBe(false)
    expect(shouldWrite(6500)).toBe(true)
  })
})
