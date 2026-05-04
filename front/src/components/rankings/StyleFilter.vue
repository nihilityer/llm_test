<script setup lang="ts">

const model = defineModel<string>('modelValue', { default: '' })

const tabs: { label: string; value: string }[] = [
  { label: '全部', value: '' },
  { label: 'OpenAI', value: 'openai' },
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Gemini', value: 'gemini' },
]

function select(value: string) {
  model.value = model.value === value ? '' : value
}
</script>

<template>
  <div class="tabs">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      class="tab"
      :class="{ 'tab--active': model === tab.value }"
      @click="select(tab.value)"
    >
      {{ tab.label }}
    </button>
  </div>
</template>
