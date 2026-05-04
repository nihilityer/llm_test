<script setup lang="ts">
import { ref } from 'vue'
import { suiteV1 } from '@/test-suite/v1'
import type { Difficulty } from '@/types/test-suite'

const expandedIds = ref<Set<string>>(new Set())

function toggle(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id)
  } else {
    expandedIds.value.add(id)
  }
  expandedIds.value = new Set(expandedIds.value)
}

function difficultyLabel(d: Difficulty): string {
  return { easy: '简单', medium: '中等', hard: '困难' }[d]
}

function difficultyClass(d: Difficulty): string {
  return { easy: 'badge--success', medium: 'badge--warning', hard: 'badge--error' }[d]
}
</script>

<template>
  <div class="card">
    <div class="suite-header">
      <span class="badge badge--info">{{ suiteV1.version }}</span>
      <h3 class="suite-name">{{ suiteV1.name }}</h3>
    </div>
    <p class="suite-desc">{{ suiteV1.description }}</p>

    <div class="case-list">
      <div
        v-for="tc in suiteV1.testCases"
        :key="tc.id"
        class="case-item"
        :class="{ 'case-item--open': expandedIds.has(tc.id) }"
      >
        <button class="case-header" @click="toggle(tc.id)">
          <div class="case-header-left">
            <span class="case-id">{{ tc.id }}</span>
            <span class="case-title">{{ tc.title }}</span>
            <span class="badge" :class="difficultyClass(tc.difficulty)">
              {{ difficultyLabel(tc.difficulty) }}
            </span>
          </div>
          <span class="case-toggle">{{ expandedIds.has(tc.id) ? '收起' : '展开' }}</span>
        </button>
        <div v-if="expandedIds.has(tc.id)" class="case-body">
          <p class="case-desc">{{ tc.description }}</p>
          <div class="case-prompt">
            <pre>{{ tc.prompt }}</pre>
          </div>
          <div v-if="tc.parameters" class="case-params">
            <div class="params-label">期望函数参数 (submit_answer)</div>
            <pre>{{ JSON.stringify(tc.parameters, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.suite-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

.suite-name {
  font-size: var(--font-size-xl);
}

.suite-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-4);
}

.case-list {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.case-item {
  border-bottom: 1px solid var(--color-border);
}

.case-item:last-child {
  border-bottom: none;
}

.case-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--space-3) var(--space-4);
  background: var(--color-white);
  border: none;
  cursor: pointer;
  font-family: var(--font-family);
  transition: background var(--transition-fast);
}

.case-header:hover {
  background: var(--color-gray-50);
}

.case-header-left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.case-id {
  font-size: var(--font-size-xs);
  font-family: var(--font-mono);
  color: var(--color-gray-400);
}

.case-title {
  font-weight: 500;
  text-align: left;
}

.case-toggle {
  font-size: var(--font-size-xs);
  color: var(--color-primary);
  white-space: nowrap;
}

.case-body {
  padding: var(--space-4);
  background: var(--color-gray-50);
  border-top: 1px solid var(--color-border);
}

.case-desc {
  font-size: var(--font-size-sm);
  margin-bottom: var(--space-3);
}

.case-prompt pre {
  font-size: var(--font-size-xs);
  white-space: pre-wrap;
  word-break: break-word;
}

.case-params {
  margin-top: var(--space-3);
}

.params-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--space-1);
}

.case-params pre {
  font-size: var(--font-size-xs);
  white-space: pre-wrap;
  word-break: break-word;
  background: var(--color-white);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2);
}
</style>
