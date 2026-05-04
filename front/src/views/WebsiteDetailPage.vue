<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useRankingsStore } from '@/stores/rankings'
import WebsiteHeader from '@/components/website/WebsiteHeader.vue'
import SubmissionList from '@/components/website/SubmissionList.vue'

const route = useRoute()
const router = useRouter()
const rankingsStore = useRankingsStore()

onMounted(() => {
  const id = route.params.id as string
  if (id) {
    rankingsStore.loadWebsiteDetail(id)
  }
})

watch(
  () => route.params.id,
  (newId) => {
    if (newId && typeof newId === 'string') {
      rankingsStore.loadWebsiteDetail(newId)
    }
  },
)
</script>

<template>
  <div class="website-detail-page">
    <!-- Back -->
    <button class="btn btn--ghost back-btn" @click="router.push('/rankings')">
      &larr; 返回排行榜
    </button>

    <!-- Error -->
    <div v-if="rankingsStore.websiteError" class="alert alert--error mb-4">
      <span>{{ rankingsStore.websiteError }}</span>
      <button class="btn btn--sm btn--ghost" @click="rankingsStore.loadWebsiteDetail(route.params.id as string)">
        重试
      </button>
    </div>

    <!-- Header -->
    <WebsiteHeader
      :website="rankingsStore.websiteDetail"
      :loading="rankingsStore.websiteLoading"
    />

    <!-- Submissions -->
    <h3 class="section-title">提交记录</h3>
    <SubmissionList
      :submissions="rankingsStore.websiteSubmissions"
      :loading="rankingsStore.websiteLoading"
    />
  </div>
</template>

<style scoped>
.website-detail-page {
  max-width: 900px;
  margin: 0 auto;
}

.back-btn {
  margin-bottom: var(--space-4);
}

.section-title {
  margin: var(--space-6) 0 var(--space-4);
}
</style>
