import http from './http'
import type { ExploreKind, SearchBook } from '../types'

export interface ExploreBookParams {
  bookSourceUrl: string
  ruleFindUrl: string
  page?: number
  /** 发现页筛选变量（线路/类型/频道/平台等），透传给书源 JS 的 getVariable(k) */
  variables?: Record<string, string>
}

/**
 * 获取探索发现的书本列表
 * 对应后端 /reader3/exploreBook 接口
 */
export function exploreBook(params: ExploreBookParams) {
  return http.post<SearchBook[]>('/exploreBook', params).then((r) => r.data)
}

/**
 * 获取书源发现页的分类/筛选控件（线路、类型、频道、平台、榜单等）
 * 对应后端 /reader3/getExploreKinds 接口
 */
export function getExploreKinds(params: {
  bookSourceUrl: string
  variables?: Record<string, string>
}) {
  return http
    .post<ExploreKind[]>('/getExploreKinds', params)
    .then((r) => r.data || [])
}
