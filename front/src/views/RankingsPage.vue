<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useRankingsStore } from '@/stores/rankings'
import StyleFilter from '@/components/rankings/StyleFilter.vue'
import SearchBar from '@/components/rankings/SearchBar.vue'
import RankingTable from '@/components/rankings/RankingTable.vue'
import ModelRankingTable from '@/components/rankings/ModelRankingTable.vue'

const router = useRouter()
const rankingsStore = useRankingsStore()

onMounted(() => {
  rankingsStore.loadRankings(true)
})

watch(() => rankingsStore.filters.search, () => {
  rankingsStore.loadRankings(true)
})

watch(() => rankingsStore.filters.style, () => {
  rankingsStore.loadRankings(true)
})

function handleWebsiteSelect(websiteId: string) {
  router.push(`/website/${websiteId}`)
}

function handleModelSelect(modelId: string) {
  router.push(`/model/${modelId}`)
}
</script>

<template>
  <div class="rankings-page">
    <h1 class="page-title">排行榜</h1>

    <!-- Type Tabs -->
    <div class="type-tabs">
      <button
        class="type-tab"
        :class="{ 'type-tab--active': rankingsStore.rankingType === 'website' }"
        @click="rankingsStore.setRankingType('website')"
      >
        网站排行
      </button>
      <button
        class="type-tab"
        :class="{ 'type-tab--active': rankingsStore.rankingType === 'model' }"
        @click="rankingsStore.setRankingType('model')"
      >
        模型排行
      </button>
    </div>

    <!-- Filters -->
    <div class="filters-bar">
      <StyleFilter v-model="rankingsStore.filters.style" />
      <SearchBar v-model="rankingsStore.filters.search" />
    </div>

    <!-- Error -->
    <div v-if="rankingsStore.error" class="alert alert--error mb-4">
      <span>{{ rankingsStore.error }}</span>
      <button class="btn btn--sm btn--ghost" @click="rankingsStore.loadRankings(true)">重试</button>
    </div>

    <!-- Website Table -->
    <RankingTable
      v-if="rankingsStore.rankingType === 'website'"
      :rankings="rankingsStore.rankings"
      :loading="rankingsStore.loading && rankingsStore.rankings.length === 0"
      @select="handleWebsiteSelect"
    />

    <!-- Model Table -->
    <ModelRankingTable
      v-else
      :rankings="rankingsStore.modelRankings"
      :loading="rankingsStore.loading && rankingsStore.modelRankings.length === 0"
      @select="handleModelSelect"
    />

    <!-- Load more -->
    <div v-if="rankingsStore.hasMore" class="load-more-section">
      <button
        class="btn btn--secondary"
        :disabled="rankingsStore.loading"
        @click="rankingsStore.loadMore()"
      >
        {{ rankingsStore.loading ? '加载中...' : '加载更多' }}
      </button>
    </div>

    <!-- End -->
    <div
      v-else-if="
        (rankingsStore.rankingType === 'website' ? rankingsStore.rankings.length : rankingsStore.modelRankings.length) > 0
        && rankingsStore.total > 0
      "
      class="end-message"
    >
      已加载全部 {{ rankingsStore.total }} 条记录
    </div>
  </div>
</template>

<style scoped>
.rankings-page {
  max-width: 1100px;
  margin: 0 auto;
}

.type-tabs {
  display: flex;
  gap: 0;
  margin-bottom: var(--space-6);
  border-bottom: 2px solid var(--color-gray-200);
}

.type-tab {
  padding: var(--space-2) var(--space-6);
  border: none;
  background: none;
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  transition: color var(--transition-fast), border-color var(--transition-fast);
}

.type-tab:hover {
  color: var(--color-text-primary);
}

.type-tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.filters-bar {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-4);
  align-items: center;
  margin-bottom: var(--space-6);
}

.filters-bar > :first-child {
  flex-shrink: 0;
}

.filters-bar > :last-child {
  flex: 1;
  min-width: 200px;
}

.load-more-section {
  text-align: center;
  margin-top: var(--space-6);
}

.end-message {
  text-align: center;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  margin-top: var(--space-6);
}
</style>
