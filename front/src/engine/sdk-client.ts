import type { ApiConfig, CallLLMOptions, LLMResponse, StreamCallbacks } from '@/types/scoring'
import { API_BASE } from '@/api/client'

const PROXY_ENDPOINT = `${API_BASE}/llm-proxy`

const SYSTEM_PROMPT = `【系统提示】你正在进行一项 AI 模型能力评测考试。

规则：
- 你必须且只能使用 submit_answer 工具提交你的最终答案
- 禁止调用 submit_answer 之外的任何其他工具或函数
- 请仔细阅读问题，做出准确判断后通过 submit_answer 提交

如果未使用 submit_answer 提交答案，或调用了其他工具，本测试将自动判定为失败。`

const EVAL_TOOL = {
  name: 'submit_answer',
  description: '提交最终答案的唯一工具。必须调用此工具来提交你的判断结果，否则测试失败。',
} as const

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

export async function callLLMStream(
  apiConfig: ApiConfig,
  options: CallLLMOptions,
  callbacks: StreamCallbacks,
): Promise<LLMResponse> {
  switch (apiConfig.style) {
    case 'openai':
      return callOpenAIStream(apiConfig, options, callbacks)
    case 'anthropic':
      return callAnthropicStream(apiConfig, options, callbacks)
    case 'gemini':
      return callGeminiStream(apiConfig, options, callbacks)
    default:
      throw new Error(`Unsupported API style: ${apiConfig.style}`)
  }
}

async function callOpenAIStream(
  config: ApiConfig,
  options: CallLLMOptions,
  callbacks: StreamCallbacks,
): Promise<LLMResponse> {
  const { default: OpenAI } = await import('openai')

  const client = new OpenAI({
    baseURL: config.endpoint.replace(/\/+$/, ''),
    apiKey: config.apiKey,
    dangerouslyAllowBrowser: true,
    ...(config.useProxy ? { fetch: createProxyFetch(options.signal) } : {}),
  })

  const stream = await client.chat.completions.create(
    {
      model: config.model || 'gpt-4o',
      messages: [
        { role: 'system', content: options.systemPrompt || SYSTEM_PROMPT },
        { role: 'user', content: options.prompt },
      ],
      ...(options.maxTokens !== undefined ? { max_tokens: options.maxTokens } : {}),
      stream: true,
      tools: options.parameters ? [{
        type: 'function' as const,
        function: {
          name: EVAL_TOOL.name,
          description: EVAL_TOOL.description,
          parameters: options.parameters,
        },
      }] : undefined,
      tool_choice: options.parameters
        ? { type: 'function' as const, function: { name: EVAL_TOOL.name } }
        : undefined,
    },
    { signal: options.signal },
  )

  let content = ''
  let thinking = ''
  const toolCallAccum: Map<number, { name: string; args: string }> = new Map()

  for await (const chunk of stream) {
    const delta = chunk.choices?.[0]?.delta
    if (!delta) continue

    const extra = delta as Record<string, unknown>
    if (typeof extra.reasoning_content === 'string') {
      thinking += extra.reasoning_content
      callbacks.onThinking(extra.reasoning_content)
    }

    if (delta.content) {
      content += delta.content
      callbacks.onChunk(delta.content)
    }

    if (delta.tool_calls) {
      for (const tc of delta.tool_calls) {
        const idx = tc.index
        if (!toolCallAccum.has(idx)) {
          toolCallAccum.set(idx, { name: '', args: '' })
        }
        const acc = toolCallAccum.get(idx)!
        if (tc.function?.name) acc.name = tc.function.name
        if (tc.function?.arguments) acc.args += tc.function.arguments
      }
    }
  }

  const toolCalls = Array.from(toolCallAccum.values())
    .filter(tc => tc.name)
    .map(tc => ({ name: tc.name, arguments: safeJsonParse(tc.args) }))

  return {
    content,
    thinking: thinking || undefined,
    toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
  }
}

async function callAnthropicStream(
  config: ApiConfig,
  options: CallLLMOptions,
  callbacks: StreamCallbacks,
): Promise<LLMResponse> {
  const { default: Anthropic } = await import('@anthropic-ai/sdk')

  const client = new Anthropic({
    baseURL: config.endpoint.replace(/\/+$/, ''),
    apiKey: config.apiKey,
    dangerouslyAllowBrowser: true,
    ...(config.useProxy ? { fetch: createProxyFetch(options.signal) } : {}),
  })

  const stream = await client.messages.create(
    {
      model: config.model || 'claude-sonnet-4',
      max_tokens: options.maxTokens ?? 4096,
      system: options.systemPrompt || SYSTEM_PROMPT,
      messages: [{ role: 'user', content: options.prompt }],
      stream: true,
      thinking: { type: 'enabled', budget_tokens: 1024 },
      // @ts-expect-error - input_schema conforms to JSON Schema spec; SDK types are overly strict
      tools: options.parameters ? [{
        name: EVAL_TOOL.name,
        description: EVAL_TOOL.description,
        input_schema: options.parameters,
      }] : undefined,
    },
    { signal: options.signal },
  )

  let content = ''
  let thinking = ''
  let inputTokens = 0
  let outputTokens = 0
  let thinkingTokens: number | undefined

  const toolCalls: Array<{ name: string; arguments: Record<string, unknown> }> = []
  const toolAccumById: Map<string, { name: string; inputJson: string }> = new Map()

  for await (const event of stream) {
    if (event.type === 'content_block_start') {
      if (event.content_block.type === 'tool_use') {
        const tu = event.content_block as unknown as { id: string; name: string }
        toolAccumById.set(tu.id, { name: tu.name, inputJson: '' })
      }
    }

    if (event.type === 'content_block_delta') {
      if (event.delta.type === 'text_delta') {
        content += event.delta.text
        callbacks.onChunk(event.delta.text)
      }
      if (event.delta.type === 'thinking_delta') {
        thinking += event.delta.thinking
        callbacks.onThinking(event.delta.thinking)
      }
      if (event.delta.type === 'input_json_delta') {
        const ijd = event.delta as unknown as { partial_json: string }
        // associate with the current tool_use block (latest in map)
        for (const acc of toolAccumById.values()) {
          if (acc.inputJson !== undefined) acc.inputJson += ijd.partial_json
        }
      }
    }

    if (event.type === 'message_delta') {
      outputTokens = event.usage.output_tokens
    }

    if (event.type === 'message_start') {
      inputTokens = event.message.usage.input_tokens
      const extra = event.message.usage as unknown as Record<string, unknown>
      if (typeof extra.cache_read_input_tokens === 'number' || typeof extra.cache_creation_input_tokens === 'number') {
        thinkingTokens = ((extra.cache_read_input_tokens as number) || 0) + ((extra.cache_creation_input_tokens as number) || 0)
      }
    }
  }

  for (const acc of toolAccumById.values()) {
    if (acc.name && acc.inputJson) {
      toolCalls.push({ name: acc.name, arguments: safeJsonParse(acc.inputJson) })
    }
  }

  return {
    content,
    thinking: thinking || undefined,
    toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
    usage: {
      promptTokens: inputTokens,
      completionTokens: outputTokens,
      totalTokens: inputTokens + outputTokens,
      thinkingTokens,
    },
  }
}

async function callGeminiProxyStream(
  config: ApiConfig,
  options: CallLLMOptions,
  callbacks: StreamCallbacks,
): Promise<LLMResponse> {
  const baseUrl = config.endpoint.replace(/\/+$/, '')
  const model = config.model || 'gemini-2.0-flash'

  const requestBody: Record<string, unknown> = {
    contents: [{ role: 'user', parts: [{ text: options.prompt }] }],
    generationConfig: {
      ...(options.maxTokens !== undefined ? { maxOutputTokens: options.maxTokens } : {}),
      thinkingConfig: { thinkingBudget: 1024 },
    },
  }
  if (options.systemPrompt) {
    requestBody.systemInstruction = { parts: [{ text: options.systemPrompt }] }
  }
  if (options.parameters) {
    requestBody.tools = [{
      functionDeclarations: [{
        name: EVAL_TOOL.name,
        description: EVAL_TOOL.description,
        parameters: options.parameters,
      }],
    }]
    requestBody.toolConfig = {
      functionCallingConfig: {
        mode: 'ANY',
        allowedFunctionNames: [EVAL_TOOL.name],
      },
    }
  }

  const proxyResponse = await fetch(PROXY_ENDPOINT, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      url: `${baseUrl}/models/${model}:streamGenerateContent?alt=sse`,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-goog-api-key': config.apiKey,
      },
      body: JSON.stringify(requestBody),
    }),
    signal: options.signal,
  })

  if (!proxyResponse.ok) {
    const errorText = await proxyResponse.text().catch(() => '')
    throw new Error(`Gemini proxy error (${proxyResponse.status}): ${errorText}`)
  }

  const reader = proxyResponse.body!.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let content = ''
  let thinking = ''
  let usage: LLMResponse['usage']
  const toolCalls: Array<{ name: string; arguments: Record<string, unknown> }> = []

  while (true) {
    const { done, value } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() || ''

    for (const line of lines) {
      const trimmed = line.trim()
      if (!trimmed.startsWith('data: ')) continue

      const jsonStr = trimmed.slice(6)
      if (jsonStr === '[DONE]') continue

      try {
        const data = JSON.parse(jsonStr)
        const candidates = data.candidates
        if (!candidates?.[0]) continue

        const parts = candidates[0].content?.parts
        if (parts) {
          for (const part of parts) {
            if (part.text) {
              if ((part as Record<string, unknown>).thought) {
                thinking += part.text
                callbacks.onThinking(part.text)
              } else {
                content += part.text
                callbacks.onChunk(part.text)
              }
            }
            const fc = (part as Record<string, unknown>).functionCall as { name: string; args: Record<string, unknown> } | undefined
            if (fc) {
              toolCalls.push({ name: fc.name, arguments: fc.args || {} })
            }
          }
        }

        // Capture usage metadata from the last chunk that has it
        const um = (data as Record<string, unknown>).usageMetadata as Record<string, number> | undefined
        if (um) {
          usage = {
            promptTokens: um.promptTokenCount ?? 0,
            completionTokens: um.candidatesTokenCount ?? 0,
            totalTokens: um.totalTokenCount ?? 0,
            thinkingTokens: um.thoughtsTokenCount as number | undefined,
          }
        }
      } catch {
        // Skip malformed JSON lines
      }
    }
  }

  return {
    content,
    thinking: thinking || undefined,
    toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
    usage,
  }
}

async function callGeminiStream(
  config: ApiConfig,
  options: CallLLMOptions,
  callbacks: StreamCallbacks,
): Promise<LLMResponse> {
  if (config.useProxy) {
    return callGeminiProxyStream(config, options, callbacks)
  }
  const { GoogleGenerativeAI } = await import('@google/generative-ai')

  const client = new GoogleGenerativeAI(config.apiKey)
  // @ts-expect-error - tools/parameters conform to Gemini API spec; SDK types are overly strict
  const model = client.getGenerativeModel({
    model: config.model || 'gemini-2.0-flash',
    systemInstruction: options.systemPrompt || SYSTEM_PROMPT,
    ...(options.parameters ? {
      tools: [{
        functionDeclarations: [{
          name: EVAL_TOOL.name,
          description: EVAL_TOOL.description,
          parameters: options.parameters,
        }],
      }],
      toolConfig: {
        functionCallingConfig: {
          mode: 'ANY',
          allowedFunctionNames: [EVAL_TOOL.name],
        },
      },
    } : {}),
  }, {
    baseUrl: config.endpoint
  })

  const result = await model.generateContentStream({
    contents: [{ role: 'user', parts: [{ text: options.prompt }] }],
    generationConfig: {
      ...(options.maxTokens !== undefined ? { maxOutputTokens: options.maxTokens } : {}),
      thinkingConfig: { thinkingBudget: 1024 },
    } as Record<string, unknown>,
    // @ts-expect-error - abortSignal is a newer API addition
    abortSignal: options.signal,
  })

  let content = ''
  let thinking = ''

  for await (const chunk of result.stream) {
    const parts = chunk.candidates?.[0]?.content?.parts
    if (parts) {
      for (const part of parts) {
        if (!part.text) continue
        if ((part as unknown as Record<string, unknown>).thought) {
          thinking += part.text
          callbacks.onThinking(part.text)
        } else {
          content += part.text
          callbacks.onChunk(part.text)
        }
      }
    }
    // Backward compat: also check via .text()
    const chunkText = chunk.text()
    if (chunkText && !parts?.length) {
      content += chunkText
      callbacks.onChunk(chunkText)
    }
  }

  const response = await result.response

  // Separate thinking from final response parts (in case stream missed it)
  const responseParts = response.candidates?.[0]?.content?.parts
  if (responseParts) {
    let finalContent = ''
    let finalThinking = ''
    for (const part of responseParts) {
      if (!part.text) continue
      if ((part as unknown as Record<string, unknown>).thought) {
        finalThinking += part.text
      } else {
        finalContent += part.text
      }
    }
    if (finalThinking) thinking = finalThinking
    if (finalContent) content = finalContent
  }

  // Extract function calls from final response parts
  const toolCalls: Array<{ name: string; arguments: Record<string, unknown> }> = []
  if (responseParts) {
    for (const part of responseParts) {
      const fc = (part as unknown as { functionCall?: { name: string; args: Record<string, unknown> } }).functionCall
      if (fc) {
        toolCalls.push({ name: fc.name, arguments: fc.args || {} })
      }
    }
  }

  const usage = (
    response as unknown as {
      usageMetadata?: {
        promptTokenCount?: number
        candidatesTokenCount?: number
        totalTokenCount?: number
        thoughtsTokenCount?: number
      }
    }
  ).usageMetadata

  return {
    content,
    thinking: thinking || undefined,
    toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
    usage: usage
      ? {
          promptTokens: usage.promptTokenCount ?? 0,
          completionTokens: usage.candidatesTokenCount ?? 0,
          totalTokens: usage.totalTokenCount ?? 0,
          thinkingTokens: usage.thoughtsTokenCount,
        }
      : undefined,
  }
}

function safeJsonParse(raw: string): Record<string, unknown> {
  try {
    return JSON.parse(raw)
  } catch {
    return {}
  }
}
