import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import type {
  ApiConfig,
  DimensionScores,
  SubmitState,
  TabDef,
  TabId,
  TestProgress,
  TestRunState
} from '@/types/scoring'
import type { SubmissionResponse, TestResultItem } from '@/types/api'
import { runTestSuite } from '@/engine/runner'
import { calculateScores } from '@/engine/scoring'
import { extractDomain } from '@/utils/domain'
import { hashEndpoint } from '@/utils/crypto'
import { submitResult } from '@/api/submissions'
import { suiteV1 } from '@/test-suite/v1'
import { useAuthStore } from './auth'

export const useTestStore = defineStore('test', () => {
  // --- Config ---
  const apiConfig = ref<ApiConfig>({
    style: 'openai',
    endpoint: '',
    apiKey: '',
    model: '',
    useProxy: false,
  })

  const domain = ref('')

  watch(
    () => apiConfig.value.endpoint,
    (val) => {
      domain.value = extractDomain(val)
    },
  )

  // --- Execution ---
  const runState = ref<TestRunState>('idle')
  const progress = ref<TestProgress>({
    current: 0,
    total: 0,
    currentCaseId: null,
    currentCaseTitle: null,
  })
  const results = ref<TestResultItem[]>([])
  const scores = ref<DimensionScores | null>(null)
  const totalScore = ref<number | null>(null)
  const errorMessage = ref<string | null>(null)
  const abortController = ref<AbortController | null>(null)

  // --- Streaming ---
  const streamingOutputs = ref<Record<string, string>>({})
  const streamingThinking = ref<Record<string, string>>({})

  // --- Tabs ---
  const tabs = ref<TabDef[]>([
    { id: 0, label: 'API 配置', status: 'active' },
    { id: 1, label: '测试执行', status: 'pending' },
    { id: 2, label: '结果上传', status: 'pending' },
  ])

  function getTab(id: TabId): TabDef {
    return tabs.value.find((t) => t.id === id)!
  }

  function navigateToTab(id: TabId) {
    const tab = getTab(id)
    if (tab.status === 'pending') return
    tabs.value.forEach((t) => {
      t.status = t.id === id ? 'active' : t.status === 'active' ? 'completed' : t.status
    })
  }

  function setTabCompleted(id: TabId) {
    const tab = getTab(id)
    tab.status = 'completed'
  }

  function activateTab(id: TabId) {
    const tab = getTab(id)
    tab.status = 'active'
  }

  function resetTabs() {
    tabs.value = [
      { id: 0, label: 'API 配置', status: 'active' },
      { id: 1, label: '测试执行', status: 'pending' },
      { id: 2, label: '结果上传', status: 'pending' },
    ]
  }

  function clearStreamingOutputs() {
    streamingOutputs.value = {}
    streamingThinking.value = {}
  }

  function appendStreamingOutput(caseId: string, chunk: string) {
    if (!streamingOutputs.value[caseId]) {
      streamingOutputs.value[caseId] = ''
    }
    streamingOutputs.value[caseId] += chunk
  }

  function appendStreamingThinking(caseId: string, chunk: string) {
    if (!streamingThinking.value[caseId]) {
      streamingThinking.value[caseId] = ''
    }
    streamingThinking.value[caseId] += chunk
  }

  // --- Submit ---
  const submitState = ref<SubmitState>('idle')
  const submitResultData = ref<SubmissionResponse | null>(null)
  const submitError = ref<string | null>(null)

  // --- Getters ---
  const isConfigValid = computed(() => {
    const ep = apiConfig.value.endpoint.trim()
    const key = apiConfig.value.apiKey.trim()
    if (!ep || !key) return false
    return domain.value;
  })

  const canStart = computed(() => isConfigValid.value && runState.value === 'idle')

  const averageResponseTime = computed(() => {
    const valid = results.value.filter((r) => r.response_time_ms > 0)
    if (valid.length === 0) return 0
    return Math.round(
      valid.reduce((s, r) => s + r.response_time_ms, 0) / valid.length,
    )
  })

  // --- Actions ---
  function setApiConfig(patch: Partial<ApiConfig>) {
    Object.assign(apiConfig.value, patch)
  }

  async function startTest() {
    if (!isConfigValid.value) return

    runState.value = 'running'
    errorMessage.value = null
    results.value = []
    scores.value = null
    totalScore.value = null
    submitState.value = 'idle'
    submitResultData.value = null
    submitError.value = null

    setTabCompleted(0)
    activateTab(1)
    clearStreamingOutputs()

    const controller = new AbortController()
    abortController.value = controller

    try {
      const testResults = await runTestSuite(
        apiConfig.value,
        suiteV1,
        {
          onProgress(prog) {
            progress.value = { ...prog }
          },
          onChunk(caseId, chunk) {
            appendStreamingOutput(caseId, chunk)
          },
          onThinking(caseId, chunk) {
            appendStreamingThinking(caseId, chunk)
          },
          onCaseComplete(result) {
            results.value = [...results.value, result]
          },
        },
        controller.signal,
      )

      if (controller.signal.aborted) {
        runState.value = 'cancelled'
        return
      }

      const { dimensionScores, totalScore: total } = calculateScores(
        testResults,
        suiteV1,
      )
      scores.value = dimensionScores
      totalScore.value = total
      runState.value = 'completed'
      setTabCompleted(1)
      activateTab(2)
    } catch (err) {
      if (
        err instanceof DOMException &&
        (err.name === 'AbortError' || err.code === 20)
      ) {
        runState.value = 'cancelled'
      } else {
        runState.value = 'error'
        errorMessage.value = err instanceof Error ? err.message : '测试执行出错'
      }
    } finally {
      abortController.value = null
    }
  }

  function cancelTest() {
    abortController.value?.abort()
  }

  function resetTest() {
    runState.value = 'idle'
    progress.value = { current: 0, total: 0, currentCaseId: null, currentCaseTitle: null }
    results.value = []
    scores.value = null
    totalScore.value = null
    errorMessage.value = null
    submitState.value = 'idle'
    submitResultData.value = null
    submitError.value = null
    resetTabs()
    clearStreamingOutputs()
  }

  async function submit() {
    if (!scores.value || !domain.value) return

    submitState.value = 'submitting'
    submitError.value = null

    try {
      const endpointHash = await hashEndpoint(apiConfig.value.endpoint)

      const auth = useAuthStore()
      const useAnon = auth.tokenType === 'anonymous'

      submitResultData.value = await submitResult(
        {
          domain: domain.value,
          model: apiConfig.value.model.trim(),
          test_suite_version: suiteV1.version,
          api_style: apiConfig.value.style,
          endpoint_hash: endpointHash,
          total_score: totalScore.value!,
          dimension_scores: scores.value,
          test_results: results.value.map((r) => ({
            ...r,
            output_preview: r.output_preview.slice(0, 500),
          })),
        },
        useAnon ? { useAnonymousToken: true } : undefined,
      )
      submitState.value = 'submitted'
    } catch (err) {
      submitState.value = 'error'
      submitError.value = err instanceof Error ? err.message : '提交失败，请重试'
    }
  }

  function resetSubmit() {
    submitState.value = 'idle'
    submitResultData.value = null
    submitError.value = null
  }

  return {
    apiConfig,
    domain,
    runState,
    progress,
    results,
    scores,
    totalScore,
    errorMessage,
    submitState,
    submitResultData,
    submitError,
    isConfigValid,
    canStart,
    averageResponseTime,
    tabs,
    streamingOutputs,
    streamingThinking,
    setApiConfig,
    startTest,
    cancelTest,
    resetTest,
    submit,
    resetSubmit,
    navigateToTab,
    getTab,
  }
})
