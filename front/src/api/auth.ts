import { apiPost, API_BASE } from './client'
import type { AuthResponse } from '@/types/api'

export async function exchangeGithubCode(code: string): Promise<AuthResponse> {
  return apiPost<AuthResponse>('/auth/github/callback', { code })
}

export async function getAnonymousToken(
  turnstileToken: string,
): Promise<AuthResponse> {
  return apiPost<AuthResponse>('/auth/anonymous', {
    turnstile_token: turnstileToken,
  })
}

export function getGithubLoginUrl(): string {
  return `${API_BASE}/auth/github/login`
}
