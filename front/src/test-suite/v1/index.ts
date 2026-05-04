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
  dimensions: [
    { key: 'correctness', name: '正确性', maxScore: 75 },
    { key: 'efficiency', name: '运行效率', maxScore: 25 },
  ],
  testCases: [asTestCase(carWashJson), asTestCase(schoolCleaningJson)],
}
