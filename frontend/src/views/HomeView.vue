<template>
  <div class="home-view">
    <!-- Search Mode -->
    <SearchResults
      v-if="shelfStore.isSearchMode"
      @back="shelfStore.clearSearch()"
    />

    <!-- Normal Bookshelf View -->
    <div v-else class="shelf-content">
      <!-- Shelf Header (Edit mode selection actions) -->
      <div v-if="isLoggedIn && shelfStore.editMode" class="shelf-header">
        <div class="shelf-actions">
          <button class="shelf-btn" type="button" title="全选" aria-label="全选" @click="shelfStore.selectAll()">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 11l3 3L22 4" />
              <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
            </svg>
            <span class="shelf-btn-label">全选</span>
          </button>
          <button class="shelf-btn" type="button" title="取消全选" aria-label="取消全选" @click="shelfStore.clearSelection()">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <path d="M8 12h8" />
            </svg>
            <span class="shelf-btn-label">取消全选</span>
          </button>
          <button
            class="shelf-btn active"
            type="button"
            title="完成"
            aria-label="完成"
            @click="toggleEditMode"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20 6 9 17l-5-5" />
            </svg>
            <span class="shelf-btn-label">完成</span>
          </button>
        </div>
      </div>

      <input
        ref="localBookFileInputRef"
        type="file"
        accept=".txt,.epub,text/plain,application/epub+zip"
        class="hidden-input"
        @change="handleLocalBookFileChange"
      />

      <!-- Group Tabs -->
      <div class="group-tabs">
        <div class="tabs-scroll">
          <button
            v-for="group in shelfStore.displayGroups"
            :key="group.groupId"
            class="tab-item"
            :class="{ active: shelfStore.activeGroupId === group.groupId }"
            @click="shelfStore.activeGroupId = group.groupId"
          >
            {{ group.groupName }}
          </button>
        </div>
      </div>

      <!-- Book Grid -->
      <div class="shelf-grid-wrapper">
        <BookGrid
          :books="shelfStore.filteredBooks"
          :edit-mode="shelfStore.editMode"
          :selected-urls="shelfStore.selectedBookUrls"
          :loading="shelfStore.loading"
          :sortable="!shelfStore.editMode && !shelfStore.loading && !shelfStore.sorting && !shelfStore.isSearchMode"
          empty-text="书架空空如也，搜索添加新书吧"
        @click="handleBookClick"
        @info="handleBookInfo"
        @delete="handleDeleteBook"
        @ai="handleBookAi"
        @select="shelfStore.toggleSelection($event.bookUrl)"
        @reorder="handleReorderBooks"
        />
      </div>

      <!-- Batch Toolbar -->
      <Transition name="slide-up">
        <div v-if="shelfStore.editMode && shelfStore.selectedBookUrls.size > 0" class="batch-toolbar">
          <div class="batch-info">
            已选中 <span>{{ shelfStore.selectedBookUrls.size }}</span> 本书
          </div>
          <div class="batch-actions">
            <button class="batch-btn" @click="handleBulkMove">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14" /></svg>
              移动分组
            </button>
            <button class="batch-btn danger" @click="handleBulkDelete">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a3 3 0 0 1-3-3H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /></svg>
              批量删除
            </button>
          </div>
        </div>
      </Transition>
    </div>

    <!-- Book Detail Modal -->
    <BookDetailModal
      v-model="showDetail"
      :book="selectedBook"
    />

    <!-- Group Select Modal -->
    <GroupSelectModal
      v-model="showGroupSelect"
      @select="handleSetGroup"
    />
    <GroupManagerModal v-model="showGroupManager" />

    <CacheLibraryModal v-model="showCacheManager" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useBookshelfStore } from '../stores/bookshelf'
import { useReaderStore } from '../stores/reader'
import { useAppStore } from '../stores/app'
import { uploadEpubBook, uploadTxtBook } from '../api/bookshelf'
import BookGrid from '../components/BookGrid.vue'
import BookDetailModal from '../components/BookDetailModal.vue'
import GroupSelectModal from '../components/bookshelf/GroupSelectModal.vue'
import GroupManagerModal from '../components/bookshelf/GroupManagerModal.vue'
import SearchResults from '../components/SearchResults.vue'
import CacheLibraryModal from '../components/CacheLibraryModal.vue'
import type { Book, SearchBook } from '../types'

const router = useRouter()
const shelfStore = useBookshelfStore()
const readerStore = useReaderStore()
const appStore = useAppStore()
const isLoggedIn = computed(() => appStore.isLoggedIn)

const showDetail = ref(false)
const showGroupSelect = ref(false)
const showGroupManager = ref(false)
const showCacheManager = ref(false)
const selectedBook = ref<Book | SearchBook | null>(null)
const openingBookUrl = ref('')
const localBookFileInputRef = ref<HTMLInputElement | null>(null)
const localBookUploading = ref(false)

onMounted(async () => {
  window.addEventListener('trigger-local-book-upload', triggerLocalBookUpload)
  await appStore.fetchUserInfo()
  await Promise.all([
    shelfStore.fetchBooks().catch(() => undefined),
    shelfStore.fetchGroups().catch(() => undefined),
  ])
  if (!appStore.isOnline) {
    const restored = await readerStore.restorePersistedSession()
    if (restored) {
      appStore.showToast('已恢复最近阅读的离线章节', 'success')
      router.replace('/reader')
    }
  }
})

function triggerLocalBookUpload() {
  if (localBookUploading.value) return
  localBookFileInputRef.value?.click()
}

async function handleLocalBookFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  const lowerName = file.name.toLowerCase()
  if (!lowerName.endsWith('.txt') && !lowerName.endsWith('.epub')) {
    appStore.showToast('只支持导入 .txt 或 .epub 文件', 'warning')
    return
  }

  localBookUploading.value = true
  try {
    const book = lowerName.endsWith('.epub')
      ? await uploadEpubBook(file)
      : await uploadTxtBook(file)
    await shelfStore.fetchBooks()
    appStore.showToast(`已导入《${book.name}》`, 'success')
    await handleBookClick(book)
  } catch (e: unknown) {
    appStore.showToast((e as Error).message || '本地书导入失败', 'error')
  } finally {
    localBookUploading.value = false
  }
}

async function handleBookClick(book: Book | SearchBook) {
  const b = book as Book
  if (openingBookUrl.value === b.bookUrl) return

  openingBookUrl.value = b.bookUrl
  const targetIndex = b.durChapterIndex || 0

  try {
    await shelfStore.moveBookToFront(b.bookUrl).catch(() => undefined)
    const loadBookTask = readerStore.loadBook(b)
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

function handleBookAi(book: Book | SearchBook) {
  const currentBook = book as Book
  router.push({
    name: 'ai-book',
    query: { bookUrl: currentBook.bookUrl },
  })
}

async function handleDeleteBook(book: Book | SearchBook) {
  const b = book as Book
  if (!confirm(`确定从书架删除 "${b.name}"？`)) return
  try {
    await shelfStore.removeBook(b)
    appStore.showToast(`已删除 "${b.name}"`, 'success')
  } catch (e: unknown) {
    appStore.showToast((e as Error).message, 'error')
  }
}

function toggleEditMode() {
  shelfStore.editMode = !shelfStore.editMode
  if (!shelfStore.editMode) {
    shelfStore.clearSelection()
  }
}

async function handleBulkDelete() {
  const count = shelfStore.selectedBookUrls.size
  if (!confirm(`确定删除选中的 ${count} 本书？`)) return
  try {
    await shelfStore.bulkDelete()
    appStore.showToast(`成功删除 ${count} 本书`, 'success')
  } catch (e: any) {
    appStore.showToast(e.message, 'error')
  }
}

async function handleBulkMove() {
  showGroupSelect.value = true
}

async function handleSetGroup(groupId: number) {
  const count = shelfStore.selectedBookUrls.size
  try {
    await shelfStore.bulkSetGroup(groupId)
    appStore.showToast(`成功将 ${count} 本书移至新分组`, 'success')
  } catch (e: any) {
    appStore.showToast(e.message, 'error')
  }
}
async function handleReorderBooks(payload: { draggedUrl: string; targetUrl: string }) {
  try {
    await shelfStore.reorderBooks(payload.draggedUrl, payload.targetUrl)
  } catch (e: any) {
    appStore.showToast(e.message || '排序失败', 'error')
  }
}
</script>

<style scoped>
.home-view {
  height: 100%;
  min-height: 0;
  overflow: hidden;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.home-view::-webkit-scrollbar {
  display: none;
}

.shelf-content {
  height: 100%;
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding: 0 var(--space-6);
  display: flex;
  flex-direction: column;
  min-height: 0;
}


.hidden-input {
  display: none;
}

.shelf-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-6) 0 var(--space-3);
}

.shelf-title {
  font-size: var(--text-2xl);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.book-count {
  font-size: var(--text-base);
  font-weight: 400;
  color: var(--color-text-tertiary);
}

.shelf-actions {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.shelf-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 90px;
  min-height: 38px;
  padding: 0;
  border-radius: 999px;
  font-size: 13.5px;
  font-weight: 500;
  color: #555862;
  background: rgba(255, 255, 255, 0.65);
  border: 1px solid rgba(255, 255, 255, 0.8);
  box-shadow:
    0 4px 12px rgba(0, 0, 0, 0.04),
    inset 0 1px 1.5px rgba(255, 255, 255, 0.95),
    inset 0 -1px 1px rgba(0, 0, 0, 0.03);
  backdrop-filter: blur(24px) saturate(190%);
  -webkit-backdrop-filter: blur(24px) saturate(190%);
  transition: all var(--duration-fast);
  white-space: nowrap;
}

[data-theme='dark'] .shelf-btn,
.theme-dark .shelf-btn {
  color: rgba(235, 237, 242, 0.75);
  background: rgba(28, 29, 34, 0.62);
  border-color: rgba(255, 255, 255, 0.14);
  box-shadow:
    0 6px 16px rgba(0, 0, 0, 0.35),
    inset 0 1px 1px rgba(255, 255, 255, 0.18),
    inset 0 -1px 1px rgba(0, 0, 0, 0.45);
}

.shelf-btn:disabled {
  opacity: 0.55;
  cursor: wait;
}

.shelf-btn svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.shelf-btn:hover {
  background: rgba(255, 255, 255, 0.85);
  color: var(--color-primary);
  border-color: rgba(255, 255, 255, 0.95);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.07);
}

[data-theme='dark'] .shelf-btn:hover,
.theme-dark .shelf-btn:hover {
  background: rgba(45, 47, 54, 0.85);
  color: var(--color-primary);
  border-color: rgba(255, 255, 255, 0.25);
}

.shelf-btn.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
  box-shadow: 0 4px 14px color-mix(in srgb, var(--color-primary) 35%, transparent);
}

.shelf-btn:active {
  transform: scale(0.96);
}

.shelf-btn .spinning {
  animation: shelf-spin 0.9s linear infinite;
}

@keyframes shelf-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.group-tabs {
  border-bottom: none;
  margin-bottom: var(--space-2);
}

.tabs-scroll {
  display: flex;
  justify-content: center;
  gap: 0;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.tabs-scroll::-webkit-scrollbar {
  display: none;
}

.tab-item {
  padding: var(--space-3) var(--space-5);
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  white-space: nowrap;
  position: relative;
  transition: color var(--duration-fast);
  border-bottom: none;
  margin-bottom: 0;
}

.tab-item:hover {
  color: var(--color-text-secondary);
}

.tab-item.active {
  color: var(--color-primary);
  border-bottom: none;
}

.shelf-grid-wrapper {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding-bottom: calc(104px + var(--space-6));
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.shelf-grid-wrapper::-webkit-scrollbar {
  display: none;
}

.batch-toolbar {
  position: fixed;
  bottom: calc(104px + var(--space-4));
  left: 50%;
  transform: translateX(-50%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-xl);
  border-radius: var(--radius-2xl);
  padding: var(--space-3) var(--space-6);
  display: flex;
  align-items: center;
  gap: var(--space-8);
  z-index: calc(var(--z-sticky) + 5);
  backdrop-filter: blur(12px);
}

.batch-info {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}

.batch-info span {
  font-weight: 700;
  color: var(--color-primary);
  margin: 0 4px;
}

.batch-actions {
  display: flex;
  gap: var(--space-2);
}

.batch-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-lg);
  font-size: var(--text-sm);
  font-weight: 600;
  background: var(--color-bg-hover);
  color: var(--color-text-secondary);
  transition: all var(--duration-fast);
}

.batch-btn:hover {
  background: var(--color-bg-sunken);
  color: var(--color-text);
}

.batch-btn.danger {
  color: var(--color-danger);
}

.batch-btn svg {
  width: 16px;
  height: 16px;
}

/* slide-up transition */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all var(--duration-normal) var(--ease-out);
}
.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px);
}

@media (max-width: 640px) {
  .shelf-content {
    padding: 0 var(--space-4);
  }

  .shelf-header {
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-5) 0 var(--space-3);
  }

  .shelf-title {
    flex: 0 0 auto;
    font-size: var(--text-2xl);
  }

  .shelf-actions {
    flex: 1 1 auto;
    justify-content: flex-end;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .shelf-actions::-webkit-scrollbar {
    display: none;
  }

  .shelf-btn {
    width: 42px;
    min-width: 42px;
    height: 42px;
    min-height: 42px;
    padding: 0;
    border-radius: var(--radius-lg);
  }

  .shelf-btn svg {
    width: 19px;
    height: 19px;
  }

  .shelf-btn-label {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .batch-toolbar {
    width: calc(100% - var(--space-8));
    bottom: calc(104px + var(--space-3));
    gap: var(--space-4);
    justify-content: space-between;
  }
  .batch-info { display: none; }
}
</style>
