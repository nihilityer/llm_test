<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTestStore } from '@/stores/test'
import { API_STYLES } from '@/types/api'

const testStore = useTestStore()

const showKey = ref(false)

function toggleShowKey() {
  showKey.value = !showKey.value
}

const displayDomain = computed(() => {
  const d = testStore.domain
  return d ? `检测到域名: ${d}` : ''
})
</script>

<template>
  <div class="card">
    <h3 class="card__header">API 配置</h3>

    <!-- Style tabs -->
    <label class="field-label">接口风格</label>
    <div class="tabs">
      <button
        v-for="style in API_STYLES"
        :key="style.value"
        class="tab"
        :class="{ 'tab--active': testStore.apiConfig.style === style.value }"
        @click="testStore.setApiConfig({ style: style.value })"
      >
        {{ style.label }}
      </button>
    </div>

    <!-- Endpoint -->
    <div class="field">
      <label class="field-label" for="endpoint">端点 URL</label>
      <input
        id="endpoint"
        class="input"
        :class="{ 'input--error': testStore.apiConfig.endpoint.trim() && !testStore.domain }"
        type="text"
        placeholder="https://api.example.com/v1"
        :value="testStore.apiConfig.endpoint"
        :disabled="testStore.runState === 'running'"
        @input="testStore.setApiConfig({ endpoint: ($event.target as HTMLInputElement).value })"
      />
      <span v-if="displayDomain" class="field-hint">{{ displayDomain }}</span>
      <span v-else-if="testStore.apiConfig.endpoint.trim() && !testStore.domain" class="field-error">
        请输入有效的 URL 地址
      </span>
    </div>

    <!-- API Key -->
    <div class="field">
      <label class="field-label" for="apiKey">API Key</label>
      <div class="input-with-btn">
        <input
          id="apiKey"
          class="input"
          :type="showKey ? 'text' : 'password'"
          placeholder="sk-..."
          :value="testStore.apiConfig.apiKey"
          :disabled="testStore.runState === 'running'"
          @input="testStore.setApiConfig({ apiKey: ($event.target as HTMLInputElement).value })"
        />
        <button class="btn btn--ghost btn--sm toggle-btn" type="button" @click="toggleShowKey">
          {{ showKey ? '隐藏' : '显示' }}
        </button>
      </div>
      <span class="field-warning">仅存浏览器内存，不会上传</span>
    </div>

    <!-- Model -->
    <div class="field">
      <label class="field-label" for="model">模型名称（可选）</label>
      <input
        id="model"
        class="input"
        type="text"
        placeholder="gpt-4o / claude-sonnet-4 / gemini-2.0-flash"
        :value="testStore.apiConfig.model"
        :disabled="testStore.runState === 'running'"
        @input="testStore.setApiConfig({ model: ($event.target as HTMLInputElement).value })"
      />
    </div>

    <!-- Proxy mode toggle -->
    <div class="field">
      <label class="field-label">代理模式</label>
      <div class="proxy-toggle-row">
        <label class="toggle">
          <input
            type="checkbox"
            :checked="testStore.apiConfig.useProxy"
            :disabled="testStore.runState === 'running'"
            @change="testStore.setApiConfig({ useProxy: ($event.target as HTMLInputElement).checked })"
          />
          <span class="toggle-slider"></span>
        </label>
        <span class="toggle-label">
          {{ testStore.apiConfig.useProxy ? '通过服务器代理请求' : '直接从浏览器请求' }}
        </span>
      </div>
      <p v-if="testStore.apiConfig.useProxy" class="field-warning">
        API 密钥将通过服务器转发。密钥不会被存储，但会在传输过程中对服务器可见。
      </p>
      <p v-else class="field-hint">
        如果遇到跨域（CORS）错误，请启用代理模式。
      </p>
    </div>
  </div>
</template>

<style scoped>
.field {
  margin-top: var(--space-4);
}

.field-label {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: var(--space-1);
}

.field-hint {
  display: block;
  margin-top: var(--space-1);
  font-size: var(--font-size-xs);
  color: var(--color-primary);
}

.field-error {
  display: block;
  margin-top: var(--space-1);
  font-size: var(--font-size-xs);
  color: var(--color-error);
}

.field-warning {
  display: block;
  margin-top: var(--space-1);
  font-size: var(--font-size-xs);
  color: var(--color-error);
}

.input-with-btn {
  display: flex;
  gap: var(--space-2);
}

.input-with-btn .input {
  flex: 1;
}

.toggle-btn {
  flex-shrink: 0;
}

.proxy-toggle-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-1);
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  cursor: pointer;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background-color: var(--color-border);
  border-radius: 12px;
  transition: background-color 0.2s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: transform 0.2s;
}

.toggle input:checked + .toggle-slider {
  background-color: var(--color-primary);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

.toggle input:disabled + .toggle-slider {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
