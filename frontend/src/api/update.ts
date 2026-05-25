import http from './http'
import type { VersionUpdateInfo } from '../types'

export function getVersionUpdate(force = false) {
  return http
    .get<VersionUpdateInfo>('/getVersionUpdate', {
      params: force ? { force: true } : undefined,
    })
    .then((r) => r.data)
}

export function dismissVersionUpdate(version: string) {
  return http
    .post<VersionUpdateInfo>('/dismissVersionUpdate', { version })
    .then((r) => r.data)
}
