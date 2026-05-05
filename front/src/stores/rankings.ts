import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type {
  RankingEntry,
  ModelRankingEntry,
  RankingsResponse,
  RankingsFilters,
  WebsiteSummary,
  ModelSummary,
  SubmissionDetail,
} from '@/types/api'
import { fetchRankings, fetchWebsiteDetail, fetchModelDetail } from '@/api/rankings'

export const useRankingsStore = defineStore('rankings', () => {
  // --- Mode ---
  const rankingType = ref<'website' | 'model'>('website')

  // --- Rankings (website mode) ---
  const rankings = ref<RankingEntry[]>([])

  // --- Rankings (model mode) ---
  const modelRankings = ref<ModelRankingEntry[]>([])

  // --- Common ---
  const total = ref(0)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const filters = ref<RankingsFilters>({
    search: '',
    style: '',
    limit: 50,
    offset: 0,
  })

  const hasMore = computed(() => {
    const current = rankingType.value === 'website' ? rankings.value.length : modelRankings.value.length
    return current < total.value
  })
  const activeStyle = computed(() => filters.value.style || null)

  // --- Website Detail ---
  const websiteDetail = ref<WebsiteSummary | null>(null)
  const websiteSubmissions = ref<SubmissionDetail[]>([])
  const websiteLoading = ref(false)
  const websiteError = ref<string | null>(null)

  // --- Model Detail ---
  const modelDetail = ref<ModelSummary | null>(null)
  const modelSubmissions = ref<SubmissionDetail[]>([])
  const modelLoading = ref(false)
  const modelError = ref<string | null>(null)

  // --- Actions ---
  function setRankingType(type: 'website' | 'model') {
    if (rankingType.value === type) return
    rankingType.value = type
    filters.value.offset = 0
    total.value = 0
    loadRankings(true)
  }

  async function loadRankings(reset = false) {
    if (loading.value) return

    loading.value = true
    error.value = null

    if (reset) {
      filters.value.offset = 0
    }

    try {
      const resp = await fetchRankings({
        ...filters.value,
        rankingType: rankingType.value,
      })

      if (rankingType.value === 'website') {
        const data = resp as RankingsResponse<RankingEntry>
        if (reset) {
          rankings.value = data.rankings as RankingEntry[]
        } else {
          rankings.value = [...rankings.value, ...(data.rankings as RankingEntry[])]
        }
      } else {
        const data = resp as RankingsResponse<ModelRankingEntry>
        if (reset) {
          modelRankings.value = data.rankings as ModelRankingEntry[]
        } else {
          modelRankings.value = [...modelRankings.value, ...(data.rankings as ModelRankingEntry[])]
        }
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

  async function loadModelDetail(id: string) {
    modelLoading.value = true
    modelError.value = null
    modelDetail.value = null
    modelSubmissions.value = []

    try {
      const resp = await fetchModelDetail(id)
      modelDetail.value = resp.model
      modelSubmissions.value = resp.submissions
    } catch (err) {
      modelError.value = err instanceof Error ? err.message : '加载模型详情失败'
    } finally {
      modelLoading.value = false
    }
  }

  return {
    rankingType,
    rankings,
    modelRankings,
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
    modelDetail,
    modelSubmissions,
    modelLoading,
    modelError,
    setRankingType,
    loadRankings,
    loadMore,
    loadWebsiteDetail,
    loadModelDetail,
  }
})
