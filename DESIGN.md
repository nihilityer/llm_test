# 编程模型测试跑分统计排行网站 - 架构设计文档

## 项目概述

快速验证 AI 服务提供商实际编码能力的测试与排行系统。用户打开网站即可输入 API 信息、运行测试、查看结果。支持 GitHub OAuth 和匿名 Turnstile 两种提交身份，排名基于网站所有提交的**加权平均分**。

**核心原则**：尽可能减少与 Cloudflare Worker 后端的交互，降低 Worker 调用费用。AI 调用使用各家官方 SDK，由浏览器直连目标 API。

---

## 1. 页面设计

### 1.1 路由表

| 路由 | 页面 | 说明 |
|------|------|------|
| `/` | **快速测试**（首页） | 打开即用的测试入口 |
| `/rankings` | 排行榜 | 通过导航栏打开 |
| `/website/:id` | 网站详情 | 查看某网站的提交记录 |
| `/auth/callback` | OAuth 回调 | GitHub 登录回调 |
| `/about` | 关于 | 测试方法与评分规则 |

### 1.2 各页面详细说明

#### 首页 `/` — 快速测试入口（核心页面）

打开网站直接看到测试配置，零门槛开始测试。

- **API 配置区**:
  - 接口风格选择：OpenAI / Anthropic / Gemini（Tab 切换）
  - **端点 URL 输入框**（核心输入，从此 URL 自动提取域名匹配网站）
  - API Key 输入框（密码类型，红色提示"仅存浏览器内存，不会上传"）
  - 模型名称输入（可选）
- **测试用例区**:
  - 当前测试套件版本号与简要说明
  - 测试用例列表折叠面板（名称、描述、难度标签）
  - 醒目的 **"开始测试"** 按钮
- **测试执行区**（开始后展开）:
  - 实时进度条 + 用例状态卡片
  - 取消按钮
- **结果展示区**（测试完成后）:
  - 总分大字展示 + 各维度雷达图
  - 每题详细结果可展开
  - 提交按钮组：已登录直接提交 / GitHub登录后提交 / 匿名提交(Turnstile)
- **匹配网站信息**（提交前自动显示）:
  - 从端点 URL 提取的域名，显示"将提交到：xxx.com"
  - 用户无需手动选择或创建网站

**调用的后端 API**：仅提交时 1 次 `POST /api/submissions`（其余全在浏览器完成）

#### 排行榜 `/rankings`

- 网站排行表格：排名、名称、域名、平均分、提交次数、最高/最低分、最近测试
- API 风格筛选 Tab
- 搜索框
- 点击行进入网站详情

**调用 API**：`GET /api/rankings` 进入页面时 1 次

#### 网站详情 `/website/:id`

- 网站信息头（名称、域名、平均分、提交次数）
- 所有提交列表（提交者、得分、各维度、风格、版本、时间、OAuth/匿名标识）
- 版本筛选

**调用 API**：`GET /api/rankings?website_id=:id` 进入页面时 1 次

#### OAuth 回调 `/auth/callback`

- 从 URL 获取 `?code=`，调用后端交换 token，存储 JWT，跳回首页

**调用 API**：`POST /api/auth/github/callback` 1 次

#### 关于 `/about`

- 纯静态页面，说明评分维度、权重公式、版本历史、匿名/登录权重差异

**调用 API**：零

---

## 2. API 接口设计（精简为 5 个端点）

> 基础路径: `/api`

### `GET /api/auth/github/login`
- **说明**: 302 重定向到 GitHub OAuth 授权页
- **响应**: 302

### `POST /api/auth/github/callback`
- **说明**: 用 OAuth code 交换 JWT
- **请求**: `{ "code": "github_oauth_code" }`
- **响应**: `{ "token": "jwt", "user": { "id", "login", "avatar_url" } }`

### `POST /api/auth/anonymous`
- **说明**: 验证 Turnstile token，返回匿名会话 token
- **请求**: `{ "turnstile_token": "cf_response" }`
- **响应**: `{ "token": "anon_token" }`
- **逻辑**: 验证 Turnstile → IP hash → 频率检查 → 签发 24h 匿名 token

### `POST /api/submissions`
- **说明**: 提交跑分结果。后端根据 `domain` 自动匹配或创建网站。OAuth 和匿名统一入口。
- **认证**: `Authorization: Bearer <jwt>` 或 `X-Anonymous-Token: <anon_token>`
- **请求体**:
```json
{
  "domain": "api.example.com",
  "test_suite_version": "v1",
  "api_style": "openai",
  "endpoint_hash": "sha256_of_endpoint_url",
  "total_score": 85.5,
  "dimension_scores": {
    "correctness": 40.0,
    "efficiency": 25.0,
    "code_style": 12.0,
    "tool_use": 8.5
  },
  "test_results": [
    {
      "test_case_id": "tc_001",
      "passed": true,
      "response_time_ms": 1234,
      "tokens_used": 500,
      "output_preview": "..."
    }
  ]
}
```
- **响应**: `{ "id": "sub_id", "website_id": "xxx", "website_name": "api.example.com" }`
- **后端逻辑**:
  1. 验证身份 → 确定 submitter_type (oauth/anonymous)
  2. **网站匹配**: 在 `websites` 表的 `domains` JSON 数组中搜索 `domain`，未匹配则自动创建网站（name=domain, domains=[domain]）
  3. OAuth: Upsert (user_id + website_id 唯一)
  4. 匿名: 检查 IP 频率限制 → Upsert (ip_hash + website_id 唯一)
- **管理员操作**: 后续通过 D1 直接修改 website 的 name 和 domains 字段（将同一服务的多个域名合并）

### `GET /api/rankings`
- **说明**: 排行榜 / 网站列表 / 网站详情 三合一查询
- **查询参数**:
  - `website_id` (可选): 查询单个网站的提交详情
  - `search` (可选): 按网站名称/域名搜索
  - `style` (可选): 按 API 风格筛选
  - `limit` (默认 50), `offset` (默认 0)
- **不带 website_id 时的响应**（排行榜/网站列表）:
```json
{
  "rankings": [
    {
      "rank": 1,
      "website_id": "xxx",
      "website_name": "Example",
      "domains": ["api.example.com"],
      "avg_score": 78.3,
      "submission_count": 12,
      "max_score": 92.0,
      "min_score": 55.0,
      "last_tested_at": "2026-05-04T10:00:00Z"
    }
  ],
  "total": 150
}
```
- **带 website_id 时的响应**（网站详情）:
```json
{
  "website": {
    "id": "xxx",
    "name": "Example",
    "domains": ["api.example.com"],
    "avg_score": 78.3,
    "submission_count": 12
  },
  "submissions": [
    {
      "id": "sub_xxx",
      "submitter_type": "oauth",
      "submitter_name": "username",
      "submitter_avatar": "...",
      "total_score": 85.5,
      "dimension_scores": {...},
      "test_suite_version": "v1",
      "api_style": "openai",
      "created_at": "..."
    }
  ]
}
```
- **排名计算**（后端聚合逻辑）:
  1. 从环境变量读取 `VERSION_WEIGHTS` (JSON) 和 `SUBMITTER_WEIGHT`
  2. 每条提交: `加权分 = total_score × version_weight × submitter_weight`
  3. 网站 `avg_score = AVG(所有加权分)`
  4. 按 avg_score 降序排列

---

## 3. 数据库设计（3 张表）

### `users`
```sql
CREATE TABLE users (
    id          TEXT PRIMARY KEY,
    github_id   INTEGER UNIQUE NOT NULL,
    login       TEXT NOT NULL,
    avatar_url  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX idx_users_github_id ON users(github_id);
```

### `websites`
```sql
CREATE TABLE websites (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    domains     TEXT NOT NULL DEFAULT '[]',   -- JSON 数组
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### `submissions` — 核心表
```sql
CREATE TABLE submissions (
    id                  TEXT PRIMARY KEY,
    submitter_type      TEXT NOT NULL CHECK(submitter_type IN ('oauth','anonymous')),
    user_id             TEXT,
    ip_hash             TEXT,
    website_id          TEXT NOT NULL REFERENCES websites(id),
    test_suite_version  TEXT NOT NULL,
    api_style           TEXT NOT NULL,
    endpoint_hash       TEXT NOT NULL,
    total_score         REAL NOT NULL,
    dimension_scores    TEXT NOT NULL,        -- JSON
    test_results        TEXT NOT NULL,        -- JSON
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- OAuth: 同一用户+网站唯一
CREATE UNIQUE INDEX idx_oauth_uniq ON submissions(user_id, website_id)
    WHERE submitter_type = 'oauth';
-- 匿名: 同一IP+网站唯一
CREATE UNIQUE INDEX idx_anon_uniq ON submissions(ip_hash, website_id)
    WHERE submitter_type = 'anonymous';
-- 查询
CREATE INDEX idx_sub_website ON submissions(website_id);
CREATE INDEX idx_sub_ip_created ON submissions(ip_hash, created_at);
```

> 测试用例内容不存数据库，作为静态文件打包在前端项目中。版本权重和提交者权重均通过环境变量配置。

---

## 4. AI 调用方式（SDK）

使用各家官方 npm SDK，配置自定义 baseURL 以支持第三方端点。

### 4.1 依赖

```bash
npm install openai @anthropic-ai/sdk @google/generative-ai
```

### 4.2 SDK 客户端初始化

```typescript
// OpenAI 风格（兼容大多数第三方）
import OpenAI from 'openai'
const client = new OpenAI({
  baseURL: userEndpoint,    // 用户输入的自定义端点
  apiKey: userApiKey,
  dangerouslyAllowBrowser: true,  // 浏览器端使用
})

// Anthropic 风格
import Anthropic from '@anthropic-ai/sdk'
const client = new Anthropic({
  baseURL: userEndpoint,
  apiKey: userApiKey,
})

// Gemini 风格
import { GoogleGenAI } from '@google/generative-ai'
const client = new GoogleGenAI({
  baseURL: userEndpoint,  // 自定义端点
  apiKey: userApiKey,
})
```

### 4.3 测试执行流程

```typescript
// 测试套件定义（静态 TS 文件，打包在前端）
const TEST_SUITE = {
  version: 'v1',
  name: '编码能力综合测试 v1',
  dimensions: [
    { key: 'correctness', name: '正确性', maxScore: 40 },
    { key: 'efficiency', name: '运行效率', maxScore: 25 },
    { key: 'code_style', name: '代码风格', maxScore: 20 },
    { key: 'tool_use', name: '工具使用', maxScore: 15 },
  ],
  testCases: [
    {
      id: 'tc_001',
      title: '实现二分查找',
      description: '给定有序数组和目标值，返回索引或 -1',
      prompt: 'Implement binary search function...',
      expectedOutput: '...',
      maxTokens: 500,
    },
    // ... 更多用例
  ],
}

async function runTest(apiConfig: ApiConfig, testSuite: TestSuite): Promise<TestResult> {
  const client = createSdkClient(apiConfig)  // 按风格创建 SDK 客户端

  const results = []
  for (const testCase of testSuite.testCases) {
    const startTime = Date.now()

    // 用 SDK 发送请求（浏览器直连目标 API）
    const response = await client.chat.completions.create({  // 示例：OpenAI 风格
      model: apiConfig.model || 'default',
      messages: [{ role: 'user', content: testCase.prompt }],
      max_tokens: testCase.maxTokens,
    })

    const elapsed = Date.now() - startTime
    const output = extractCode(response)

    // 本地评估
    results.push({
      testCaseId: testCase.id,
      passed: evaluateCorrectness(output, testCase.expectedOutput),
      responseTimeMs: elapsed,
      tokensUsed: response.usage?.total_tokens || 0,
      outputPreview: output.slice(0, 200),
    })
  }

  // 计算各维度得分
  const scores = calculateScores(results, testSuite.dimensions)
  return { results, scores, totalScore: sum(scores) }
}
```

**关键点**：
- 使用 SDK 而非裸 fetch，正确处理认证、重试、流式等
- 所有 LLM 调用从浏览器直连用户端点，Worker 零中转
- API Key 仅存浏览器内存（组件 state 中），不持久化
- 测试套件定义是前端静态文件，更新测试标准 = 部署新前端版本
- **网站匹配**: 前端从端点 URL 提取域名（`new URL(endpoint).hostname`），随结果一起提交；后端自动匹配或创建网站

---

## 5. 后端调用次数统计

一个典型用户完成一次完整测试流程产生的 Worker 调用：

| 操作 | Worker 调用数 | 调用时机 |
|------|:---:|------|
| 打开首页 | 0 | 页面静态资源来自 CF Pages |
| 搜索已有网站 | 1 | `GET /api/rankings?search=xxx` |
| 提交结果 | 1 | `POST /api/submissions` |
| 浏览排行榜 | 1 | `GET /api/rankings` |
| 查看网站详情 | 1 | `GET /api/rankings?website_id=xxx` |

**核心路径（打开→测试→提交）：最少仅 1-2 次 Worker 调用。**
排行榜浏览按需触发，不自动加载。

---

## 6. 前端架构

### 6.1 组件树

```
App.vue
├── NavBar.vue
│   ├── Logo (链接到 /)
│   ├── NavLinks: 排行榜(/rankings) | 关于(/about)
│   └── AuthSection (GitHub登录按钮 / 用户头像)
│
├── <RouterView>
│   ├── HomePage.vue              # `/` 快速测试入口
│   │   ├── ApiConfigPanel.vue    # API风格Tab + 端点URL + Key
│   │   ├── TestCaseAccordion.vue # 用例列表
│   │   ├── TestRunner.vue        # 进度 + 实时状态
│   │   ├── ScoreBoard.vue        # 总分 + 雷达图
│   │   └── SubmitPanel.vue       # 显示匹配域名 + 提交按钮 + Turnstile
│   │
│   ├── RankingsPage.vue          # `/rankings`
│   │   ├── StyleFilter.vue
│   │   ├── SearchBar.vue
│   │   └── RankingTable.vue
│   │
│   ├── WebsiteDetailPage.vue     # `/website/:id`
│   │   ├── WebsiteHeader.vue
│   │   └── SubmissionList.vue
│   │
│   ├── AuthCallbackPage.vue      # `/auth/callback`
│   └── AboutPage.vue             # `/about`
│
└── Footer.vue
```

### 6.2 Pinia Store

```typescript
// useAuthStore — 认证
{
  user: User | null,
  token: string | null,
  isLoggedIn: boolean,
  loginWithGithub(), handleCallback(code), logout(),
}

// useTestStore — 测试运行（纯前端，不调后端）
{
  apiConfig: { style, endpoint, apiKey, model },
  domain: string,           // 从端点 URL 自动提取的域名
  results: TestResult[],
  isRunning: boolean,
  progress: number,
  scores: DimensionScores,
  totalScore: number,

  extractDomain(endpoint),  // 从 URL 提取域名
  startTest(cases),         // 开始运行（全部在前端执行）
  cancelTest(),
}

// useRankingsStore — 排行榜（需要时调后端）
{
  rankings: RankingEntry[],
  fetchRankings(filters),
  fetchWebsiteDetail(id),
}
```

---

## 7. 评分引擎（前端计算）

### 维度与权重

| 维度 | 满分 | 计算方式 |
|------|:---:|------|
| correctness 正确性 | 40 | 通过数 / 总用例数 × 40 |
| efficiency 效率 | 25 | 基于平均响应时间和 token 效率，上限 25 |
| code_style 代码风格 | 20 | 命名、注释、结构规则检查，上限 20 |
| tool_use 工具使用 | 15 | function call 正确使用，上限 15 |

### 排名加权（后端计算）

```
单条加权分 = total_score × version_weight × submitter_weight
网站平均分 = AVG(该网站所有提交的加权分)
```

| 权重类型 | 默认值 | 环境变量 |
|------|:---:|------|
| OAuth 提交者 | 1.0 | `SUBMITTER_WEIGHT_OAUTH` |
| 匿名提交者 | 0.7 | `SUBMITTER_WEIGHT_ANONYMOUS` |
| 各版本权重 | `{"v1":0.7,"v2":1.0}` | `VERSION_WEIGHTS` (JSON 字符串) |
| 当前测试套件版本 | `v1` | `CURRENT_TEST_SUITE` |

---

## 8. 认证流

### GitHub OAuth
```
点击登录 → GET /api/auth/github/login → 302 GitHub
→ 授权 → /auth/callback?code=xxx
→ POST /api/auth/github/callback → 返回 JWT → localStorage
```

### 匿名 Turnstile
```
测试完成 → 点击匿名提交 → Turnstile 弹窗
→ POST /api/auth/anonymous → 返回 24h 匿名 token
→ 用匿名 token 调 POST /api/submissions
```

**防滥用**：同一 IP hash 同网站 24h 最多 3 次，全站 24h 最多 10 次。

---

## 9. 部署

| 组件 | 平台 | 配置 |
|------|------|------|
| 前端 | Cloudflare Pages | `front/` 目录, `npm run build` |
| 后端 | Cloudflare Workers | `api/`, Rust→WASM |
| 数据库 | Cloudflare D1 | `rank-data` |

### 环境变量（wrangler.toml vars）

```toml
[vars]
FRONTEND_URL = "https://code-llm-test.pages.dev"
TURNSTILE_SITE_KEY = "1x00000000000000000000AA"
CURRENT_TEST_SUITE = "v1"
VERSION_WEIGHTS = "{\"v1\":1.0}"
SUBMITTER_WEIGHT_OAUTH = "1.0"
SUBMITTER_WEIGHT_ANONYMOUS = "0.7"
```

### Secrets

| Secret 名 | 说明 |
|-----------|------|
| `JWT_SECRET` | JWT 签名密钥 (HS256) |
| `GITHUB_CLIENT_ID` | GitHub OAuth App Client ID |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App Client Secret |
| `TURNSTILE_SECRET_KEY` | Cloudflare Turnstile Secret Key |

> 修改权重只需更新环境变量并重新部署 Worker，无需改动数据库。添加新测试版本时更新 `VERSION_WEIGHTS` 和 `CURRENT_TEST_SUITE` 即可。

---

## 10. 验证方式

1. **前端**: `npm run dev` → 打开首页 → 输入测试 API 配置 → 运行测试 → 验证评分
2. **后端**: `wrangler dev` → curl 测试 5 个端点
3. **排名算法**: 提交不同权重的数据，验证 AVG 计算和排序