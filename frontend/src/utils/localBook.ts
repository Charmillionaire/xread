import type { Book, SearchBook } from '../types'

type BookLike = Pick<Book | SearchBook, 'origin' | 'bookUrl'>

export function isLocalTxtBook(book?: BookLike | null) {
  if (!book) return false
  return book.origin?.trim() === 'local-txt' || book.bookUrl?.trim().startsWith('local-txt:')
}
