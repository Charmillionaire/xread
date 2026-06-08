import { describe, expect, it } from 'vitest'
import { isLocalTxtBook } from './localBook'

describe('isLocalTxtBook', () => {
  it('detects uploaded local txt books by origin or url', () => {
    expect(isLocalTxtBook({ origin: 'local-txt', bookUrl: 'anything' })).toBe(true)
    expect(isLocalTxtBook({ origin: 'remote', bookUrl: 'local-txt:abc' })).toBe(true)
    expect(isLocalTxtBook({ origin: 'remote', bookUrl: 'https://example.test/book' })).toBe(false)
    expect(isLocalTxtBook(null)).toBe(false)
  })
})
