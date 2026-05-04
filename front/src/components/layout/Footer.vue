<script setup lang="ts">
const year = new Date().getFullYear()
</script>

<template>
  <footer class="footer">
    <div class="footer-inner">
      <p class="footer-title">LLM编码跑分</p>
      <p class="footer-desc">AI编码能力测试与排行系统</p>
      <p class="footer-disclaimer">测试结果仅供参考，不构成对任何AI服务的最终评价</p>
      <p class="footer-copy">&copy; {{ year }} LLM编码跑分</p>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  border-top: 1px solid var(--color-border);
  background: var(--color-white);
  margin-top: var(--space-16);
}

.footer-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) var(--space-4);
  text-align: center;
}

.footer-title {
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--space-1);
}

.footer-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-3);
}

.footer-disclaimer {
  font-size: var(--font-size-xs);
  color: var(--color-gray-400);
  margin-bottom: var(--space-4);
}

.footer-copy {
  font-size: var(--font-size-xs);
  color: var(--color-gray-400);
}
</style>
