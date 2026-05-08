import carWashJson from './cases/car_wash.json'
import schoolCleaningJson from './cases/school_cleaning.json'
import type { TestCaseDef, TestSuite } from '@/types/test-suite'

function asTestCase(json: Record<string, unknown>): TestCaseDef {
  return {
    id: json.id as string,
    title: json.title as string,
    description: json.description as string,
    difficulty: json.difficulty as TestCaseDef['difficulty'],
    prompt: json.prompt as string,
    parameters: json.parameters as Record<string, unknown>,
    verify: json.verify as { arguments: Record<string, unknown> },
  }
}

export const suiteV1: TestSuite = {
  version: 'v1',
  name: '判断力综合测试 v1',
  description: '测试 AI 模型在常识推理和逻辑判断方面的能力，包含 2 道精选测试用例。',
  scoring: {
    difficultyWeights: { easy: 3, medium: 2, hard: 1 },
  },
  dimensions: [
    {
      key: 'correctness',
      name: '正确性',
      maxScore: 55,
      type: 'weighted_correctness',
    },
    {
      key: 'first_token_latency',
      name: '首 Token 延迟',
      maxScore: 15,
      type: 'first_token_latency',
      params: { threshold_500ms: 15, threshold_1000ms: 10, threshold_3000ms: 5, floor: 2 },
    },
    {
      key: 'token_efficiency',
      name: 'Token 效率',
      maxScore: 20,
      type: 'token_efficiency',
      params: { threshold_200: 20, threshold_500: 14, threshold_1000: 8, floor: 4 },
    },
    {
      key: 'consistency',
      name: '响应一致性',
      maxScore: 10,
      type: 'consistency',
      params: { threshold_cv_03: 10, threshold_cv_06: 7, threshold_cv_10: 4, floor: 2 },
    },
  ],
  testCases: [asTestCase(carWashJson), asTestCase(schoolCleaningJson)],
}
