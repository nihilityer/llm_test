import type { ApiStyle } from './api'

export type TestRunState = 'idle' | 'running' | 'completed' | 'cancelled' | 'error'

export type SubmitState = 'idle' | 'submitting' | 'submitted' | 'error'

export type TabId = 0 | 1 | 2

export type TabStatus = 'active' | 'completed' | 'pending'

export interface TabDef {
  id: TabId
  label: string
  status: TabStatus
}

export interface StreamCallbacks {
  onChunk: (text: string) => void
  onThinking: (text: string) => void
}

export interface ApiConfig {
  style: ApiStyle
  endpoint: string
  apiKey: string
  model: string
  useProxy: boolean
}

export interface TestProgress {
  current: number
  total: number
  currentCaseId: string | null
  currentCaseTitle: string | null
}

export type DimensionScores = Record<string, number>

export interface LLMResponse {
  content: string
  thinking?: string
  toolCalls?: Array<{ name: string; arguments: Record<string, unknown> }>
  usage?: {
    promptTokens: number
    completionTokens: number
    totalTokens: number
    thinkingTokens?: number
  }
}

export interface CallLLMOptions {
  prompt: string
  systemPrompt?: string
  maxTokens?: number
  signal?: AbortSignal
  parameters?: Record<string, unknown>
}
