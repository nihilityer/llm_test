<script setup lang="ts">
</script>

<template>
  <div class="about-page">
    <h1 class="page-title">关于评分</h1>

    <!-- Scoring Dimensions -->
    <section class="card">
      <h2 class="card__header">评分维度</h2>
      <div class="about-table">
        <table class="table">
          <thead>
            <tr>
              <th>维度</th>
              <th>满分</th>
              <th>说明</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>正确性</strong></td>
              <td style="text-align: center">75</td>
              <td>测试用例的通过率，验证 AI 对常识推理和逻辑判断题的判断是否准确</td>
            </tr>
            <tr>
              <td><strong>运行效率</strong></td>
              <td style="text-align: center">25</td>
              <td>基于平均响应时间和 Token 使用量综合评估</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="text-sm text-secondary mt-4">
        当前 v1 为判断力综合测试，后续版本将扩展更多维度（如代码风格、工具使用等）。
      </p>
    </section>

    <!-- Calculation -->
    <section class="card mt-8">
      <h2 class="card__header">计算公式</h2>
      <div class="formula-block">
        <h3>维度得分</h3>
        <ul class="formula-list">
          <li><strong>正确性</strong> = (通过用例数 / 总用例数) × 75</li>
          <li><strong>运行效率</strong> = 响应时间评分(0-15) + Token效率评分(0-10)，上限25</li>
        </ul>
        <h3 style="margin-top: 16px">总分</h3>
        <p>总分 = 正确性 + 运行效率，上限 100 分</p>
      </div>
    </section>

    <!-- Ranking Algorithm -->
    <section class="card mt-8">
      <h2 class="card__header">排名算法</h2>
      <div class="formula-block">
        <p>每个网站的排名基于所有提交的<strong>加权平均分</strong>：</p>
        <div class="formula-display">
          <code>加权分 = 得分 x 版本权重 x 提交者权重</code>
          <br />
          <code>网站平均分 = AVG(该网站所有加权分)</code>
        </div>
        <h3 style="margin-top: 16px">权重配置</h3>
        <div class="about-table">
          <table class="table">
            <thead>
              <tr>
                <th>权重类型</th>
                <th>默认值</th>
                <th>说明</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>OAuth 提交者</td>
                <td style="text-align: center">1.0</td>
                <td>通过 GitHub 登录的用户提交</td>
              </tr>
              <tr>
                <td>匿名提交者</td>
                <td style="text-align: center">0.7</td>
                <td>通过 Turnstile 验证的匿名用户提交</td>
              </tr>
              <tr>
                <td>版本权重</td>
                <td style="text-align: center">可变</td>
                <td>不同测试套件版本有不同的权重系数</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p class="text-sm text-secondary mt-4">
          权重可以通过服务端环境变量动态调整，无需修改代码。
        </p>
      </div>
    </section>

    <!-- Version History -->
    <section class="card mt-8">
      <h2 class="card__header">版本历史</h2>
      <div class="about-table">
        <table class="table">
          <thead>
            <tr>
              <th>版本</th>
              <th>发布时间</th>
              <th>说明</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><span class="badge badge--warn">v1</span></td>
              <td>2026-05</td>
              <td>
                判断力综合测试，包含 2 道常识推理与逻辑判断测试题（洗车问题、大扫除制度）。
                <strong>当前测试用例仍在开发阶段</strong>，后续版本将扩展题目数量、难度和覆盖领域。
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Disclaimer -->
    <section class="card mt-8" style="background: var(--color-warning-bg); border-color: var(--color-warning-border)">
      <h2 class="card__header" style="color: var(--color-warning)">免责声明</h2>
      <p class="text-sm">
        本系统的测试结果仅供技术参考，不构成对任何 AI 服务的最终评价或背书。
        测试用例和评分标准可能无法全面反映 AI 模型的实际能力。
        用户输入的 API Key 仅存储在浏览器内存中，不会上传至服务器。
      </p>
    </section>
  </div>
</template>

<style scoped>
.about-page {
  max-width: 800px;
  margin: 0 auto;
}

.about-table {
  margin-top: var(--space-2);
}

.about-table table {
  font-size: var(--font-size-sm);
}

.formula-block {
  font-size: var(--font-size-sm);
  line-height: 1.8;
}

.formula-list {
  padding-left: var(--space-6);
  margin-top: var(--space-2);
}

.formula-display {
  background: var(--color-gray-100);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  margin-top: var(--space-3);
  font-size: var(--font-size-sm);
  line-height: 2;
}

.formula-display code {
  background: none;
  padding: 0;
}
</style>
