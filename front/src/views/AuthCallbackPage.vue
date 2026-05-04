<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const status = ref<'processing' | 'error'>('processing')
const errorMsg = ref('')

onMounted(async () => {
  const code = route.query.code as string | undefined

  if (!code) {
    router.replace('/')
    return
  }

  try {
    await authStore.handleCallback(code)
    const redirect = authStore.getLoginRedirect()
    router.replace(redirect)
  } catch (err) {
    status.value = 'error'
    errorMsg.value = err instanceof Error ? err.message : '登录失败，请重试'
    setTimeout(() => {
      router.replace('/')
    }, 3000)
  }
})
</script>

<template>
  <div class="auth-callback-page">
    <div v-if="status === 'processing'" class="callback-card">
      <div class="spinner" />
      <p>正在处理 GitHub 登录...</p>
    </div>
    <div v-else class="callback-card">
      <div class="alert alert--error">
        <span>{{ errorMsg }}</span>
      </div>
      <p class="text-secondary text-sm mt-4">即将返回首页...</p>
    </div>
  </div>
</template>

<style scoped>
.auth-callback-page {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
}

.callback-card {
  text-align: center;
  padding: var(--space-8);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-gray-200);
  border-top-color: var(--color-primary);
  border-radius: var(--radius-full);
  animation: spin 0.6s linear infinite;
  margin: 0 auto var(--space-4);
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
