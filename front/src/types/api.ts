// ---------- Auth ----------

export interface AuthResponse {
  token: string
  user: UserInfo | null
}

export interface UserInfo {
  id: string
  login: string
  avatar_url: string | null
}

export interface AnonymousAuthRequest {
  turnstile_token: string
}

export interface GithubCallbackRequest {
  code: string
}

// ---------- Submission ----------

export interface SubmissionRequest {
  domain: string
  model: string
  test_suite_version: string
  api_style: string
  endpoint_hash: string
  total_score: number
  dimension_scores: Record<string, number>
  test_results: TestResultItem[]
}

export interface SubmissionResponse {
  id: string
  website_id: string
  website_name: string
  model_id: string
  model_name: string
}

export interface TestResultItem {
  test_case_id: string
  passed: boolean
  response_time_ms: number
  tokens_used: number
  output_preview: string
  details?: string
}

// ---------- Rankings ----------

export interface RankingEntry {
  rank: number
  website_id: string
  website_name: string
  domains: string[]
  avg_score: number
  submission_count: number
  max_score: number
  min_score: number
  last_tested_at: string
}

export interface ModelRankingEntry {
  rank: number
  model_id: string
  model_name: string
  aliases: string[]
  avg_score: number
  submission_count: number
  max_score: number
  min_score: number
  last_tested_at: string
  website_count: number
}

export interface RankingsResponse<T = RankingEntry> {
  rankings: T[]
  total: number
}

export interface RankingsFilters {
  rankingType?: 'website' | 'model'
  search?: string
  style?: string
  limit: number
  offset: number
}

// ---------- Website Detail ----------

export interface SubmissionDetail {
  id: string
  submitter_type: string
  submitter_name: string | null
  submitter_avatar: string | null
  model_id: string
  model_name: string
  total_score: number
  dimension_scores: Record<string, number>
  test_suite_version: string
  api_style: string
  created_at: string
}

export interface WebsiteDetailResponse {
  website: WebsiteSummary
  submissions: SubmissionDetail[]
}

export interface WebsiteSummary {
  id: string
  name: string
  domains: string[]
  avg_score: number
  submission_count: number
}

// ---------- Model Detail ----------

export interface ModelDetailResponse {
  model: ModelSummary
  submissions: SubmissionDetail[]
}

export interface ModelSummary {
  id: string
  name: string
  aliases: string[]
  avg_score: number
  submission_count: number
  website_count: number
}

// ---------- Shared ----------

export type ApiStyle = 'openai' | 'anthropic' | 'gemini'

export const API_STYLES: { value: ApiStyle; label: string }[] = [
  { value: 'openai', label: 'OpenAI 风格' },
  { value: 'anthropic', label: 'Anthropic 风格' },
  { value: 'gemini', label: 'Gemini 风格' },
]

// ---------- Error ----------

export class AppError extends Error {
  constructor(
    message: string,
    public code: 'NETWORK' | 'AUTH' | 'VALIDATION' | 'SUBMIT' | 'TEST_RUN' | 'TURNSTILE' | 'NOT_FOUND',
    public detail?: string,
  ) {
    super(message)
    this.name = 'AppError'
  }
}
