import type { ApiConfig, TestProgress } from '@/types/scoring'
import type { TestCaseEvalResult, TestSuite } from '@/types/test-suite'
import type { TestResultItem } from '@/types/api'
import { callLLMStream } from './sdk-client'
import { createTestResultItem } from '@/types/test-suite'
import { truncate } from '@/utils/format'

export interface RunCallbacks {
  onProgress: (progress: TestProgress) => void
  onChunk: (caseId: string, chunk: string) => void
  onThinking: (caseId: string, chunk: string) => void
  onCaseComplete: (result: TestResultItem) => void
}

export async function runTestSuite(
  apiConfig: ApiConfig,
  suite: TestSuite,
  callbacks: RunCallbacks,
  signal: AbortSignal,
): Promise<TestResultItem[]> {
  const results: TestResultItem[] = []
  const total = suite.testCases.length

  for (let i = 0; i < total; i++) {
    if (signal.aborted) {
      return results
    }

    const testCase = suite.testCases[i]!

    callbacks.onProgress({
      current: i,
      total,
      currentCaseId: testCase.id,
      currentCaseTitle: testCase.title,
    })

    const caseSignal = AbortSignal.any([signal, AbortSignal.timeout(60000)])

    try {
      const startTime = performance.now()
      const response = await callLLMStream(
        apiConfig,
        {
          prompt: testCase.prompt,
          signal: caseSignal,
          parameters: testCase.parameters,
        },
        {
          onChunk: (text: string) => callbacks.onChunk(testCase.id, text),
          onThinking: (text: string) => callbacks.onThinking(testCase.id, text),
        },
      )
      const elapsed = Math.round(performance.now() - startTime)

      const evalResult = evaluateFunctionCall(response.toolCalls, testCase.verify.arguments)

      const previewText = toolCallsPreview(response.toolCalls) || response.content

      const resultItem = createTestResultItem(
        testCase.id,
        evalResult,
        elapsed,
        response.usage?.totalTokens ?? 0,
        truncate(previewText, 300),
      )

      results.push(resultItem)
      callbacks.onCaseComplete(resultItem)
    } catch (err: unknown) {
      if (err instanceof DOMException && err.name === 'AbortError') {
        if (signal.aborted) return results
        const resultItem = createTestResultItem(
          testCase.id,
          { passed: false, details: '用例超时 (60s)' },
          60000,
          0,
          '',
        )
        results.push(resultItem)
        callbacks.onCaseComplete(resultItem)
      } else {
        const message = err instanceof Error ? err.message : '未知错误'
        const resultItem = createTestResultItem(
          testCase.id,
          { passed: false, details: `执行错误: ${message}` },
          0,
          0,
          '',
        )
        results.push(resultItem)
        callbacks.onCaseComplete(resultItem)
      }
    }
  }

  callbacks.onProgress({
    current: total,
    total,
    currentCaseId: null,
    currentCaseTitle: null,
  })

  return results
}

function evaluateFunctionCall(
  toolCalls: Array<{ name: string; arguments: Record<string, unknown> }> | undefined,
  expectedArgs: Record<string, unknown>,
): TestCaseEvalResult {
  if (!toolCalls || toolCalls.length === 0) {
    return { passed: false, details: '模型未调用预期的函数（submit_answer）' }
  }

  const matched = toolCalls.find(tc => tc.name === 'submit_answer')
  if (!matched) {
    return { passed: false, details: `模型调用了其他工具: ${toolCalls.map(tc => tc.name).join(', ')}` }
  }

  const mismatches: string[] = []
  for (const [key, expected] of Object.entries(expectedArgs)) {
    const actual = matched.arguments[key]
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      mismatches.push(`${key}: 期望 ${JSON.stringify(expected)}, 实际 ${JSON.stringify(actual)}`)
    }
  }

  if (mismatches.length > 0) {
    return { passed: false, details: `函数调用参数不匹配: ${mismatches.join('; ')}` }
  }

  const argsSummary = Object.entries(expectedArgs)
    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
    .join(', ')
  return { passed: true, details: `函数调用参数正确: ${argsSummary}` }
}

function toolCallsPreview(toolCalls: Array<{ name: string; arguments: Record<string, unknown> }> | undefined): string {
  if (!toolCalls || toolCalls.length === 0) return ''
  return toolCalls.map(tc => `${tc.name}(${JSON.stringify(tc.arguments)})`).join(', ')
}
