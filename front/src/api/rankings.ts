import { apiGet } from './client'
import type { RankingsResponse, RankingsFilters, WebsiteDetailResponse, ModelDetailResponse, RankingEntry, ModelRankingEntry } from '@/types/api'

export async function fetchRankings(
  filters: RankingsFilters,
): Promise<RankingsResponse<RankingEntry | ModelRankingEntry>> {
  const params: Record<string, string> = {}
  if (filters.search) params.search = filters.search
  if (filters.style) params.style = filters.style
  if (filters.rankingType) params.ranking_type = filters.rankingType
  params.limit = String(filters.limit)
  params.offset = String(filters.offset)
  return apiGet<RankingsResponse<RankingEntry | ModelRankingEntry>>('/rankings', params)
}

export async function fetchWebsiteDetail(
  websiteId: string,
): Promise<WebsiteDetailResponse> {
  return apiGet<WebsiteDetailResponse>('/rankings', { website_id: websiteId })
}

export async function fetchModelDetail(
  modelId: string,
): Promise<ModelDetailResponse> {
  return apiGet<ModelDetailResponse>('/rankings', { model_id: modelId })
}
