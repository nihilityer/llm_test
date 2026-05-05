<script setup lang="ts">
import { useTestStore } from '@/stores/test'
import ApiConfigPanel from '@/components/home/ApiConfigPanel.vue'
import TestRunner from '@/components/home/TestRunner.vue'
import ScoreBoard from '@/components/home/ScoreBoard.vue'
import SubmitPanel from '@/components/home/SubmitPanel.vue'

const testStore = useTestStore()

function handleStartTest() {
  testStore.startTest()
}

function handleReset() {
  testStore.resetTest()
}
</script>

<template>
  <div class="home-page">
    <!-- Hero -->
    <section class="hero">
      <h1 class="hero-title">AI能力快速测试</h1>
      <p class="hero-subtitle">输入 API 配置，快速测试 AI 能力</p>
    </section>

    <!-- Tab Navigation -->
    <nav class="main-tabs">
      <template v-for="(tab, index) in testStore.tabs" :key="tab.id">
        <button
          class="main-tab"
          :class="{
            'main-tab--active': tab.status === 'active',
            'main-tab--completed': tab.status === 'completed',
            'main-tab--pending': tab.status === 'pending',
          }"
          :disabled="tab.status === 'pending'"
          @click="testStore.navigateToTab(tab.id)"
        >
          <span
            class="main-tab-indicator"
            :class="{
              'main-tab-indicator--active': tab.status === 'active',
              'main-tab-indicator--completed': tab.status === 'completed',
            }"
          >
            <template v-if="tab.status === 'completed'">&#10003;</template>
            <template v-else>{{ tab.id + 1 }}</template>
          </span>
          <span class="main-tab-label">{{ tab.label }}</span>
        </button>
        <div
          v-if="index < testStore.tabs.length - 1"
          class="main-tab-connector"
          :class="{
            'main-tab-connector--completed': tab.status === 'completed' || tab.status === 'active',
          }"
        />
      </template>
    </nav>

    <!-- Tab Content -->
    <div class="tab-content">
      <!-- Tab 0: API Configuration -->
      <div v-if="testStore.getTab(0).status !== 'pending'" v-show="testStore.getTab(0).status === 'active'" class="tab-panel">
        <section class="section">
          <ApiConfigPanel />
        </section>
        <section v-if="testStore.runState === 'idle'" class="section" style="text-align: center">
          <button
            class="btn btn--primary btn--lg"
            :disabled="!testStore.canStart"
            @click="handleStartTest"
          >
            {{ testStore.isConfigValid ? '开始测试' : '请完善 API 配置' }}
          </button>
        </section>
        <section
          v-if="testStore.runState === 'cancelled' || testStore.runState === 'error'"
          class="section"
          style="text-align: center"
        >
          <button class="btn btn--secondary" @click="handleReset">重新测试</button>
        </section>
      </div>

      <!-- Tab 1: Test Execution -->
      <div v-if="testStore.getTab(1).status !== 'pending'" v-show="testStore.getTab(1).status === 'active'" class="tab-panel">
        <section class="section">
          <TestRunner />
        </section>
        <section
          v-if="testStore.runState === 'cancelled' || testStore.runState === 'error'"
          class="section"
          style="text-align: center"
        >
          <button class="btn btn--secondary" @click="handleReset">重新测试</button>
        </section>
      </div>

      <!-- Tab 2: Results & Upload -->
      <div v-if="testStore.getTab(2).status !== 'pending'" v-show="testStore.getTab(2).status === 'active'" class="tab-panel">
        <section v-if="testStore.scores" class="section">
          <ScoreBoard />
        </section>
        <section v-if="testStore.scores" class="section">
          <SubmitPanel />
        </section>
        <section class="section" style="text-align: center">
          <button class="btn btn--secondary" @click="handleReset">重新测试</button>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-page {
  max-width: 720px;
  margin: 0 auto;
}

.hero {
  text-align: center;
  padding: var(--space-12) 0 var(--space-8);
}

.hero-title {
  font-size: var(--font-size-4xl);
  font-weight: 800;
  color: var(--color-text);
  letter-spacing: -0.02em;
}

.hero-subtitle {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
  margin-top: var(--space-3);
}

.section {
  margin-bottom: var(--space-6);
}

/* Main Tab Navigation */
.main-tabs {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin-bottom: var(--space-8);
  padding: var(--space-2);
}

.main-tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border: none;
  background: transparent;
  cursor: pointer;
  font-family: inherit;
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  transition: color 0.15s ease, opacity 0.15s ease;
  white-space: nowrap;
}

.main-tab:hover:not(:disabled) {
  color: var(--color-text);
}

.main-tab--active {
  color: var(--color-primary);
  font-weight: 600;
}

.main-tab--completed {
  color: var(--color-success);
}

.main-tab--pending {
  color: var(--color-gray-400);
  cursor: not-allowed;
  opacity: 0.6;
}

.main-tab-indicator {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 9999px;
  font-size: var(--font-size-xs);
  font-weight: 700;
  background: var(--color-gray-200);
  color: var(--color-white);
  transition: background 0.15s ease, transform 0.15s ease;
  flex-shrink: 0;
}

.main-tab-indicator--active {
  background: var(--color-primary);
  transform: scale(1.1);
}

.main-tab-indicator--completed {
  background: var(--color-success);
}

.main-tab-label {
  display: none;
}

@media (min-width: 480px) {
  .main-tab-label {
    display: inline;
  }
}

.main-tab-connector {
  width: 32px;
  height: 2px;
  background: var(--color-gray-200);
  transition: background 0.2s ease;
  flex-shrink: 0;
}

.main-tab-connector--completed {
  background: var(--color-success);
}

.tab-content {
  min-height: 200px;
}

.tab-panel {
  animation: tab-fade-in 0.2s ease;
}

@keyframes tab-fade-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
