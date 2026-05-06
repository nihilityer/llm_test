import type { ApiConfig } from '@/types/scoring'
import { API_BASE } from '@/api/client'

const PROXY_ENDPOINT = `${API_BASE}/llm-proxy`

function createProxyFetch(signal?: AbortSignal): typeof fetch {
  return async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const url = typeof input === 'string'
      ? input
      : input instanceof URL
        ? input.toString()
        : input.url

    const proxyResponse = await fetch(PROXY_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url,
        method: init?.method || 'GET',
        headers: init?.headers ? Object.fromEntries(new Headers(init.headers).entries()) : {},
        body: typeof init?.body === 'string' ? init.body : undefined,
      }),
      signal,
    })

    if (!proxyResponse.ok) {
      const errorText = await proxyResponse.text().catch(() => '')
      throw new Error(`LLM proxy error (${proxyResponse.status}): ${errorText}`)
    }

    return new Response(proxyResponse.body, {
      status: proxyResponse.status,
      statusText: proxyResponse.statusText,
      headers: proxyResponse.headers,
    })
  }
}

async function listModelsOpenAI(config: ApiConfig): Promise<string[]> {
  const { default: OpenAI } = await import('openai')

  const client = new OpenAI({
    baseURL: config.endpoint.replace(/\/+$/, ''),
    apiKey: config.apiKey,
    dangerouslyAllowBrowser: true,
    ...(config.useProxy ? { fetch: createProxyFetch() } : {}),
  })

  const ids: string[] = []
  const page = await client.models.list()
  for await (const model of page) {
    ids.push(model.id)
  }
  return ids
}

async function listModelsAnthropic(config: ApiConfig): Promise<string[]> {
  const { default: Anthropic } = await import('@anthropic-ai/sdk')

  const client = new Anthropic({
    baseURL: config.endpoint.replace(/\/+$/, ''),
    apiKey: config.apiKey,
    dangerouslyAllowBrowser: true,
    ...(config.useProxy ? { fetch: createProxyFetch() } : {}),
  })

  const ids: string[] = []
  const page = await client.models.list()
  for await (const model of page) {
    ids.push(model.id)
  }
  return ids
}

export async function listModels(config: ApiConfig): Promise<string[]> {
  switch (config.style) {
    case 'openai':
      return listModelsOpenAI(config)
    case 'anthropic':
      return listModelsAnthropic(config)
    default:
      throw new Error(`Model listing not supported for style: ${config.style}`)
  }
}
