<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useRankingsStore } from '@/stores/rankings'
import ModelHeader from '@/components/model/ModelHeader.vue'
import SubmissionList from '@/components/website/SubmissionList.vue'

const route = useRoute()
const router = useRouter()
const rankingsStore = useRankingsStore()

onMounted(() => {
  const id = route.params.id as string
  if (id) {
    rankingsStore.loadModelDetail(id)
  }
})

watch(
  () => route.params.id,
  (newId) => {
    if (newId && typeof newId === 'string') {
      rankingsStore.loadModelDetail(newId)
    }
  },
)
</script>

<template>
  <div class="model-detail-page">
    <!-- Back -->
    <button class="btn btn--ghost back-btn" @click="router.push('/rankings')">
      &larr; 返回排行榜
    </button>

    <!-- Error -->
    <div v-if="rankingsStore.modelError" class="alert alert--error mb-4">
      <span>{{ rankingsStore.modelError }}</span>
      <button class="btn btn--sm btn--ghost" @click="rankingsStore.loadModelDetail(route.params.id as string)">
        重试
      </button>
    </div>

    <!-- Header -->
    <ModelHeader
      :model="rankingsStore.modelDetail"
      :loading="rankingsStore.modelLoading"
    />

    <!-- Submissions -->
    <h3 class="section-title">提交记录</h3>
    <SubmissionList
      :submissions="rankingsStore.modelSubmissions"
      :loading="rankingsStore.modelLoading"
    />
  </div>
</template>

<style scoped>
.model-detail-page {
  max-width: 900px;
  margin: 0 auto;
}

.back-btn {
  margin-bottom: var(--space-4);
}

.mb-4 {
  margin-bottom: var(--space-4);
}

.section-title {
  font-size: var(--font-size-lg);
  font-weight: 600;
  margin: var(--space-8) 0 var(--space-4);
}
</style>
