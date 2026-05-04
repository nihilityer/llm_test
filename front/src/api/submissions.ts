import { apiPost } from './client'
import type { SubmissionRequest, SubmissionResponse } from '@/types/api'

export async function submitResult(
  data: SubmissionRequest,
  options?: { useAnonymousToken?: boolean },
): Promise<SubmissionResponse> {
  return apiPost<SubmissionResponse>('/submissions', data, options)
}
