<script setup lang="ts">
const year = new Date().getFullYear()
</script>

<template>
  <footer class="footer">
    <div class="footer-inner">
      <p class="footer-title">LLM跑分</p>
      <p class="footer-desc">AI能力简单测试与排行系统</p>
      <p class="footer-disclaimer">测试结果仅供参考，不构成对任何AI服务的最终评价</p>
      <p class="footer-feedback">
        有反馈或问题？欢迎前往
        <a
          class="footer-link"
          href="https://github.com/nihilityer/llm_test/issues"
          target="_blank"
          rel="noopener noreferrer"
        >GitHub 提交 Issue</a>
      </p>
      <p class="footer-copy">&copy; {{ year }} LLM跑分</p>
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

.footer-feedback {
  font-size: var(--font-size-xs);
  color: var(--color-gray-400);
  margin-bottom: var(--space-2);
}

.footer-link {
  color: var(--color-gray-400);
  text-decoration: none;
  transition: color 0.2s;
}

.footer-link:hover {
  color: var(--color-primary);
}

.footer-copy {
  font-size: var(--font-size-xs);
  color: var(--color-gray-400);
}
</style>
