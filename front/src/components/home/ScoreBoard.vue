<script setup lang="ts">
import { computed } from 'vue'
import { useTestStore } from '@/stores/test'
import { suiteV1 } from '@/test-suite/v1'
import { formatScore } from '@/utils/format'

const testStore = useTestStore()

const scoreColor = computed(() => {
  const s = testStore.totalScore ?? 0
  if (s >= 80) return 'var(--color-success)'
  if (s >= 60) return 'var(--color-info)'
  if (s >= 40) return 'var(--color-warning)'
  return 'var(--color-error)'
})

// Radar chart: 4 axes at 0, 90, 180, 270 degrees
const cx = 120
const cy = 120
const r = 90

function axisEndpoint(angleDeg: number, value: number, maxScore: number): { x: number; y: number } {
  const ratio = maxScore > 0 ? value / maxScore : 0
  const angle = (angleDeg - 90) * (Math.PI / 180)
  return {
    x: cx + Math.cos(angle) * r * ratio,
    y: cy + Math.sin(angle) * r * ratio,
  }
}

function gridPoint(angleDeg: number, level: number): { x: number; y: number } {
  const angle = (angleDeg - 90) * (Math.PI / 180)
  return {
    x: cx + Math.cos(angle) * r * level,
    y: cy + Math.sin(angle) * r * level,
  }
}

function labelPoint(angleDeg: number): { x: number; y: number; anchor: string } {
  const angle = (angleDeg - 90) * (Math.PI / 180)
  const dist = r + 22
  const x = cx + Math.cos(angle) * dist
  const y = cy + Math.sin(angle) * dist
  let anchor = 'middle'
  if (angleDeg === 0) anchor = 'start'
  else if (angleDeg === 180) anchor = 'end'
  return { x, y, anchor }
}

const axes = computed(() => {
  const dims = suiteV1.dimensions
  return dims.map((dim, i) => {
    const angle = (360 / dims.length) * i
    const score = testStore.scores?.[dim.key] ?? 0
    return { ...dim, angle, score }
  })
})

function dataPoints(): string {
  return axes.value
    .map((a) => {
      const pt = axisEndpoint(a.angle, a.score, a.maxScore)
      return `${pt.x},${pt.y}`
    })
    .join(' ')
}

function gridPolygon(level: number): string {
  const angles = axes.value.map((a) => a.angle)
  return angles
    .map((a) => {
      const pt = gridPoint(a, level)
      return `${pt.x},${pt.y}`
    })
    .join(' ')
}

const showDetails = ref(false)
</script>

<template>
  <div v-if="testStore.runState === 'completed' && testStore.scores" class="card">
    <!-- Total Score -->
    <div class="total-score-section">
      <div class="total-score" :style="{ color: scoreColor }">
        {{ formatScore(testStore.totalScore ?? 0) }}
      </div>
      <div class="total-label">总分</div>
    </div>

    <!-- Radar Chart (only for 3+ dimensions) -->
    <div v-if="axes.length >= 3" class="radar-section">
      <svg viewBox="0 0 240 240" class="radar-svg">
        <!-- Grid -->
        <polygon
          v-for="level in [0.25, 0.5, 0.75, 1]"
          :key="'grid-' + level"
          :points="gridPolygon(level)"
          fill="none"
          stroke="var(--color-gray-200)"
          stroke-width="1"
        />
        <!-- Axes -->
        <line
          v-for="a in axes"
          :key="'axis-' + a.key"
          :x1="cx"
          :y1="cy"
          :x2="gridPoint(a.angle, 1).x"
          :y2="gridPoint(a.angle, 1).y"
          stroke="var(--color-gray-200)"
          stroke-width="1"
        />
        <!-- Data polygon -->
        <polygon
          :points="dataPoints()"
          fill="var(--color-primary)"
          fill-opacity="0.2"
          stroke="var(--color-primary)"
          stroke-width="2"
        />
        <!-- Data points -->
        <circle
          v-for="a in axes"
          :key="'dot-' + a.key"
          :cx="axisEndpoint(a.angle, a.score, a.maxScore).x"
          :cy="axisEndpoint(a.angle, a.score, a.maxScore).y"
          r="3"
          fill="var(--color-primary)"
        />
        <!-- Labels -->
        <text
          v-for="a in axes"
          :key="'label-' + a.key"
          :x="labelPoint(a.angle).x"
          :y="labelPoint(a.angle).y"
          :text-anchor="labelPoint(a.angle).anchor"
          dominant-baseline="middle"
          font-size="10"
          fill="var(--color-text-secondary)"
        >
          {{ a.name }} {{ a.score }}
        </text>
      </svg>
    </div>

    <!-- Dimension Bars -->
    <div class="dimension-bars">
      <div v-for="a in axes" :key="a.key" class="dim-bar-item">
        <div class="dim-bar-header">
          <span>{{ a.name }}</span>
          <span class="text-mono">{{ a.score }}/{{ a.maxScore }}</span>
        </div>
        <div class="dim-bar-track">
          <div
            class="dim-bar-fill"
            :style="{ width: `${(a.score / a.maxScore) * 100}%` }"
          />
        </div>
      </div>
    </div>

    <!-- Per-case details -->
    <div class="details-section">
      <button class="btn btn--ghost" @click="showDetails = !showDetails">
        {{ showDetails ? '收起详细结果' : '查看详细结果' }}
      </button>
      <div v-if="showDetails" class="case-details-list">
        <div v-for="r in testStore.results" :key="r.test_case_id" class="detail-item">
          <div class="detail-header">
            <span class="text-mono detail-id">{{ r.test_case_id }}</span>
            <span class="badge" :class="r.passed ? 'badge--success' : 'badge--error'">
              {{ r.passed ? '通过' : '未通过' }}
            </span>
            <span class="detail-meta text-sm text-secondary">
              {{ r.response_time_ms }}ms | {{ r.tokens_used }} tokens
            </span>
          </div>
          <pre v-if="r.output_preview" class="detail-preview">{{ r.output_preview }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { ref } from 'vue'
</script>

<style scoped>
.total-score-section {
  text-align: center;
  margin-bottom: var(--space-6);
}

.total-score {
  font-size: 4rem;
  font-weight: 800;
  font-family: var(--font-mono),serif;
  line-height: 1;
}

.total-label {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
  margin-top: var(--space-1);
}

.radar-section {
  margin-bottom: var(--space-6);
}

.radar-svg {
  width: 100%;
  max-width: 320px;
  display: block;
  margin: 0 auto;
}

.dimension-bars {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.dim-bar-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dim-bar-header {
  display: flex;
  justify-content: space-between;
  font-size: var(--font-size-sm);
  font-weight: 500;
}

.dim-bar-track {
  height: 8px;
  background: var(--color-gray-200);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.dim-bar-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-full);
  transition: width 0.8s ease;
}

.details-section {
  margin-top: var(--space-4);
  border-top: 1px solid var(--color-border);
  padding-top: var(--space-4);
}

.case-details-list {
  margin-top: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.detail-item {
  background: var(--color-gray-50);
  border-radius: var(--radius-md);
  padding: var(--space-3);
}

.detail-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

.detail-id {
  color: var(--color-gray-500);
  font-size: var(--font-size-xs);
}

.detail-meta {
  margin-left: auto;
}

.detail-preview {
  font-size: var(--font-size-xs);
  margin-top: var(--space-2);
  padding: var(--space-2);
  max-height: 120px;
  overflow-y: auto;
}
</style>
