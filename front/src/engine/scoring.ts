import type { TestSuite, DimensionDef } from '@/types/test-suite'
import type { TestResultItem } from '@/types/api'
import type { DimensionScores } from '@/types/scoring'

export function calculateScores(
  results: TestResultItem[],
  suite: TestSuite,
): { dimensionScores: DimensionScores; totalScore: number } {
  const dimensionScores: DimensionScores = {}

  for (const dim of suite.dimensions) {
    const score = scoreDimension(results, suite, dim)
    dimensionScores[dim.key] = Math.min(dim.maxScore, Math.max(0, Math.round(score * 10) / 10))
  }

  const totalScore = Math.min(
    100,
    Math.round(
      Object.values(dimensionScores).reduce((s: number, v: number) => s + v, 0) * 10,
    ) / 10,
  )

  return { dimensionScores, totalScore }
}

function scoreDimension(
  results: TestResultItem[],
  suite: TestSuite,
  dim: DimensionDef,
): number {
  switch (dim.type) {
    case 'weighted_correctness':
      return scoreWeightedCorrectness(results, suite, dim)
    case 'first_token_latency':
      return scoreFirstTokenLatency(results, dim)
    case 'token_efficiency':
      return scoreTokenEfficiency(results, dim)
    case 'consistency':
      return scoreConsistency(results, dim)
    default:
      return 0
  }
}

function scoreWeightedCorrectness(
  results: TestResultItem[],
  suite: TestSuite,
  dim: DimensionDef,
): number {
  if (results.length === 0) return 0

  const weights = suite.scoring.difficultyWeights
  const diffMap = new Map(suite.testCases.map((tc) => [tc.id, tc.difficulty]))

  let weightedPassed = 0
  let weightedTotal = 0

  for (const r of results) {
    const difficulty = diffMap.get(r.test_case_id) ?? 'medium'
    const w = weights[difficulty]
    weightedTotal += w
    if (r.passed) {
      weightedPassed += w
    }
  }

  return weightedTotal > 0 ? (weightedPassed / weightedTotal) * dim.maxScore : 0
}

function scoreThreshold(
  value: number,
  params: Record<string, number> | undefined,
): number {
  if (!params) return 0

  const bands: Array<{ threshold: number; score: number }> = []
  let floorScore = 0

  for (const [key, score] of Object.entries(params)) {
    if (key === 'floor') {
      floorScore = score
    } else if (key.startsWith('threshold_')) {
      const threshold = parseThresholdKey(key)
      if (threshold !== null) {
        bands.push({ threshold, score })
      }
    }
  }

  bands.sort((a, b) => a.threshold - b.threshold)

  for (const band of bands) {
    if (value < band.threshold) return band.score
  }

  return floorScore
}

function parseThresholdKey(key: string): number | null {
  const rest = key.slice('threshold_'.length)
  if (rest.startsWith('cv_')) {
    const digits = rest.slice('cv_'.length)
    const num = parseInt(digits, 10)
    return isNaN(num) ? null : num / Math.pow(10, digits.length)
  }
  const numStr = rest.endsWith('ms') ? rest.slice(0, -2) : rest
  const num = parseInt(numStr, 10)
  return isNaN(num) ? null : num
}

function scoreFirstTokenLatency(
  results: TestResultItem[],
  dim: DimensionDef,
): number {
  const valid = results.filter((r) => r.first_token_time_ms > 0)
  if (valid.length === 0) return 0

  const avg = valid.reduce((s, r) => s + r.first_token_time_ms, 0) / valid.length
  return scoreThreshold(avg, dim.params)
}

function scoreTokenEfficiency(
  results: TestResultItem[],
  dim: DimensionDef,
): number {
  const passed = results.filter((r) => r.passed && r.tokens_used > 0)
  if (passed.length === 0) return 0

  const avgTokens = passed.reduce((s, r) => s + r.tokens_used, 0) / passed.length
  return scoreThreshold(avgTokens, dim.params)
}

function scoreConsistency(
  results: TestResultItem[],
  dim: DimensionDef,
): number {
  const valid = results.filter((r) => r.response_time_ms > 0)
  if (valid.length < 2) return dim.maxScore

  const times = valid.map((r) => r.response_time_ms)
  const mean = times.reduce((s, t) => s + t, 0) / times.length
  if (mean === 0) return dim.maxScore

  const variance = times.reduce((s, t) => s + (t - mean) ** 2, 0) / times.length
  const cv = Math.sqrt(variance) / mean
  return scoreThreshold(cv, dim.params)
}
