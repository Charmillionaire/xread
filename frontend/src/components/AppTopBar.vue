<template>
  <header class="app-topbar">
    <div class="topbar-inner">
      <div class="topbar-left">
        <div class="logo" @click="goHome">
          <img class="logo-icon" src="/favicon-32x32.png" alt="Logo" />
          <span class="logo-text">Read</span>
        </div>
      </div>

      <div class="topbar-center">
        <form
          v-if="showGlobalSearch"
          class="search-box"
          :class="{ focused: searchFocused }"
          role="search"
          @submit.prevent="handleSearch"
        >
          <input
            v-model="searchValue"
            type="text"
            :placeholder="inputPlaceholder"
            @focus="searchFocused = true"
            @blur="searchFocused = false"
          />
          <button v-if="searchValue" class="search-clear" type="button" @click="clearSearch">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>

          <!-- 模式切换下拉胶囊 (纯文字，无图标) -->
          <div class="category-dropdown" ref="dropdownRef">
            <button
              class="category-trigger"
              type="button"
              @click.stop="toggleDropdown"
              :aria-expanded="showDropdown"
            >
              <span class="category-label">{{ currentCategoryLabel }}</span>
              <svg class="category-arrow" :class="{ open: showDropdown }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m6 9 6 6 6-6" />
              </svg>
            </button>
            <div v-if="showDropdown" class="category-menu">
              <button
                v-for="item in categories"
                :key="item.value"
                type="button"
                class="category-item"
                :class="{ active: currentCategory === item.value }"
                @click.stop="selectCategory(item.value)"
              >
                {{ item.label }}
              </button>
            </div>
          </div>

          <button
            class="search-submit"
            type="submit"
            title="搜索"
            aria-label="搜索"
            :disabled="!canSearch"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.3-4.3" />
            </svg>
          </button>
        </form>
      </div>

      <div class="topbar-right">
        <button class="topbar-btn" @click="toggleTheme" title="切换主题">
          <svg v-if="theme === 'light'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
          </svg>
        </button>

        <button v-if="!isLoggedIn" class="topbar-btn" @click="openSettings" title="设置">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
          <span v-if="hasVersionUpdateReminder" class="update-indicator" aria-hidden="true"></span>
        </button>

        <button v-else class="topbar-btn user-btn" @click="openSettings" title="用户">
          <div class="user-avatar">{{ userInfo?.username?.charAt(0)?.toUpperCase() || 'U' }}</div>
          <span v-if="hasVersionUpdateReminder" class="update-indicator" aria-hidden="true"></span>
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/app'
import { useBookshelfStore } from '../stores/bookshelf'
import { useExploreStore } from '../stores/explore'

import { onMounted, onUnmounted } from 'vue'

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()
const shelfStore = useBookshelfStore()
const exploreStore = useExploreStore()

const searchFocused = ref(false)
const searchValue = ref('')
const showDropdown = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

type CategoryType = 'novel' | 'skit' | 'manga' | 'audio'

const categories: { label: string; value: CategoryType }[] = [
  { label: '小说', value: 'novel' },
  { label: '短剧', value: 'skit' },
  { label: '漫画', value: 'manga' },
  { label: '有声', value: 'audio' },
]

const currentCategory = computed(() => shelfStore.searchCategory)
const currentCategoryLabel = computed(() => {
  const match = categories.find((c) => c.value === currentCategory.value)
  return match ? match.label : '小说'
})

const inputPlaceholder = computed(() => {
  return '搜索' + currentCategoryLabel.value + '...'
})

function toggleDropdown() {
  showDropdown.value = !showDropdown.value
}

function selectCategory(cat: CategoryType) {
  shelfStore.searchCategory = cat
  showDropdown.value = false
  // 如果当前已经有搜索词且在搜索结果中，直接触发重新搜索
  if (searchValue.value.trim() && shelfStore.isSearchMode) {
    handleSearch()
  }
}

function handleClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    showDropdown.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  window.removeEventListener('click', handleClickOutside)
})

const theme = computed(() => appStore.theme)
const isLoggedIn = computed(() => appStore.isLoggedIn)
const userInfo = computed(() => appStore.userInfo)
const hasVersionUpdateReminder = computed(() => appStore.hasVersionUpdateReminder)
const showGlobalSearch = computed(() => !route.path.startsWith('/rss') && route.path !== '/recent')
const canSearch = computed(() => searchValue.value.trim().length > 0)

function goHome() {
  shelfStore.clearSearch()
  router.replace('/')
}

function handleSearch() {
  const value = searchValue.value.trim()
  if (!value) return

  shelfStore.startSearch(value, {
    scope: 'source',
    sourceUrl: route.path === '/explore' ? exploreStore.activeSourceUrl : '',
  })

  if (route.path !== '/') {
    router.push('/')
  }
}

function clearSearch() {
  searchValue.value = ''
  shelfStore.clearSearch()
}

function toggleTheme() {
  appStore.toggleTheme()
}

function openSettings() {
  appStore.showSettingsDrawer = true
}
</script>

<style scoped>
.app-topbar {
  position: sticky;
  top: 0;
  z-index: var(--z-sticky);
  min-height: calc(var(--header-height) + var(--safe-area-top) + 10px);
  padding-top: var(--safe-area-top);
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border-light);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-sizing: border-box;
}

.topbar-inner {
  max-width: var(--content-max-width);
  margin: 0 auto;
  min-height: calc(var(--header-height) + 10px);
  display: flex;
  align-items: center;
  gap: var(--space-5);
  padding: 0 var(--space-6);
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex: 0 0 auto;
  min-width: 0;
}

.topbar-center {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1 1 auto;
  min-width: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
  flex-shrink: 0;
}

.logo-icon {
  object-fit: contain;
  border-radius: 6px;
  width: 28px;
  height: 28px;
  color: var(--color-primary);
}

.logo-text {
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.02em;
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-dark));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.search-box {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.03);
  border-radius: var(--radius-full);
  padding: var(--space-2) var(--space-4);
  max-width: 520px;
  width: 100%;
  flex: 1 1 auto;
  min-width: 220px;
  transition: all var(--duration-normal) var(--ease-out);
}

.search-box.focused {
  border-color: var(--color-primary);
  background: var(--color-bg-elevated);
  box-shadow: 0 0 0 3px var(--color-primary-bg), 0 4px 12px rgba(0, 0, 0, 0.05);
}

.search-icon {
  width: 18px;
  height: 18px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.search-box input {
  flex: 1;
  border: none;
  background: none;
  outline: none;
  font-size: var(--text-sm);
  color: var(--color-text);
  min-width: 0;
}

.search-box input::placeholder {
  color: var(--color-text-tertiary);
}

.search-clear {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  padding: 0;
}

.search-clear svg {
  width: 14px;
  height: 14px;
}

.search-submit {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
  color: var(--color-text-inverse);
  background: var(--color-primary);
  flex-shrink: 0;
  padding: 0;
  transition: transform var(--duration-fast), opacity var(--duration-fast), background var(--duration-fast);
}

.search-submit:hover:not(:disabled) {
  background: var(--color-primary-dark);
}

.search-submit:active:not(:disabled) {
  transform: scale(0.94);
}

.search-submit:disabled {
  color: var(--color-text-tertiary);
  background: transparent;
  opacity: 0.75;
}

.search-submit svg {
  width: 15px;
  height: 15px;
}

.topbar-right {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex: 0 0 auto;
  margin-left: auto;
}

.topbar-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  min-height: 42px;
  padding: 10px;
  border-radius: var(--radius-full);
  color: var(--color-text-secondary);
  transition: all var(--duration-fast) var(--ease-out);
}

.update-indicator {
  position: absolute;
  top: 7px;
  right: 7px;
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--color-warning);
  border: 2px solid var(--color-bg-elevated);
  box-shadow: 0 0 0 2px rgba(244, 63, 94, 0.14);
}

.topbar-btn:hover {
  background: var(--color-bg-elevated);
  color: var(--color-text);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.06);
}

.topbar-btn:active {
  background: var(--color-bg-active);
  transform: scale(0.97);
}

.topbar-btn svg {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.user-avatar {
  width: 30px;
  height: 30px;
  border-radius: var(--radius-full);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-sm);
  font-weight: 600;
}

@media (max-width: 640px) {
  .topbar-inner {
    padding: 0 var(--space-3);
    gap: var(--space-2);
  }

  .logo-text {
    display: none;
  }

  .search-box {
    max-width: none;
    min-width: 0;
    gap: 6px;
    padding: 7px 8px 7px var(--space-3);
  }

  .search-submit {
    width: 28px;
    height: 28px;
  }

  .topbar-left {
    gap: var(--space-3);
  }

  .topbar-btn {
    min-width: 38px;
    min-height: 38px;
    padding: 8px;
  }

  .topbar-btn svg {
    width: 18px;
    height: 18px;
  }

  .user-avatar {
    width: 28px;
    height: 28px;
    font-size: 12px;
  }
}

/* 搜索分类下拉 (无边框无背景胶囊，纯文字风格) */
.category-dropdown {
  position: relative;
  display: flex;
  align-items: center;
  margin: 0 4px;
  user-select: none;
}

.category-trigger {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 6px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm, 6px);
  color: var(--color-text, #333);
  font-size: var(--text-sm, 14px);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s) ease;
  white-space: nowrap;
}

.category-trigger:hover {
  color: var(--color-primary, #1890ff);
  background: var(--color-bg-hover, rgba(0, 0, 0, 0.04));
}

.category-arrow {
  width: 14px;
  height: 14px;
  transition: transform var(--duration-fast, 0.15s) ease;
  color: var(--color-text-secondary, #666);
}

.category-arrow.open {
  transform: rotate(180deg);
}

.category-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  background: var(--color-bg-elevated, #fff);
  border: 1px solid var(--color-border, rgba(0, 0, 0, 0.1));
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  border-radius: var(--radius-md, 8px);
  padding: 4px;
  min-width: 90px;
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.category-item {
  width: 100%;
  text-align: center;
  padding: 8px 14px;
  border: none;
  background: none;
  border-radius: var(--radius-sm, 6px);
  font-size: var(--text-sm, 14px);
  color: var(--color-text, #333);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s) ease;
}

.category-item:hover {
  background: var(--color-bg-hover, rgba(0, 0, 0, 0.05));
  color: var(--color-primary, #1890ff);
}

.category-item.active {
  background: var(--color-primary-bg, rgba(24, 144, 255, 0.1));
  color: var(--color-primary, #1890ff);
  font-weight: 600;
}
</style>
