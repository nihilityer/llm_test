<script setup lang="ts">
import { ref } from 'vue'
import type { SubmissionDetail } from '@/types/api'
import { formatScore, formatRelativeTime, formatDate } from '@/utils/format'

defineProps<{
  submissions: SubmissionDetail[]
  loading: boolean
}>()

const expandedIds = ref<Set<string>>(new Set())

function toggle(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id)
  } else {
    expandedIds.value.add(id)
  }
  // Trigger reactivity
  expandedIds.value = new Set(expandedIds.value)
}

function styleLabel(style: string): string {
  const map: Record<string, string> = {
    openai: 'OpenAI',
    anthropic: 'Anthropic',
    gemini: 'Gemini',
  }
  return map[style] || style
}
</script>

<template>
  <div class="table-wrapper">
    <table class="table">
      <thead>
        <tr>
          <th style="width: 30px" />
          <th>提交者</th>
          <th>模型</th>
          <th style="text-align: right">总分</th>
          <th>各维度</th>
          <th>风格</th>
          <th>版本</th>
          <th style="text-align: right">时间</th>
        </tr>
      </thead>
      <tbody>
        <!-- Loading -->
        <template v-if="loading">
          <tr v-for="i in 5" :key="'skel-' + i">
            <td v-for="j in 8" :key="j">
              <div class="skeleton skeleton--text" style="margin: 4px 0" />
            </td>
          </tr>
        </template>

        <!-- Empty -->
        <tr v-else-if="submissions.length === 0">
          <td colspan="8" style="text-align: center; padding: 48px 16px; color: var(--color-text-secondary)">
            暂无提交记录
          </td>
        </tr>

        <!-- Data -->
        <template v-for="sub in submissions" :key="sub.id">
          <tr class="table__row table__row--clickable" @click="toggle(sub.id)">
            <td>
              <span class="expand-icon" :class="{ 'expand-icon--open': expandedIds.has(sub.id) }">&#9654;</span>
            </td>
            <td>
              <div class="submitter-cell">
                <img
                  v-if="sub.submitter_avatar"
                  :src="sub.submitter_avatar"
                  class="submitter-avatar"
                  alt=""
                />
                <span class="submitter-name">{{ sub.submitter_name || '匿名用户' }}</span>
                <span
                  class="badge"
                  :class="sub.submitter_type === 'oauth' ? 'badge--info' : 'badge--default'"
                  style="margin-left: 6px"
                >
                  {{ sub.submitter_type === 'oauth' ? 'OAuth' : '匿名' }}
                </span>
              </div>
            </td>
            <td>
              <span class="badge badge--primary">{{ sub.model_name }}</span>
            </td>
            <td style="text-align: right; font-weight: 600" class="text-mono">
              {{ formatScore(sub.total_score) }}
            </td>
            <td>
              <div class="dimension-mini">
                <span
                  v-for="(score, key) in sub.dimension_scores"
                  :key="key"
                  class="badge badge--default"
                  style="margin-right: 4px"
                >
                  {{ key }}: {{ score }}
                </span>
              </div>
            </td>
            <td>
              <span class="badge badge--info">{{ styleLabel(sub.api_style) }}</span>
            </td>
            <td class="text-sm">{{ sub.test_suite_version }}</td>
            <td
              style="text-align: right; font-size: var(--font-size-xs); color: var(--color-text-secondary)"
              :title="formatDate(sub.created_at)"
            >
              {{ formatRelativeTime(sub.created_at) }}
            </td>
          </tr>
          <!-- Expanded dimension detail -->
          <tr v-if="expandedIds.has(sub.id)" class="expand-row">
            <td colspan="8">
              <div class="dimension-detail">
                <h4>各维度得分</h4>
                <div class="dimension-bars">
                  <div
                    v-for="(score, key) in sub.dimension_scores"
                    :key="key"
                    class="dimension-bar-item"
                  >
                    <div class="dimension-bar-label">
                      <span>{{ key }}</span>
                      <span class="text-mono">{{ score }}</span>
                    </div>
                    <div class="dimension-bar-track">
                      <div
                        class="dimension-bar-fill"
                        :style="{ width: `${Math.min(100, (Number(score) / 40) * 100)}%` }"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.submitter-cell {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.submitter-avatar {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-full);
  object-fit: cover;
}

.submitter-name {
  font-weight: 500;
}

.expand-icon {
  display: inline-block;
  font-size: 10px;
  transition: transform var(--transition-fast);
  color: var(--color-gray-400);
}

.expand-icon--open {
  transform: rotate(90deg);
}

.expand-row td {
  background: var(--color-gray-50);
  padding: var(--space-4) var(--space-6);
}

.dimension-detail h4 {
  font-size: var(--font-size-sm);
  margin-bottom: var(--space-3);
  color: var(--color-text-secondary);
}

.dimension-bars {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.dimension-bar-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dimension-bar-label {
  display: flex;
  justify-content: space-between;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.dimension-bar-track {
  height: 6px;
  background: var(--color-gray-200);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.dimension-bar-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-full);
  transition: width 0.5s ease;
}

.dimension-mini {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
}
</style>
