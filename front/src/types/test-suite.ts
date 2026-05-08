import type { TestResultItem } from './api'
import type { ScoringDimensionType } from '@/types/scoring.ts'

export type Difficulty = 'easy' | 'medium' | 'hard'

export interface DimensionDef {
  key: string
  name: string
  maxScore: number
  type: ScoringDimensionType
  params?: Record<string, number>
}

export interface TestCaseEvalResult {
  passed: boolean
  details: string
  toolUseScore?: number
  codeContent?: string
}

export interface TestCaseDef {
  id: string
  title: string
  description: string
  difficulty: Difficulty
  prompt: string
  parameters: Record<string, unknown>
  verify: { arguments: Record<string, unknown> }
}

export interface TestSuite {
  version: string
  name: string
  description: string
  dimensions: DimensionDef[]
  testCases: TestCaseDef[]
  scoring: {
    difficultyWeights: Record<Difficulty, number>
  }
}

export function createTestResultItem(
  testCaseId: string,
  evalResult: TestCaseEvalResult,
  responseTimeMs: number,
  firstTokenTimeMs: number,
  tokensUsed: number,
  outputPreview: string,
): TestResultItem {
  return {
    test_case_id: testCaseId,
    passed: evalResult.passed,
    response_time_ms: responseTimeMs,
    first_token_time_ms: firstTokenTimeMs,
    tokens_used: tokensUsed,
    output_preview: outputPreview,
    details: evalResult.details,
  }
}
