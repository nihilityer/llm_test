# LLM Test Frontend — 前端应用

Vue 3 + TypeScript 单页应用，用户通过浏览器直接调用 AI API 进行模型能力测试。所有 LLM 请求直接在浏览器中发起，API Key 仅存内存，不上传服务器。

## 技术栈

- **框架**：Vue 3.5（Composition API，`<script setup>`）
- **语言**：TypeScript
- **构建工具**：Vite 8.0
- **状态管理**：Pinia 3.0
- **路由**：Vue Router 5.0
- **AI SDK**：openai、@anthropic-ai/sdk、@google/generative-ai
- **代码检查**：ESLint + Oxlint + Prettier

## 本地开发

```bash
cd front
npm install
npm run dev
```

默认在 `http://localhost:5173` 运行。开发模式下 API 请求自动代理到 `http://localhost:8787`（后端 Worker）。

## 构建

```bash
npm run build        # 生产构建，输出到 dist/
npm run type-check   # TypeScript 类型检查
npm run lint         # ESLint + Oxlint 检查
npm run format       # Prettier 格式化
```

## 项目结构

```
src/
├── api/              # 后端 API 调用封装（auth、client、models、rankings、submissions）
├── components/       # Vue 组件
│   ├── home/         # 首页组件（API配置、测试用例、运行器、评分板、提交面板）
│   ├── layout/       # 布局组件
│   ├── model/        # 模型相关组件
│   ├── rankings/     # 排行榜组件
│   └── website/      # 网站相关组件
├── engine/           # 核心测试引擎
│   ├── runner.ts     # 测试套件运行器
│   ├── scoring.ts    # 评分计算器
│   └── sdk-client.ts # LLM SDK 客户端初始化
├── router/           # Vue Router 路由配置（6 个页面路由）
├── stores/           # Pinia 状态管理（auth、rankings、test）
├── test-suite/       # 测试用例定义
│   └── v1/           # v1 测试套件（2 个用例）
├── types/            # TypeScript 类型定义
├── utils/            # 工具函数
└── views/            # 页面组件（首页、排行榜、详情、关于）
```

完整架构设计、组件树、评分引擎、Store 结构详见 **[DESIGN.md](../DESIGN.md)**。

## 添加新测试用例

测试用例以 JSON 格式存储在 `src/test-suite/v1/cases/` 下。添加新用例后，需在 `src/test-suite/v1/index.ts` 中注册。详见 **[CONTRIBUTING.md](../CONTRIBUTING.md)**。

## 更多信息

- 完整架构文档：[DESIGN.md](../DESIGN.md)
- 贡献指南：[CONTRIBUTING.md](../CONTRIBUTING.md)
- 后端 API：[api/README.md](../api/README.md)
