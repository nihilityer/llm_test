<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const mobileMenuOpen = ref(false)

function isActive(path: string): boolean {
  return route.path === path
}

function navigate(path: string) {
  mobileMenuOpen.value = false
  router.push(path)
}
</script>

<template>
  <nav class="navbar">
    <div class="navbar-inner">
      <!-- Logo -->
      <a href="/" class="navbar-logo" @click.prevent="navigate('/')">LLM跑分</a>

      <!-- Nav links - desktop -->
      <div class="navbar-links">
        <button
          class="nav-link"
          :class="{ 'nav-link--active': isActive('/rankings') }"
          @click="navigate('/rankings')"
        >
          排行榜
        </button>
        <button
          class="nav-link"
          :class="{ 'nav-link--active': isActive('/about') }"
          @click="navigate('/about')"
        >
          关于
        </button>
        <a
          class="nav-link nav-link--github"
          href="https://github.com/nihilityer/llm_test"
          target="_blank"
          rel="noopener noreferrer"
          title="GitHub"
        >
          <svg class="github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
          </svg>
          <span class="nav-github-label">GitHub</span>
        </a>
      </div>

      <!-- Auth section -->
      <div class="navbar-auth">
        <template v-if="authStore.isLoggedIn && authStore.user">
          <img
            v-if="authStore.user.avatar_url"
            :src="authStore.user.avatar_url"
            class="nav-avatar"
            alt=""
          />
          <span class="nav-username">{{ authStore.user.login }}</span>
          <button class="btn btn--ghost btn--sm" @click="authStore.logout()">退出</button>
        </template>
        <button v-else class="btn btn--ghost btn--sm" @click="authStore.loginWithGithub()">
          GitHub 登录
        </button>
      </div>

      <!-- Mobile menu toggle -->
      <button class="mobile-toggle" @click="mobileMenuOpen = !mobileMenuOpen" aria-label="菜单">
        <span :class="{ 'mobile-bar--open': mobileMenuOpen }" />
        <span :class="{ 'mobile-bar--open': mobileMenuOpen }" />
        <span :class="{ 'mobile-bar--open': mobileMenuOpen }" />
      </button>
    </div>

    <!-- Mobile menu -->
    <div v-if="mobileMenuOpen" class="mobile-menu">
      <button
        class="mobile-link"
        :class="{ 'mobile-link--active': isActive('/rankings') }"
        @click="navigate('/rankings')"
      >
        排行榜
      </button>
      <button
        class="mobile-link"
        :class="{ 'mobile-link--active': isActive('/about') }"
        @click="navigate('/about')"
      >
        关于
      </button>
      <a
        class="mobile-link"
        href="https://github.com/nihilityer/llm_test"
        target="_blank"
        rel="noopener noreferrer"
      >
        <svg class="github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
        </svg>
        <span class="nav-github-label">GitHub</span>
      </a>
      <div class="mobile-auth">
        <template v-if="authStore.isLoggedIn && authStore.user">
          <span style="font-size: var(--font-size-sm)">{{ authStore.user.login }}</span>
          <button class="btn btn--ghost btn--sm" @click="authStore.logout()">退出</button>
        </template>
        <button v-else class="btn btn--primary btn--sm" @click="authStore.loginWithGithub()">
          GitHub 登录
        </button>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.navbar {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--color-border);
}

.navbar-inner {
  display: flex;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 var(--space-4);
  height: 56px;
}

.navbar-logo {
  font-size: var(--font-size-xl);
  font-weight: 700;
  color: var(--color-primary);
  margin-right: var(--space-8);
  text-decoration: none;
  white-space: nowrap;
}

.navbar-logo:hover {
  color: var(--color-primary-dark);
}

.navbar-links {
  display: flex;
  gap: var(--space-1);
}

.nav-link {
  padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  background: none;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-family: var(--font-family);
}

.nav-link:hover {
  color: var(--color-text);
  background: var(--color-gray-100);
}

.nav-link--active {
  color: var(--color-primary);
  background: var(--color-primary-bg);
}

.nav-link--github {
  display: inline-flex !important;
  align-items: center;
  gap: var(--space-1);
  text-decoration: none;
}

.github-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.nav-github-label {
  font-size: var(--font-size-sm);
}

.navbar-auth {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-left: auto;
}

.nav-avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  object-fit: cover;
}

.nav-username {
  font-size: var(--font-size-sm);
  font-weight: 500;
}

/* Mobile */
.mobile-toggle {
  display: none;
  flex-direction: column;
  gap: 5px;
  background: none;
  border: none;
  cursor: pointer;
  padding: var(--space-2);
}

.mobile-toggle span {
  display: block;
  width: 20px;
  height: 2px;
  background: var(--color-text);
  transition: transform var(--transition-fast);
}

.mobile-bar--open:nth-child(1) {
  transform: translateY(7px) rotate(45deg);
}

.mobile-bar--open:nth-child(2) {
  opacity: 0;
}

.mobile-bar--open:nth-child(3) {
  transform: translateY(-7px) rotate(-45deg);
}

.mobile-menu {
  padding: var(--space-4);
  border-top: 1px solid var(--color-border);
  background: var(--color-white);
}

.mobile-link {
  display: block;
  width: 100%;
  padding: var(--space-3) var(--space-4);
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--color-text);
  background: none;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  text-align: left;
  font-family: var(--font-family);
}

.mobile-link--active {
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.mobile-auth {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  margin-top: var(--space-2);
  border-top: 1px solid var(--color-border);
}

@media (max-width: 768px) {
  .navbar-links,
  .navbar-auth {
    display: none;
  }

  .mobile-toggle {
    display: flex;
    margin-left: auto;
  }
}
</style>
