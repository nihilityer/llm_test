<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useRankingsStore } from '@/stores/rankings'
import StyleFilter from '@/components/rankings/StyleFilter.vue'
import SearchBar from '@/components/rankings/SearchBar.vue'
import RankingTable from '@/components/rankings/RankingTable.vue'

const router = useRouter()
const rankingsStore = useRankingsStore()

onMounted(() => {
  rankingsStore.loadRankings(true)
})

function handleSelect(websiteId: string) {
  router.push(`/website/${websiteId}`)
}
</script>

<template>
  <div class="rankings-page">
    <h1 class="page-title">排行榜</h1>

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

    <!-- Table -->
    <RankingTable
      :rankings="rankingsStore.rankings"
      :loading="rankingsStore.loading && rankingsStore.rankings.length === 0"
      @select="handleSelect"
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
      v-else-if="rankingsStore.rankings.length > 0 && rankingsStore.total > 0"
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
