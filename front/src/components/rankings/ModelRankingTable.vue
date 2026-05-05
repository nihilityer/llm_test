<script setup lang="ts">
import type { ModelRankingEntry } from '@/types/api'
import { formatScore, formatRelativeTime } from '@/utils/format'

defineProps<{
  rankings: ModelRankingEntry[]
  loading: boolean
}>()

const emit = defineEmits<{
  select: [modelId: string]
}>()
</script>

<template>
  <div class="table-wrapper">
    <table class="table">
      <thead>
        <tr>
          <th style="width: 60px; text-align: center">排名</th>
          <th>模型名称</th>
          <th>别名</th>
          <th style="text-align: right">平均分</th>
          <th style="text-align: center">提交次数</th>
          <th style="text-align: center">网站数</th>
          <th style="text-align: right">最高分</th>
          <th style="text-align: right">最低分</th>
          <th style="text-align: right">最近测试</th>
        </tr>
      </thead>
      <tbody>
        <!-- Loading skeletons -->
        <template v-if="loading">
          <tr v-for="i in 5" :key="'skel-' + i">
            <td v-for="j in 9" :key="j">
              <div class="skeleton skeleton--text" style="margin: 4px 0" />
            </td>
          </tr>
        </template>

        <!-- Empty state -->
        <tr v-else-if="rankings.length === 0">
          <td colspan="9" style="text-align: center; padding: 48px 16px; color: var(--color-text-secondary)">
            暂无模型排名数据
          </td>
        </tr>

        <!-- Data rows -->
        <tr
          v-for="entry in rankings"
          :key="entry.model_id"
          class="table__row table__row--clickable"
          @click="emit('select', entry.model_id)"
        >
          <td style="text-align: center">
            <span class="rank-badge" :class="{ 'rank-badge--top3': entry.rank <= 3 }">
              {{ entry.rank }}
            </span>
          </td>
          <td class="fw-600">{{ entry.model_name }}</td>
          <td class="text-secondary text-sm">{{ entry.aliases.join(', ') }}</td>
          <td style="text-align: right; font-weight: 600" class="text-mono">
            {{ formatScore(entry.avg_score) }}
          </td>
          <td style="text-align: center">{{ entry.submission_count }}</td>
          <td style="text-align: center">{{ entry.website_count }}</td>
          <td style="text-align: right" class="text-mono">{{ formatScore(entry.max_score) }}</td>
          <td style="text-align: right" class="text-mono text-secondary">{{ formatScore(entry.min_score) }}</td>
          <td style="text-align: right" class="text-sm text-secondary">
            {{ formatRelativeTime(entry.last_tested_at) }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.rank-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-md);
  font-weight: 700;
  font-size: var(--font-size-sm);
  background: var(--color-gray-100);
  color: var(--color-text-secondary);
}

.rank-badge--top3 {
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.fw-600 {
  font-weight: 600;
}
</style>
