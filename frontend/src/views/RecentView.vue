<template>
  <div class="recent-view">
    <div class="recent-content">
      <div class="recent-header">
        <h1 class="recent-title">
          最近阅读
          <span class="recent-count">({{ shelfStore.recentBooks.length }})</span>
        </h1>
        <button
          class="recent-clear-btn"
          :disabled="!shelfStore.recentBooks.length"
          @click="handleClearRecent"
        >
          一键清空
        </button>
      </div>

      <BookGrid
        :books="shelfStore.recentBooks"
        :loading="shelfStore.loading"
        empty-text="暂无最近阅读"
        :show-delete-action="true"
        @click="handleBookClick"
        @info="handleBookInfo"
        @delete="handleRecentDelete"
      />
    </div>

    <BookDetailModal
      v-model="showDetail"
      :book="selectedBook"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import BookDetailModal from '../components/BookDetailModal.vue'
import BookGrid from '../components/BookGrid.vue'
import { useBookshelfStore } from '../stores/bookshelf'
import { useReaderStore } from '../stores/reader'
import type { Book, SearchBook } from '../types'

const router = useRouter()
const shelfStore = useBookshelfStore()
const readerStore = useReaderStore()

const showDetail = ref(false)
const selectedBook = ref<Book | SearchBook | null>(null)
const openingBookUrl = ref('')

onMounted(async () => {
  await shelfStore.fetchBooks().catch(() => undefined)
  await shelfStore.refreshRecentBooks().catch(() => undefined)
})

async function handleBookClick(book: Book | SearchBook) {
  const currentBook = book as Book
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

function handleBookInfo(book: Book | SearchBook) {
  selectedBook.value = book
  showDetail.value = true
}

async function handleRecentDelete(book: Book | SearchBook) {
  await shelfStore.removeRecentBook(book as Book).catch(() => undefined)
}

async function handleClearRecent() {
  await shelfStore.clearAllRecentBooks().catch(() => undefined)
}
</script>

<style scoped>
.recent-view {
  min-height: calc(100dvh - var(--header-height) - var(--safe-area-top));
}

.recent-content {
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding: 0 var(--space-6);
}

.recent-header {
  padding: var(--space-6) 0 var(--space-3);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
}

.recent-title {
  font-size: var(--text-2xl);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.recent-count {
  font-size: var(--text-base);
  font-weight: 400;
  color: var(--color-text-tertiary);
}

.recent-clear-btn {
  padding: 10px 16px;
  border-radius: 999px;
  border: 1px solid var(--color-border-light);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  transition: all var(--duration-fast) var(--ease-out);
}

.recent-clear-btn:hover:not(:disabled) {
  border-color: rgba(225, 76, 76, 0.22);
  color: var(--color-danger);
}

.recent-clear-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
