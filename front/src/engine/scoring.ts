import type { TestSuite } from '@/types/test-suite'
import type { TestResultItem } from '@/types/api'
import type { DimensionScores } from '@/types/scoring'
import { scoreEfficiency } from './efficiency'

export function calculateScores(
  results: TestResultItem[],
  suite: TestSuite,
): { dimensionScores: DimensionScores; totalScore: number } {
  const dimensionScores: DimensionScores = {}

  for (const dim of suite.dimensions) {
    switch (dim.key) {
      case 'correctness': {
        const passed = results.filter((r) => r.passed).length
        const total = results.length
        dimensionScores.correctness =
          total > 0
            ? Math.round(((passed / total) * dim.maxScore) * 10) / 10
            : 0
        break
      }
      case 'efficiency':
        dimensionScores.efficiency = scoreEfficiency(results)
        break
    }
  }

  for (const dim of suite.dimensions) {
    if (dimensionScores[dim.key] !== undefined) {
      dimensionScores[dim.key] = Math.min(
        dim.maxScore,
        Math.max(0, dimensionScores[dim.key]!),
      )
    }
  }

  const totalScore = Math.min(
    100,
    Math.round(
      Object.values(dimensionScores).reduce(
        (s: number, v: number) => s + v,
        0,
      ) * 10,
    ) / 10,
  )

  return { dimensionScores, totalScore }
}
