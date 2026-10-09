<template>
  <div class="rss-view">
    <section
      class="rss-main"
      :class="{
        'has-active-article': !!store.activeArticle,
      }"
    >
      <aside class="article-list-panel">
        <div class="panel-head source-head">
          <div class="source-select" v-if="store.enabledSources.length" ref="sourceSelectRef">
            <button
              type="button"
              class="scope-chip source-chip"
              :class="{ active: openSourceMenu }"
              @click.stop="toggleSourceMenu"
            >
              <span class="source-chip-text">{{ scopeLabel }}</span>
              <svg
                class="source-caret"
                :class="{ open: openSourceMenu }"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="m6 9 6 6 6-6" />
              </svg>
            </button>

            <Transition name="dropdown-fade">
              <div v-if="openSourceMenu" class="source-menu" @click.stop>
                <button
                  type="button"
                  class="source-menu-item"
                  :class="{ active: store.articleScope === 'all' }"
                  @click="selectAllSources"
                >
                  全部订阅
                </button>
                <button
                  v-for="source in store.enabledSources"
                  :key="source.sourceUrl"
                  type="button"
                  class="source-menu-item"
                  :class="{ active: store.articleScope === 'source' && store.activeSourceUrl === source.sourceUrl }"
                  @click="selectSource(source.sourceUrl)"
                >
                  {{ source.sourceName }}
                </button>
                <button
                  v-for="group in store.groupNames"
                  :key="`group-${group}`"
                  type="button"
                  class="source-menu-item"
                  :class="{ active: store.articleScope === 'group' && store.activeGroupName === group }"
                  @click="selectGroup(group)"
                >
                  {{ group }}
                </button>
              </div>
            </Transition>
          </div>
          <span v-else class="source-placeholder">暂无订阅源</span>
          <div class="head-actions">
            <button class="ghost-btn icon-btn" @click="goManage" aria-label="管理订阅源" title="管理订阅源">
              <span class="btn-icon">⚙</span>
            </button>
            <button
              class="ghost-btn icon-btn"
              :disabled="!store.enabledSources.length"
              @click="store.fetchArticles(true)"
              aria-label="刷新文章"
              title="刷新文章"
            >
              <span class="btn-icon">↻</span>
            </button>
          </div>
        </div>
        <div class="panel-scroll article-list-scroll">
          <div v-if="!store.sources.length" class="empty-box">还没有 RSS 源，先去管理页添加。</div>
          <div v-else-if="!store.activeSourceUrl" class="empty-box">请选择一个 RSS 源。</div>
          <template v-else>
            <div class="article-list-stack">
            <button
              v-for="article in store.articles"
              :key="`${article.variable || 'rss'}-${article.link}`"
              class="article-item"
              :class="{ active: store.activeArticle?.link === article.link && store.activeArticle?.variable === article.variable }"
              @click="handleOpenArticle(article)"
            >
              <div class="article-title">{{ article.title || '无标题' }}</div>
              <div class="article-meta-line">
                <span>{{ formatRelativeTime(article.pubDate) || '暂无时间' }}</span>
                <span v-if="article.origin" class="meta-sep">·</span>
                <span v-if="article.origin">{{ article.origin }}</span>
              </div>
              <div v-if="article.description" class="article-desc">{{ toPlainPreview(article.description) }}</div>
            </button>
            </div>
            <button v-if="store.hasMore && !store.loading" class="load-more-btn" @click="store.fetchArticles()">加载更多</button>
            <div v-if="store.loading" class="empty-box loading-box">文章加载中...</div>
          </template>
        </div>
      </aside>

      <article v-if="!isMobileLayout" class="article-content-panel" :class="{ collapsed: !store.activeArticle }">
        <template v-if="store.activeArticle">
          <div class="panel-head content-head">
            <div class="content-head-main">
              <h2>{{ store.activeArticle.title || '正文' }}</h2>
              <div class="content-head-meta">
                <div v-if="store.activeArticle.pubDate" class="content-head-time">
                  {{ formatRelativeTime(store.activeArticle.pubDate) }}
                </div>
                <span v-if="store.activeArticle.origin" class="content-head-dot">·</span>
                <div v-if="store.activeArticle.origin" class="content-head-origin">{{ store.activeArticle.origin }}</div>
                <a
                  v-if="store.activeArticle.link"
                  class="content-origin-link"
                  :href="store.activeArticle.link"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  打开原文
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M7 17 17 7M9 7h8v8" />
                  </svg>
                </a>
              </div>
            </div>
          </div>
          <div class="panel-scroll content-scroll">
            <div v-if="store.contentLoading" class="empty-box">正文加载中...</div>
            <div v-else-if="store.activeContent" class="content-html" v-html="store.activeContent"></div>
            <div v-else class="empty-box">这篇文章暂时没有可显示的正文。</div>
          </div>
        </template>
        <div v-else class="content-placeholder">
          <div class="content-placeholder-title">选择一篇文章开始阅读</div>
          <div class="content-placeholder-text">正文区会在你点开文章后展开显示。</div>
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useRssStore } from '../stores/rss'
import type { RssArticle } from '../types'

const router = useRouter()
const store = useRssStore()
const isMobileLayout = ref(false)
const openSourceMenu = ref(false)
const sourceSelectRef = ref<HTMLElement | null>(null)

const scopeLabel = computed(() => {
  if (store.articleScope === 'all') return '全部订阅'
  if (store.articleScope === 'group' && store.activeGroupName) return store.activeGroupName
  return store.activeSource?.sourceName || '全部订阅'
})

onMounted(async () => {
  syncViewportMode()
  window.addEventListener('resize', syncViewportMode)
  document.addEventListener('click', handleDocumentClick)
  await store.fetchSources()
  if (store.enabledSources.length && store.articles.length === 0) {
    await store.fetchArticles(true)
  }
  if (isMobileLayout.value) {
    store.activeArticle = null
    store.activeContent = ''
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', syncViewportMode)
  document.removeEventListener('click', handleDocumentClick)
})

function toggleSourceMenu() {
  openSourceMenu.value = !openSourceMenu.value
}

function handleDocumentClick(event: MouseEvent) {
  if (!openSourceMenu.value) return
  if (sourceSelectRef.value && !sourceSelectRef.value.contains(event.target as Node)) {
    openSourceMenu.value = false
  }
}

async function selectSource(url: string) {
  openSourceMenu.value = false
  await store.setSource(url)
}

async function selectAllSources() {
  openSourceMenu.value = false
  await store.setAllSources()
}

async function selectGroup(name: string) {
  openSourceMenu.value = false
  await store.setGroup(name)
}

function goManage() {
  router.push('/rss/manage')
}

function syncViewportMode() {
  isMobileLayout.value = window.innerWidth <= 960
}

function toPlainPreview(html: string) {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, ' ')
    .replace(/<style[\s\S]*?<\/style>/gi, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/\s+/g, ' ')
    .trim()
}

function formatRelativeTime(value?: string) {
  if (!value) return ''
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) return value

  const diff = Date.now() - timestamp
  if (diff < 0) return '刚刚'

  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  const month = 30 * day
  const year = 365 * day

  if (diff < minute) return '刚刚'
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`
  if (diff < month) return `${Math.floor(diff / day)} 天前`
  if (diff < year) return `${Math.floor(diff / month)} 个月前`
  return `${Math.floor(diff / year)} 年前`
}

async function handleOpenArticle(article: RssArticle & { variable?: string }) {
  if (isMobileLayout.value) {
    await router.push({
      name: 'rss-article',
      query: {
        source: article.variable || store.activeSourceUrl,
        link: article.link,
        title: article.title || '',
        pubDate: article.pubDate || '',
        origin: article.origin || '',
      },
    })
    return
  }
  await store.openArticle(article)
}
</script>

<style scoped>
.rss-view {
  height: calc(var(--app-height, 100dvh) - var(--header-height) - 104px - var(--safe-area-top) - var(--safe-area-bottom));
  min-height: calc(var(--app-height, 100dvh) - var(--header-height) - 104px - var(--safe-area-top) - var(--safe-area-bottom));
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px var(--space-6);
  max-width: var(--content-max-width);
  margin: 0 auto;
  width: 100%;
  overflow: hidden;
}

.article-list-panel,
.article-content-panel {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: 22px;
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

/* 左侧列表头部：订阅源下拉 + 操作按钮 */
.panel-head.source-head {
  margin-bottom: 8px;
  gap: 8px;
  flex-wrap: nowrap;
}

.head-actions {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex: 0 0 auto;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.btn-icon {
  font-size: 14px;
  line-height: 1;
}

.btn-text {
  line-height: 1;
}

.source-placeholder {
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.source-select {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-width: 0;
  flex: 1 1 auto;
  max-width: 100%;
}

/* 订阅源 / 全部订阅 / 分组：统一用一个通透毛玻璃下拉承载 */
.source-chip {
  width: 100%;
  max-width: 100%;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.source-chip-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-caret {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  color: var(--color-text-tertiary);
  transition: transform var(--duration-fast) var(--ease-out);
}

.source-caret.open {
  transform: rotate(180deg);
  color: var(--color-primary);
}

.source-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  z-index: var(--z-dropdown, 150);
  max-height: 260px;
  overflow-y: auto;
  padding: 6px;
  border-radius: 14px;
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow-hover), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.source-menu-item {
  width: 100%;
  padding: 7px 12px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--color-text);
  font-size: var(--text-sm);
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: all var(--duration-fast) var(--ease-out);
}

.source-menu-item:hover {
  background: var(--glass-bg-hover);
  color: var(--color-primary);
}

.source-menu-item.active {
  background: var(--color-primary-bg);
  color: var(--color-primary);
  font-weight: 600;
}

.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity var(--duration-fast) ease, transform var(--duration-fast) ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.scope-chip,
.load-more-btn,
.ghost-btn {
  padding: 5px 12px;
  border-radius: 999px;
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  color: var(--color-text-secondary);
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
  transition: all var(--duration-fast) var(--ease-out);
  white-space: nowrap;
  font-size: 12px;
  cursor: pointer;
}

.scope-chip.active {
  border-color: color-mix(in srgb, var(--color-primary) 26%, transparent);
  background: color-mix(in srgb, var(--color-primary) 10%, transparent);
  color: var(--color-primary);
}

.load-more-btn:hover,
.ghost-btn:hover:not(:disabled),
.scope-chip:hover {
  border-color: var(--color-primary-border);
  color: var(--color-primary);
}

.rss-main {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 360px minmax(0, 1fr);
  gap: 8px;
  overflow: hidden;
}

.article-list-panel,
.article-content-panel {
  min-height: 0;
  overflow: hidden;
  padding: 12px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  box-sizing: border-box;
}


.article-content-panel.collapsed {
  display: flex;
  align-items: center;
  justify-content: center;
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  flex-shrink: 0;
}

.panel-head h2 {
  margin: 0;
  font-size: 16px;
}

.panel-head span {
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.panel-scroll {
  min-height: 0;
  height: 100%;
  overflow: auto;
  -webkit-overflow-scrolling: touch;
  overscroll-behavior: contain;
}

.article-list-scroll {
  padding-right: 4px;
}

.article-list-stack {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.content-scroll {
  padding-right: 6px;
}

.content-head-main {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.content-head-main h2 {
  font-size: 22px;
  font-weight: 700;
  line-height: 1.4;
  letter-spacing: -0.01em;
  margin: 0;
}

.content-head-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.content-head-time,
.content-head-origin {
  font-size: 12.5px;
  color: var(--color-text-tertiary);
}

.content-head-dot {
  color: var(--color-text-tertiary);
  opacity: 0.6;
  font-size: 12px;
}

.content-origin-link {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  margin-left: auto;
  padding: 4px 11px;
  border-radius: var(--radius-full);
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow), var(--glass-inset-highlight);
  backdrop-filter: blur(18px) saturate(160%);
  -webkit-backdrop-filter: blur(18px) saturate(160%);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-decoration: none;
  transition: all var(--duration-fast) var(--ease-out);
}

.content-origin-link:hover {
  color: var(--color-primary);
  background: var(--glass-bg-hover);
}

.content-origin-link svg {
  width: 13px;
  height: 13px;
}

.content-placeholder {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  text-align: center;
  color: var(--color-text-tertiary);
}

.content-placeholder-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.content-placeholder-text {
  font-size: 14px;
}

.article-item {
  position: relative;
  display: block;
  width: 100%;
  text-align: left;
  padding: 14px 15px;
  border-radius: 16px;
  border: 1px solid var(--glass-border-subtle);
  background: var(--glass-bg);
  box-shadow: var(--glass-shadow);
  backdrop-filter: blur(14px) saturate(150%);
  -webkit-backdrop-filter: blur(14px) saturate(150%);
  transition: all var(--duration-fast) var(--ease-out);
  flex: 0 0 auto;
}

.article-item:hover {
  border-color: var(--color-primary-border);
  transform: translateY(-1px);
}

.article-item.active {
  border-color: color-mix(in srgb, var(--color-primary) 26%, transparent);
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
}

.article-title {
  font-weight: 600;
  line-height: 1.45;
}

.article-meta-line {
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-text-tertiary);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.meta-sep {
  opacity: 0.5;
}

.article-desc {
  margin-top: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.content-html {
  line-height: 1.9;
  color: var(--color-text);
  font-size: 16px;
  letter-spacing: 0.01em;
  max-width: 100%;
  overflow-wrap: anywhere;
  width: min(720px, 100%);
  margin: 0 auto;
  padding: 4px 2px 40px;
}

.content-html :deep(p) {
  margin: 0 0 1.25em;
}

.content-html :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  border-radius: 14px;
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.10);
  margin: 1.2em auto;
}

.content-html :deep(video),
.content-html :deep(iframe),
.content-html :deep(table),
.content-html :deep(pre),
.content-html :deep(code) {
  max-width: 100%;
}

.content-html :deep(pre) {
  overflow: auto;
}

.empty-box {
  min-height: 140px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--color-text-tertiary);
}

.loading-box {
  min-height: 72px;
}

@media (max-width: 960px) {
  .rss-view {
    height: calc(var(--app-height, 100dvh) - var(--header-height) - 104px - var(--safe-area-top) - var(--safe-area-bottom));
    min-height: calc(var(--app-height, 100dvh) - var(--header-height) - 104px - var(--safe-area-top) - var(--safe-area-bottom));
    padding: 6px;
    gap: 4px;
    overflow: hidden;
  }

  .rss-main {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .article-list-panel {
    flex: 1;
    min-height: 0;
    max-height: none;
  }

  .article-list-panel,
  .article-content-panel {
    padding: 8px;
  }

  .content-html {
    width: 100%;
  }
}

@media (max-width: 640px) {
  .rss-view {
    padding: 4px;
    gap: 3px;
  }

  .source-chip {
    font-size: 12.5px;
    padding: 5px 12px;
  }

  .rss-main {
    gap: 8px;
    flex: 1;
    min-height: 0;
  }

  .article-list-panel,
  .article-content-panel {
    padding: 8px;
    border-radius: 20px;
  }

  .panel-head {
    margin-bottom: 8px;
  }

  .article-item {
    padding: 12px;
    border-radius: 16px;
  }

  .article-list-stack {
    gap: 8px;
  }

  .article-title {
    font-size: 15px;
  }

  .article-desc {
    -webkit-line-clamp: 1;
  }

  .article-list-panel {
    flex: 1;
    min-height: 0;
    max-height: none;
  }

  .btn-text {
    display: none;
  }

  .icon-btn {
    padding-left: 8px;
    padding-right: 8px;
  }
}
</style>
