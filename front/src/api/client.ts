export const API_BASE = import.meta.env.VITE_API_BASE_URL || '/api'

class ApiError extends Error {
  status: number
  constructor(message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

function getToken(): string | null {
  return localStorage.getItem('oauth_token') || sessionStorage.getItem('anon_token')
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  options?: { useAnonymousToken?: boolean },
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }

  const token = getToken()
  if (token) {
    if (options?.useAnonymousToken) {
      headers['X-Anonymous-Token'] = token
    } else {
      headers['Authorization'] = `Bearer ${token}`
    }
  }

  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!res.ok) {
    if (res.status === 401) {
      localStorage.removeItem('oauth_token')
      sessionStorage.removeItem('anon_token')
    }
    const text = await res.text().catch(() => 'Unknown error')
    throw new ApiError(text, res.status)
  }

  return res.json() as Promise<T>
}

export async function apiGet<T>(
  path: string,
  params?: Record<string, string>,
): Promise<T> {
  const query = params
    ? '?' + new URLSearchParams(params).toString()
    : ''
  return request<T>('GET', `${path}${query}`)
}

export async function apiPost<T>(
  path: string,
  body: unknown,
  options?: { useAnonymousToken?: boolean },
): Promise<T> {
  return request<T>('POST', path, body, options)
}

export { ApiError }
