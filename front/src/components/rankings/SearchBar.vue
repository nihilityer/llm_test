<script setup lang="ts">
import { ref, watch } from 'vue'

const model = defineModel<string>('modelValue', { default: '' })
const input = ref(model.value)

let timeout: ReturnType<typeof setTimeout> | null = null

watch(input, (val) => {
  if (timeout) clearTimeout(timeout)
  timeout = setTimeout(() => {
    model.value = val
  }, 300)
})

watch(model, (val) => {
  if (val !== input.value) {
    input.value = val
  }
})

function clear() {
  input.value = ''
  model.value = ''
}
</script>

<template>
  <div class="search-bar">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.35-4.35" />
    </svg>
    <input
      v-model="input"
      class="search-input"
      type="text"
      placeholder="搜索网站名称或域名..."
    />
    <button v-if="input" class="clear-btn" @click="clear" aria-label="清除搜索">&times;</button>
  </div>
</template>

<style scoped>
.search-bar {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 18px;
  height: 18px;
  color: var(--color-gray-400);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 36px 10px 40px;
  font-size: var(--font-size-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-white);
  outline: none;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.search-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-bg);
}

.search-input::placeholder {
  color: var(--color-gray-400);
}

.clear-btn {
  position: absolute;
  right: 8px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: var(--color-gray-200);
  color: var(--color-gray-500);
  border-radius: var(--radius-full);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  transition: background var(--transition-fast);
}

.clear-btn:hover {
  background: var(--color-gray-300);
}
</style>
