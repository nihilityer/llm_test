import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { RankingEntry, RankingsFilters, WebsiteSummary, SubmissionDetail } from '@/types/api'
import { fetchRankings, fetchWebsiteDetail } from '@/api/rankings'

export const useRankingsStore = defineStore('rankings', () => {
  // --- Rankings ---
  const rankings = ref<RankingEntry[]>([])
  const total = ref(0)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const filters = ref<RankingsFilters>({
    search: '',
    style: '',
    limit: 50,
    offset: 0,
  })

  const hasMore = computed(() => rankings.value.length < total.value)
  const activeStyle = computed(() => filters.value.style || null)

  // --- Website Detail ---
  const websiteDetail = ref<WebsiteSummary | null>(null)
  const websiteSubmissions = ref<SubmissionDetail[]>([])
  const websiteLoading = ref(false)
  const websiteError = ref<string | null>(null)

  // --- Actions ---
  async function loadRankings(reset = false) {
    if (loading.value) return

    loading.value = true
    error.value = null

    if (reset) {
      filters.value.offset = 0
    }

    try {
      const resp = await fetchRankings(filters.value)
      if (reset) {
        rankings.value = resp.rankings
      } else {
        rankings.value = [...rankings.value, ...resp.rankings]
      }
      total.value = resp.total
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载排行榜失败'
    } finally {
      loading.value = false
    }
  }

  async function loadMore() {
    if (!hasMore.value || loading.value) return
    filters.value.offset += filters.value.limit
    await loadRankings(false)
  }

  async function loadWebsiteDetail(id: string) {
    websiteLoading.value = true
    websiteError.value = null
    websiteDetail.value = null
    websiteSubmissions.value = []

    try {
      const resp = await fetchWebsiteDetail(id)
      websiteDetail.value = resp.website
      websiteSubmissions.value = resp.submissions
    } catch (err) {
      websiteError.value = err instanceof Error ? err.message : '加载网站详情失败'
    } finally {
      websiteLoading.value = false
    }
  }

  return {
    rankings,
    total,
    loading,
    error,
    filters,
    hasMore,
    activeStyle,
    websiteDetail,
    websiteSubmissions,
    websiteLoading,
    websiteError,
    loadRankings,
    loadMore,
    loadWebsiteDetail,
  }
})
