<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useTestStore } from '@/stores/test'
import { suiteV1 } from '@/test-suite/v1'
import type { TestCaseDef } from '@/types/test-suite'
import type { TestResultItem } from '@/types/api'

const testStore = useTestStore()

const progressPercent = computed(() => {
  if (testStore.progress.total === 0) return 0
  return Math.round((testStore.progress.current / testStore.progress.total) * 100)
})

const expandedCase = ref<string | null>(null)
const thinkingExpanded = ref<Record<string, boolean>>({})
const streamExpanded = ref<Record<string, boolean>>({})

function toggleThinking(caseId: string) {
  thinkingExpanded.value[caseId] = !thinkingExpanded.value[caseId]
}

function toggleStream(caseId: string) {
  streamExpanded.value[caseId] = !streamExpanded.value[caseId]
}

// Auto-expand current running case, which implicitly collapses the previous one.
// immediate: true is needed because onProgress fires synchronously before the first
// await, so by the time this component mounts, currentCaseId may already be set.
watch(
  () => testStore.progress.currentCaseId,
  (newId) => {
    if (newId) {
      expandedCase.value = newId
    }
  },
  { immediate: true },
)

function toggleCase(caseId: string) {
  expandedCase.value = expandedCase.value === caseId ? null : caseId
}

interface VisibleCase {
  testCase: TestCaseDef
  result: TestResultItem | undefined
  status: 'running' | 'completed'
}

const visibleCases = computed<VisibleCase[]>(() => {
  const cases: VisibleCase[] = []
  const total = suiteV1.testCases.length

  for (let i = 0; i <= testStore.progress.current && i < total; i++) {
    const tc = suiteV1.testCases[i]!
    const result = testStore.results.find((r) => r.test_case_id === tc.id)
    cases.push({
      testCase: tc,
      result,
      status: result
        ? 'completed'
        : testStore.runState === 'running' && i === testStore.progress.current
          ? 'running'
          : 'completed',
    })
  }

  return cases
})

function toolCallAnswer(caseId: string): string | null {
  const result = testStore.results.find(r => r.test_case_id === caseId)
  if (!result?.output_preview) return null
  return result.output_preview
}
</script>

<template>
  <div v-if="testStore.runState !== 'idle'" class="card">
    <h3 class="card__header">测试执行</h3>

    <!-- Progress -->
    <div class="progress-section">
      <div class="progress-bar-track">
        <div
          class="progress-bar-fill"
          :class="{ 'progress-bar-fill--done': testStore.runState === 'completed' }"
          :style="{ width: `${progressPercent}%` }"
        />
      </div>
      <p class="progress-text">
        <template v-if="testStore.runState === 'running'">
          正在测试第 {{ testStore.progress.current + 1 }}/{{ testStore.progress.total }} 项
          <template v-if="testStore.progress.currentCaseTitle">
            : {{ testStore.progress.currentCaseTitle }}
          </template>
        </template>
        <template v-else-if="testStore.runState === 'completed'">
          测试完成！共 {{ testStore.results.length }} 项
        </template>
        <template v-else-if="testStore.runState === 'cancelled'">
          测试已取消
        </template>
        <template v-else-if="testStore.runState === 'error'">
          测试出错
        </template>
      </p>
    </div>

    <!-- Case cards: shown as soon as each case starts (via progress) -->
    <div v-if="visibleCases.length > 0" class="case-grid">
      <div
        v-for="vc in visibleCases"
        :key="vc.testCase.id"
        class="case-card"
        :class="{
          'case-card--passed': vc.result?.passed,
          'case-card--failed': vc.result && !vc.result.passed,
          'case-card--running': vc.status === 'running',
        }"
      >
        <!-- Header -->
        <div class="case-card-main" @click="toggleCase(vc.testCase.id)">
          <span class="case-card-id">{{ vc.testCase.id }}</span>
          <span class="case-card-title">{{ vc.testCase.title }}</span>
          <span v-if="vc.status === 'running'" class="case-card-streaming-dot" />
          <span v-if="vc.result" class="case-card-status">{{ vc.result.passed ? '✓ 通过' : '✗ 未通过' }}</span>
          <span v-if="vc.result" class="case-card-time">{{ vc.result.response_time_ms }}ms</span>
          <span class="case-card-toggle">
            {{ expandedCase === vc.testCase.id ? '收起 ▲' : '展开 ▼' }}
          </span>
        </div>

        <!-- Expanded body -->
        <div v-if="expandedCase === vc.testCase.id" class="case-card-body">
          <!-- Prompt -->
          <div class="case-section">
            <div class="section-label">提示词</div>
            <pre class="section-pre">{{ vc.testCase.prompt }}</pre>
          </div>

          <!-- Expected results -->
          <div class="case-section case-section--row">
            <span class="section-label">期望调用参数：</span>
            <span
              v-for="(val, key) in vc.testCase.verify.arguments"
              :key="key"
              class="badge badge--info"
            >{{ key }}: {{ val }}</span>
          </div>

          <!-- Thinking process (separated from final output) -->
          <div
            v-if="testStore.streamingThinking[vc.testCase.id]"
            class="case-section case-section--thinking"
          >
            <div
              class="section-label section-label--thinking"
              @click="toggleThinking(vc.testCase.id)"
            >
              <span>{{ thinkingExpanded[vc.testCase.id] ? '🔽' : '▶️' }}</span>
              <span>思考过程</span>
              <span v-if="vc.status === 'running'" class="streaming-indicator" />
            </div>
            <pre
              v-if="thinkingExpanded[vc.testCase.id]"
              class="thinking-text"
            >{{ testStore.streamingThinking[vc.testCase.id] }}</pre>
          </div>

          <!-- Streaming / raw output -->
          <div
            v-if="testStore.streamingOutputs[vc.testCase.id]"
            class="case-section case-section--stream"
          >
            <div
              class="section-label section-label--stream"
              @click="toggleStream(vc.testCase.id)"
            >
              <span>{{ streamExpanded[vc.testCase.id] ? '🔽' : '▶️' }}</span>
              <template v-if="vc.status === 'running'">
                <span class="streaming-indicator" />模型输出中…
              </template>
              <template v-else>模型输出</template>
            </div>
            <pre
              v-if="streamExpanded[vc.testCase.id]"
              class="stream-text"
            >{{ testStore.streamingOutputs[vc.testCase.id] }}</pre>
          </div>

          <!-- Result comparison (shown after completion) -->
          <div v-if="vc.result" class="case-section case-section--result">
            <div
              class="result-verdict"
              :class="vc.result.passed ? 'result-verdict--pass' : 'result-verdict--fail'"
            >
              {{ vc.result.passed ? '✓ 实际输出与期望结果匹配' : '✗ 实际输出与期望结果不匹配' }}
            </div>
            <div v-if="toolCallAnswer(vc.testCase.id)" class="result-detail">
              <span class="info-label">函数调用：</span>
              <span class="answer-value">{{ toolCallAnswer(vc.testCase.id) }}</span>
            </div>
            <div v-if="vc.result.details" class="result-detail">
              <span class="info-label">详情：</span>
              <span class="answer-value">{{ vc.result.details }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Cancel / Error -->
    <div v-if="testStore.runState === 'running'" style="margin-top: var(--space-4)">
      <button class="btn btn--danger" @click="testStore.cancelTest()">取消测试</button>
    </div>
    <div v-if="testStore.runState === 'error'" class="alert alert--error mt-4">
      <span>{{ testStore.errorMessage || '未知错误' }}</span>
    </div>
  </div>
</template>

<style scoped>
.progress-section {
  margin-bottom: var(--space-4);
}

.progress-bar-track {
  width: 100%;
  height: 8px;
  background: var(--color-gray-200);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-full);
  transition: width 0.3s ease;
}

.progress-bar-fill--done {
  background: var(--color-success);
}

.progress-text {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-top: var(--space-2);
}

.case-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.case-card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.case-card--passed {
  border-color: var(--color-success-border);
  background: var(--color-success-bg);
}

.case-card--failed {
  border-color: var(--color-error-border);
  background: var(--color-error-bg);
}

.case-card--running {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 1px var(--color-primary);
}

.case-card-main {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.case-card-main:hover {
  background: rgba(0, 0, 0, 0.04);
}

.case-card-id {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.case-card-title {
  font-weight: 500;
}

.case-card-streaming-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
  animation: pulse 1.5s ease-in-out infinite;
  flex-shrink: 0;
}

.case-card-status {
  font-weight: 700;
  margin-left: auto;
  font-size: var(--font-size-xs);
}

.case-card--passed .case-card-status {
  color: var(--color-success);
}

.case-card--failed .case-card-status {
  color: var(--color-error);
}

.case-card-time {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.case-card-toggle {
  font-size: var(--font-size-xs);
  color: var(--color-primary);
  white-space: nowrap;
}

/* -- Body -- */
.case-card-body {
  display: flex;
  flex-direction: column;
}

.case-section {
  padding: var(--space-2) var(--space-3);
  border-top: 1px solid var(--color-border);
}

.case-section--row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.case-section--stream {
  background: var(--color-gray-50);
  max-height: 400px;
  overflow-y: auto;
  padding: var(--space-3);
}

.section-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--space-1);
}

.section-label--stream {
  cursor: pointer;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  gap: 6px;
  user-select: none;
}

.section-label--stream:hover {
  color: var(--color-text);
}

.streaming-indicator {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
  animation: pulse 1.5s ease-in-out infinite;
}

.section-pre {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  margin: 0;
}

.stream-text {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  margin: 0;
}

/* -- Thinking section -- */
.case-section--thinking {
  background: var(--color-gray-50);
  padding: var(--space-3);
}

.section-label--thinking {
  cursor: pointer;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  gap: 6px;
  user-select: none;
  font-size: var(--font-size-xs);
  font-weight: 600;
}

.section-label--thinking:hover {
  color: var(--color-text);
}

.thinking-text {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  margin: var(--space-2) 0 0 0;
  padding-top: var(--space-2);
  border-top: 1px solid var(--color-border);
}

/* -- Result -- */
.case-section--result {
  background: var(--color-gray-50);
}

.result-verdict {
  font-weight: 600;
  font-size: var(--font-size-sm);
}

.result-verdict--pass {
  color: var(--color-success);
}

.result-verdict--fail {
  color: var(--color-error);
}

.result-detail {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  font-size: var(--font-size-xs);
  margin-top: 4px;
}

.info-label {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.answer-value {
  font-weight: 500;
  color: var(--color-text);
  font-family: var(--font-mono);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
</style>
