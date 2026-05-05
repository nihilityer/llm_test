<script setup lang="ts">
import type { ModelSummary } from '@/types/api'
import { formatScore } from '@/utils/format'

defineProps<{
  model: ModelSummary | null
  loading: boolean
}>()
</script>

<template>
  <div class="card">
    <!-- Loading -->
    <template v-if="loading">
      <div class="skeleton skeleton--heading" />
      <div class="skeleton skeleton--text" style="width: 60%" />
      <div class="skeleton skeleton--text" style="width: 30%" />
    </template>

    <!-- Error -->
    <template v-else-if="!model">
      <div class="alert alert--error">模型信息加载失败</div>
    </template>

    <!-- Data -->
    <template v-else>
      <h2>{{ model.name }}</h2>
      <div class="aliases-row">
        <span v-for="a in model.aliases" :key="a" class="badge badge--info">{{ a }}</span>
        <span v-if="model.aliases.length === 0" class="text-secondary text-sm">无别名</span>
      </div>
      <div class="stats-row">
        <div class="stat">
          <span class="stat-value">{{ formatScore(model.avg_score) }}</span>
          <span class="stat-label">平均分</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ model.submission_count }}</span>
          <span class="stat-label">提交次数</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ model.website_count }}</span>
          <span class="stat-label">测试网站数</span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.aliases-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.stats-row {
  display: flex;
  gap: var(--space-8);
  margin-top: var(--space-6);
}

.stat {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: var(--font-size-2xl);
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--color-primary);
}

.stat-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-top: 2px;
}
</style>
