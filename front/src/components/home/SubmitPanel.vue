<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useTestStore } from '@/stores/test'
import { useAuthStore } from '@/stores/auth'

const testStore = useTestStore()
const authStore = useAuthStore()

const turnstileContainer = ref<HTMLElement | null>(null)
const showTurnstile = ref(false)
let turnstileId: string | null = null

function showTurnstileWidget() {
  showTurnstile.value = true
}

function onTurnstileCallback(token: string) {
  handleAnonymousSubmit(token)
}

async function handleOAuthSubmit() {
  await testStore.submit()
}

async function handleAnonymousSubmit(turnstileToken: string) {
  try {
    await authStore.handleAnonymousAuth(turnstileToken)
    await testStore.submit()
  } catch {
    testStore.submitError = '获取匿名身份失败，请重试'
    testStore.submitState = 'error'
  }
}

// Load Turnstile widget when shown
let observer: MutationObserver | null = null

onMounted(() => {
  observer = new MutationObserver(() => {
    if (showTurnstile.value && turnstileContainer.value && !turnstileId) {
      const win = window as unknown as {
        turnstile?: { render: (el: string | HTMLElement, opts: object) => string }
      }
      if (win.turnstile) {
        turnstileId = win.turnstile.render(turnstileContainer.value, {
          sitekey: 'Ov23liiaOmaupTPIS5hq',
          callback: onTurnstileCallback,
          theme: 'light',
        })
      }
    }
  })
  observer.observe(document.body, { childList: true, subtree: true })
})

onUnmounted(() => {
  observer?.disconnect()
})
</script>

<template>
  <div v-if="testStore.runState === 'completed' && testStore.scores" class="card">
    <h3 class="card__header">提交结果</h3>

    <!-- Domain display -->
    <div class="domain-display">
      <svg
        class="domain-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path
          d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
        />
      </svg>
      <span
        >将提交到: <strong>{{ testStore.domain }}</strong></span
      >
    </div>

    <!-- LAN endpoint notice -->
    <div v-if="testStore.isLanEndpoint" class="alert alert--warning mt-4">
      检测到端点属于局域网地址，暂不支持提交测试结果。如需提交，请使用公网可访问的端点。
    </div>

    <!-- Already submitted warning -->
    <div v-else-if="testStore.submitResultData" class="alert alert--success mt-4">
      提交成功！已关联到网站 "{{ testStore.submitResultData.website_name }}"
    </div>

    <!-- Submit error -->
    <div v-else-if="testStore.submitState === 'error'" class="alert alert--error mt-4">
      <span>{{ testStore.submitError || '提交失败' }}</span>
      <button class="btn btn--sm btn--ghost" @click="testStore.resetSubmit()">重试</button>
    </div>

    <!-- Auth-based submit -->
    <div v-else class="submit-actions">
      <!-- Logged in with OAuth -->
      <template v-if="authStore.isLoggedIn && authStore.tokenType === 'oauth'">
        <div class="auth-info">
          <img
            v-if="authStore.user?.avatar_url"
            :src="authStore.user.avatar_url"
            class="auth-avatar"
            alt=""
          />
          <span>{{ authStore.user?.login || '已登录' }}</span>
        </div>
        <button
          class="btn btn--primary btn--lg"
          :disabled="testStore.submitState === 'submitting'"
          @click="handleOAuthSubmit"
        >
          {{ testStore.submitState === 'submitting' ? '提交中...' : '提交结果' }}
        </button>
      </template>

      <!-- Not logged in -->
      <template v-else>
        <button class="btn btn--secondary" @click="authStore.loginWithGithub()">
          GitHub 登录后提交
        </button>
        <span class="divider-text">或</span>
        <button v-if="!showTurnstile" class="btn btn--primary" @click="showTurnstileWidget">
          匿名提交
        </button>
        <div v-if="showTurnstile" ref="turnstileContainer" class="turnstile-container" />
      </template>
    </div>

    <!-- Submitting spinner for anonymous flow -->
    <div
      v-if="testStore.submitState === 'submitting' && authStore.tokenType === 'anonymous'"
      style="text-align: center; margin-top: var(--space-4); color: var(--color-text-secondary)"
    >
      提交中...
    </div>
  </div>
</template>

<style scoped>
.domain-display {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--color-primary-bg);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  margin-bottom: var(--space-4);
}

.domain-icon {
  width: 20px;
  height: 20px;
  color: var(--color-primary);
  flex-shrink: 0;
}

.submit-actions {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.auth-info {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.auth-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-full);
  object-fit: cover;
}

.divider-text {
  color: var(--color-gray-400);
  font-size: var(--font-size-sm);
}

.turnstile-container {
  margin-top: var(--space-3);
  min-height: 65px;
}
</style>
