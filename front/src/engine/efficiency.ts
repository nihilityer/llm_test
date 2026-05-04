import type { TestResultItem } from '@/types/api'

export function scoreEfficiency(results: TestResultItem[]): number {
  if (results.length === 0) return 0

  const validResults = results.filter((r) => r.response_time_ms > 0)
  if (validResults.length === 0) return 0

  const avgTime =
    validResults.reduce((s, r) => s + r.response_time_ms, 0) / validResults.length
  const totalTokens = validResults.reduce((s, r) => s + r.tokens_used, 0)

  const timeScore =
    avgTime < 2000 ? 15 : avgTime < 5000 ? 10 : avgTime < 10000 ? 5 : 0

  const avgTokensPerCase = totalTokens / validResults.length
  const tokenScore =
    avgTokensPerCase < 300 ? 10 : avgTokensPerCase < 600 ? 7 : avgTokensPerCase < 1000 ? 4 : 2

  return Math.min(25, timeScore + tokenScore)
}
