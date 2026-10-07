import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { exploreBook, getExploreKinds } from '../api/explore'
import { useSourceStore } from './source'
import type { SearchBook, BookSource, ExploreKind } from '../types'
import {
  getInitialExploreCategoryUrl,
  parseExploreCategories,
  type ExploreCategory,
} from '../utils/exploreCategories'

export const useExploreStore = defineStore('explore', () => {
  const sourceStore = useSourceStore()

  const activeSourceUrl = ref<string>('')
  const activeCategoryUrl = ref<string>('')

  // 发现页控件（线路/类型/频道/平台/榜单），由后端执行书源 exploreUrl 脚本得到
  const kinds = ref<ExploreKind[]>([])
  const variables = ref<Record<string, string>>({})
  const kindsLoading = ref(false)

  const books = ref<SearchBook[]>([])
  const loading = ref(false)
  const page = ref(1)
  const hasMore = ref(true)
  const error = ref<string | null>(null)

  // 筛选出启用了 explore 的书源
  const exploreSources = computed(() => {
    return sourceStore.sources.filter((s: BookSource) => s.enabledExplore && s.exploreUrl)
  })

  // 当前选中的书源对象
  const currentSource = computed(() => {
    return sourceStore.sources.find((s: BookSource) => s.bookSourceUrl === activeSourceUrl.value)
  })

  // 可切换的下拉筛选控件（线路 / 类型 / 频道 / 平台）
  const filterKinds = computed(() =>
    kinds.value.filter((kind) => kind.kindType === 'select' && (kind.chars?.length ?? 0) > 0),
  )

  // 榜单 / 分类标签（可点击并加载对应书单）
  const rankingKinds = computed(() =>
    kinds.value.filter((kind) => {
      if (kind.kindType === 'button' || kind.kindType === 'select') return false
      return !!kind.url?.trim()
    }),
  )

  // 操作类按钮（更新配置 / 更新书源 / 书源设置等），仅作展示
  const buttonKinds = computed(() =>
    kinds.value.filter((kind) => kind.kindType === 'button'),
  )

  // 回退方案：书源脚本不可用时，使用静态 exploreUrl 解析出的分类
  const fallbackCategories = computed<ExploreCategory[]>(() =>
    parseExploreCategories(currentSource.value?.exploreUrl),
  )

  const activeFilterSummary = computed(() =>
    filterKinds.value
      .map((kind) => {
        const key = kind.paramKey || kind.title
        const value = variables.value[key] || kind.defaultValue || ''
        return { key, title: kind.title, value }
      })
      .filter((item) => !!item.value),
  )

  async function fetchKinds() {
    if (!activeSourceUrl.value) {
      kinds.value = []
      return
    }
    kindsLoading.value = true
    try {
      kinds.value = await getExploreKinds({
        bookSourceUrl: activeSourceUrl.value,
        variables: { ...variables.value },
      })
    } catch {
      kinds.value = []
    } finally {
      kindsLoading.value = false
    }
  }

  /**
   * 让变量与当前控件保持一致：
   * 保留用户已选项，补齐缺失项，并在选项已失效时回落到默认值。
   */
  function syncVariablesWithKinds() {
    const next: Record<string, string> = { ...variables.value }
    for (const kind of filterKinds.value) {
      if (!kind.paramKey) continue
      const chars = kind.chars ?? []
      const current = next[kind.paramKey]
      if (!current || (chars.length > 0 && !chars.includes(current))) {
        const fallback = kind.defaultValue && chars.includes(kind.defaultValue)
          ? kind.defaultValue
          : chars[0] ?? kind.defaultValue ?? ''
        if (fallback) next[kind.paramKey] = fallback
      }
    }
    variables.value = next
  }

  async function pickInitialCategory() {
    const firstRanking = rankingKinds.value.find((kind) => kind.url?.trim())?.url?.trim()
    const fallbackUrl = getInitialExploreCategoryUrl(fallbackCategories.value)
    const target = firstRanking || fallbackUrl || ''

    if (!target) {
      activeCategoryUrl.value = ''
      books.value = []
      hasMore.value = false
      return
    }
    activeCategoryUrl.value = target
    await resetAndFetch()
  }

  async function applySource(url: string) {
    activeSourceUrl.value = url
    activeCategoryUrl.value = ''
    books.value = []
    variables.value = {}

    await fetchKinds()
    syncVariablesWithKinds()
    await pickInitialCategory()
  }

  function ensureActiveSource() {
    if (exploreSources.value.length === 0) {
      activeSourceUrl.value = ''
      activeCategoryUrl.value = ''
      kinds.value = []
      books.value = []
      hasMore.value = false
      return
    }

    const activeSourceStillValid = exploreSources.value.some(
      (source) => source.bookSourceUrl === activeSourceUrl.value,
    )
    if (!activeSourceUrl.value || !activeSourceStillValid) {
      void applySource(exploreSources.value[0].bookSourceUrl)
    }
  }

  function setSource(url: string) {
    if (activeSourceUrl.value === url) return
    void applySource(url)
  }

  /** 切换筛选条件（线路/类型/频道/平台），榜单列表与书单会随之刷新 */
  async function setVariable(paramKey: string, value: string) {
    if (!paramKey) return
    variables.value = { ...variables.value, [paramKey]: value }
    await fetchKinds()
    syncVariablesWithKinds()
    await pickInitialCategory()
  }

  function setCategory(url: string) {
    const nextUrl = url.trim()
    if (!nextUrl) return
    if (activeCategoryUrl.value === nextUrl) return
    activeCategoryUrl.value = nextUrl
    void resetAndFetch()
  }

  async function resetAndFetch() {
    books.value = []
    page.value = 1
    hasMore.value = true
    error.value = null
    await fetchMore()
  }

  async function fetchMore() {
    if (loading.value || !hasMore.value || !activeSourceUrl.value || !activeCategoryUrl.value) return

    loading.value = true
    error.value = null
    try {
      const result = await exploreBook({
        bookSourceUrl: activeSourceUrl.value,
        ruleFindUrl: activeCategoryUrl.value,
        page: page.value,
        variables: { ...variables.value },
      })

      if (result && result.length > 0) {
        books.value.push(...result)
        page.value++
      } else {
        hasMore.value = false
      }
    } catch (err: unknown) {
      error.value = (err as Error).message || '加载失败'
      hasMore.value = false
    } finally {
      loading.value = false
    }
  }

  // 初始化时加载书源数据
  async function init() {
    if (sourceStore.sources.length === 0) {
      await sourceStore.fetchSources()
    }
    ensureActiveSource()
  }

  watch(exploreSources, () => {
    ensureActiveSource()
  })

  return {
    activeSourceUrl,
    activeCategoryUrl,
    books,
    loading,
    page,
    hasMore,
    error,
    kinds,
    kindsLoading,
    variables,
    exploreSources,
    currentSource,
    filterKinds,
    rankingKinds,
    buttonKinds,
    fallbackCategories,
    activeFilterSummary,
    init,
    setSource,
    setVariable,
    setCategory,
    fetchMore,
    resetAndFetch,
  }
})
