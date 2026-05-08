import http from './http'
import type { AiServerModelConfig, AiServerModelConfigResponse } from '../types'

export function getAiModelConfig() {
  return http
    .get<AiServerModelConfigResponse>('/getAiModelConfig')
    .then((r) => r.data)
}

export function saveAiModelConfig(config: AiServerModelConfig) {
  return http
    .post<AiServerModelConfigResponse>('/saveAiModelConfig', config)
    .then((r) => r.data)
}
