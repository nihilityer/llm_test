import { apiGet } from './client'
import type { RankingsResponse, RankingsFilters, WebsiteDetailResponse } from '@/types/api'

export async function fetchRankings(
  filters: RankingsFilters,
): Promise<RankingsResponse> {
  const params: Record<string, string> = {}
  if (filters.search) params.search = filters.search
  if (filters.style) params.style = filters.style
  params.limit = String(filters.limit)
  params.offset = String(filters.offset)
  return apiGet<RankingsResponse>('/rankings', params)
}

export async function fetchWebsiteDetail(
  websiteId: string,
): Promise<WebsiteDetailResponse> {
  return apiGet<WebsiteDetailResponse>('/rankings', { website_id: websiteId })
}
