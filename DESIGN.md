# 编程模型测试跑分统计排行网站 - 架构设计文档

## 项目概述

快速验证 AI 服务提供商实际编码能力的测试与排行系统。用户打开网站即可输入 API 信息、运行测试、查看结果。支持 GitHub OAuth 和匿名
Turnstile 两种提交身份，提供**网站排名**和**模型排名**两个维度的排行。

**核心原则**：尽可能减少与 Cloudflare Worker 后端的交互，降低 Worker 调用费用。AI 调用使用各家官方 SDK，由浏览器直连目标
API。

---

## 1. 页面设计

### 1.1 路由表

| 路由               | 页面           | 说明                 |
|------------------|--------------|--------------------|
| `/`              | **快速测试**（首页） | 打开即用的测试入口          |
| `/rankings`      | 排行榜          | 网站排行 / 模型排行 Tab 切换 |
| `/website/:id`   | 网站详情         | 查看某网站的提交记录         |
| `/model/:id`     | 模型详情         | 查看某模型的提交记录         |
| `/auth/callback` | OAuth 回调     | GitHub 登录回调        |
| `/about`         | 关于           | 测试方法与评分规则          |

### 1.2 各页面详细说明

#### 首页 `/` — 快速测试入口（核心页面）

打开网站直接看到测试配置，零门槛开始测试。

- **API 配置区**:
    - 接口风格选择：OpenAI / Anthropic / Gemini（Tab 切换）
    - **端点 URL 输入框**（核心输入，从此 URL 自动提取域名匹配网站）
    - API Key 输入框（密码类型，红色提示"仅存浏览器内存，不会上传"）
    - 模型名称输入框（必填，用于模型排名和权重计算）
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
- **匹配信息**（提交前自动显示）:
    - 从端点 URL 提取的域名 + 模型名，显示"将提交到：xxx.com / 模型：gpt-5"

**调用的后端 API**：仅提交时 1 次 `POST /api/submissions`（其余全在浏览器完成）

#### 排行榜 `/rankings`

- **Tab 切换**：网站排行 | 模型排行
- **网站排行**：排名、名称、域名、平均分、提交次数、最高/最低分、最近测试
- **模型排行**：排名、名称、别名、平均分、提交次数、覆盖网站数、最高/最低分、最近测试
- API 风格筛选 Tab
- 搜索框
- 点击行进入对应详情页

**调用 API**：`GET /api/rankings` 进入页面时 1 次

#### 网站详情 `/website/:id`

- 网站信息头（名称、域名、平均分、提交次数）
- 所有提交列表（提交者、**模型名称**、得分、各维度、风格、版本、时间、OAuth/匿名标识）

**调用 API**：`GET /api/rankings?website_id=:id` 进入页面时 1 次

#### 模型详情 `/model/:id`

- 模型信息头（名称、别名、平均分、提交次数、覆盖网站数）
- 所有提交列表（提交者、**网站名称**、得分、各维度、风格、版本、时间、OAuth/匿名标识）

**调用 API**：`GET /api/rankings?model_id=:id` 进入页面时 1 次

#### OAuth 回调 `/auth/callback`

- 从 URL 获取 `?code=`，调用后端交换 token，存储 JWT，跳回首页

#### 关于 `/about`

- 纯静态页面，说明评分维度、权重公式、版本历史、匿名/登录权重差异、模型/网站权重说明

---

## 2. API 接口设计（5 个端点）

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

### `POST /api/submissions`

- **说明**: 提交跑分结果。后端根据 `domain` 自动匹配或创建网站，根据 `model` 自动匹配或创建模型。
- **认证**: `Authorization: Bearer <jwt>` 或 `X-Anonymous-Token: <anon_token>`
- **请求体**:

```json
{
  "domain": "api.example.com",
  "model": "gpt-5",
  "test_suite_version": "v1",
  "api_style": "openai",
  "endpoint_hash": "sha256_of_endpoint_url",
  "total_score": 85.5,
  "dimension_scores": {
    "correctness": 75.0,
    "efficiency": 10.5
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

- **响应**: `{ "id": "sub_id", "website_id": "xxx", "website_name": "...", "model_id": "yyy", "model_name": "..." }`
- **后端逻辑**:
    1. 验证身份 → 确定 submitter_type (oauth/anonymous)
    2. **网站匹配**: 在 `websites` 表的 `domains` JSON 数组中搜索 `domain`，未匹配则自动创建网站
    3. **模型匹配**: 在 `models` 表及 `website_models` 别名中搜索 `model`，未匹配则自动创建模型
    4. **网站-模型关联**: 确保 `website_models` 中存在对应记录（默认权重 1.0）
    5. Upsert 提交（唯一键：user_id/ip_hash + website_id + model_id）
    6. 匿名: IP 频率限制（同网站 3 次/24h，全站 10 次/24h）

### `GET /api/rankings`

- **说明**: 排行榜 / 网站详情 / 模型详情 多合一查询
- **查询参数**:
    - `ranking_type` (可选): `"website"`（默认）| `"model"` — 排行类型
    - `website_id` (可选): 查询单个网站详情
    - `model_id` (可选): 查询单个模型详情
    - `search` (可选): 按名称/域名/别名搜索
    - `style` (可选): 按 API 风格筛选
    - `limit` (默认 50), `offset` (默认 0)

#### 网站排行响应 (`ranking_type=website` 或无此参数)：

```json
{
  "rankings": [
    {
      "rank": 1,
      "website_id": "xxx",
      "website_name": "Example",
      "domains": [
        "api.example.com"
      ],
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

#### 模型排行响应 (`ranking_type=model`)：

```json
{
  "rankings": [
    {
      "rank": 1,
      "model_id": "yyy",
      "model_name": "GPT-5",
      "aliases": [
        "gpt-5",
        "gpt-5-preview"
      ],
      "avg_score": 82.1,
      "submission_count": 45,
      "max_score": 95.0,
      "min_score": 60.0,
      "last_tested_at": "2026-05-04T10:00:00Z",
      "website_count": 8
    }
  ],
  "total": 30
}
```

#### 网站详情响应 (`website_id=xxx`)：

```json
{
  "website": {
    "id": "xxx",
    "name": "Example",
    "domains": [
      "api.example.com"
    ],
    "avg_score": 78.3,
    "submission_count": 12
  },
  "submissions": [
    {
      "id": "sub_xxx",
      "submitter_type": "oauth",
      "submitter_name": "username",
      "submitter_avatar": "...",
      "model_id": "yyy",
      "model_name": "GPT-5",
      "total_score": 85.5,
      "dimension_scores": {
        ...
      },
      "test_suite_version": "v1",
      "api_style": "openai",
      "created_at": "..."
    }
  ]
}
```

#### 模型详情响应 (`model_id=yyy`)：

```json
{
  "model": {
    "id": "yyy",
    "name": "GPT-5",
    "aliases": [
      "gpt-5",
      "gpt-5-preview"
    ],
    "avg_score": 82.1,
    "submission_count": 45,
    "website_count": 8
  },
  "submissions": [
    {
      "id": "sub_xxx",
      "submitter_type": "oauth",
      "submitter_name": "username",
      "submitter_avatar": "...",
      "model_id": "yyy",
      "model_name": "GPT-5",
      "total_score": 85.5,
      "dimension_scores": {
        ...
      },
      "test_suite_version": "v1",
      "api_style": "openai",
      "created_at": "..."
    }
  ]
}
```

### 排名计算公式

**网站排名**：

```
单条加权分 = total_score × version_weight × submitter_weight × model_weight
网站平均分 = AVG(该网站所有提交的加权分)
```

**模型排名**：

```
单条加权分 = total_score × version_weight × submitter_weight × website_weight
模型平均分 = AVG(该模型所有提交的加权分)
```

| 权重类型        |     默认值      | 配置来源                                |
|-------------|:------------:|-------------------------------------|
| OAuth 提交者   |     1.0      | 环境变量 `SUBMITTER_WEIGHT_OAUTH`       |
| 匿名提交者       |     0.7      | 环境变量 `SUBMITTER_WEIGHT_ANONYMOUS`   |
| 各版本权重       | `{"v1":1.0}` | 环境变量 `VERSION_WEIGHTS`              |
| 模型权重（网站排名时） |     1.0      | 数据库 `website_models.model_weight`   |
| 网站权重（模型排名时） |     1.0      | 数据库 `website_models.website_weight` |

> 版本权重和提交者权重从环境变量读取（全局生效）。模型权重和网站权重从数据库读取，可按网站-模型对精细调整，默认均为 1.0。

---

## 3. 数据库设计（5 张表）

### `users`

```sql
CREATE TABLE users
(
    id         TEXT PRIMARY KEY,
    github_id  INTEGER UNIQUE NOT NULL,
    login      TEXT           NOT NULL,
    avatar_url TEXT,
    created_at TEXT           NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT           NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX idx_users_github_id ON users (github_id);
```

### `websites`

```sql
CREATE TABLE websites
(
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    domains    TEXT NOT NULL DEFAULT '[]', -- JSON 数组
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### `models`

```sql
CREATE TABLE models
(
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    aliases    TEXT NOT NULL DEFAULT '[]', -- JSON 数组，模型在不同网站的常用名称
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### `website_models` — 网站与模型的多对多关联

```sql
CREATE TABLE website_models
(
    website_id     TEXT NOT NULL REFERENCES websites (id),
    model_id       TEXT NOT NULL REFERENCES models (id),
    model_weight   REAL NOT NULL DEFAULT 1.0, -- 网站排名时该模型的权重
    website_weight REAL NOT NULL DEFAULT 1.0, -- 模型排名时该网站的权重
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (website_id, model_id)
);
CREATE INDEX idx_wm_model ON website_models (model_id);
```

> 当提交时发现新的网站-模型组合，自动创建 `website_models` 记录，两个权重默认均为 1.0。管理员可通过 D1 直接修改权重值来调整排名。

### `submissions` — 核心表

```sql
CREATE TABLE submissions
(
    id                 TEXT PRIMARY KEY,
    submitter_type     TEXT NOT NULL CHECK (submitter_type IN ('oauth', 'anonymous')),
    user_id            TEXT,
    ip_hash            TEXT,
    website_id         TEXT NOT NULL REFERENCES websites (id),
    model_id           TEXT NOT NULL REFERENCES models (id),
    test_suite_version TEXT NOT NULL,
    api_style          TEXT NOT NULL,
    endpoint_hash      TEXT NOT NULL,
    total_score        REAL NOT NULL,
    dimension_scores   TEXT NOT NULL, -- JSON
    test_results       TEXT NOT NULL, -- JSON
    created_at         TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at         TEXT NOT NULL DEFAULT (datetime('now'))
);

-- OAuth: 同一用户+网站+模型唯一
CREATE UNIQUE INDEX idx_oauth_uniq ON submissions (user_id, website_id, model_id) WHERE submitter_type = 'oauth';
-- 匿名: 同一IP+网站+模型唯一
CREATE UNIQUE INDEX idx_anon_uniq ON submissions (ip_hash, website_id, model_id) WHERE submitter_type = 'anonymous';
-- 查询
CREATE INDEX idx_sub_website ON submissions (website_id);
CREATE INDEX idx_sub_model ON submissions (model_id);
CREATE INDEX idx_sub_ip_created ON submissions (ip_hash, created_at);
```

> **唯一键变更**：从 (user/ip, website_id) 改为 (user/ip, website_id, model_id)，同一用户可在同一网站用不同模型分别提交。

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
    baseURL: userEndpoint,
    apiKey: userApiKey,
    dangerouslyAllowBrowser: true,
})

// Anthropic 风格
import Anthropic from '@anthropic-ai/sdk'

const client = new Anthropic({
    baseURL: userEndpoint,
    apiKey: userApiKey,
})

// Gemini 风格
import {GoogleGenAI} from '@google/generative-ai'

const client = new GoogleGenAI({
    baseURL: userEndpoint,
    apiKey: userApiKey,
})
```

### 4.3 测试执行流程

```typescript
async function runTest(apiConfig: ApiConfig, testSuite: TestSuite): Promise<TestResult> {
    const client = createSdkClient(apiConfig)

    const results = []
    for (const testCase of testSuite.testCases) {
        const startTime = Date.now()
        const response = await client.chat.completions.create({
            model: apiConfig.model,
            messages: [{role: 'user', content: testCase.prompt}],
            max_tokens: testCase.maxTokens,
        })
        const elapsed = Date.now() - startTime
        const output = extractCode(response)

        results.push({
            testCaseId: testCase.id,
            passed: evaluateCorrectness(output, testCase.expectedOutput),
            responseTimeMs: elapsed,
            tokensUsed: response.usage?.total_tokens || 0,
            outputPreview: output.slice(0, 200),
        })
    }

    const scores = calculateScores(results, testSuite.dimensions)
    return {results, scores, totalScore: sum(scores)}
}
```

**关键点**：

- 使用 SDK 而非裸 fetch，正确处理认证、重试、流式等
- 所有 LLM 调用从浏览器直连用户端点，Worker 零中转
- API Key 仅存浏览器内存，不持久化
- 测试套件定义是前端静态文件，更新测试标准 = 部署新前端版本
- 用户输入模型名称随结果一起提交，后端自动匹配或创建模型

---

## 5. 后端调用次数统计

一个典型用户完成一次完整测试流程产生的 Worker 调用：

| 操作     | Worker 调用数 | 调用时机                               |
|--------|:----------:|------------------------------------|
| 打开首页   |     0      | 页面静态资源来自 CF Pages                  |
| 搜索已有网站 |     1      | `GET /api/rankings?search=xxx`     |
| 提交结果   |     1      | `POST /api/submissions`            |
| 浏览排行榜  |     1      | `GET /api/rankings`                |
| 查看网站详情 |     1      | `GET /api/rankings?website_id=xxx` |
| 查看模型详情 |     1      | `GET /api/rankings?model_id=yyy`   |

**核心路径（打开→测试→提交）：最少仅 1-2 次 Worker 调用。**

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
│   │   ├── ApiConfigPanel.vue    # API风格Tab + 端点URL + Key + 模型名
│   │   ├── TestCaseAccordion.vue # 用例列表
│   │   ├── TestRunner.vue        # 进度 + 实时状态
│   │   ├── ScoreBoard.vue        # 总分 + 雷达图
│   │   └── SubmitPanel.vue       # 显示匹配域名+模型名 + 提交按钮 + Turnstile
│   │
│   ├── RankingsPage.vue          # `/rankings`（网站/模型 Tab 切换）
│   │   ├── StyleFilter.vue
│   │   ├── SearchBar.vue
│   │   ├── RankingTable.vue      # 网站排行表格
│   │   └── ModelRankingTable.vue # 模型排行表格
│   │
│   ├── WebsiteDetailPage.vue     # `/website/:id`
│   │   ├── WebsiteHeader.vue
│   │   └── SubmissionList.vue    # 含模型列
│   │
│   ├── ModelDetailPage.vue       # `/model/:id`
│   │   ├── ModelHeader.vue
│   │   └── SubmissionList.vue    # 含网站列
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
            token:string | null,
        isLoggedIn
:
    boolean,
        tokenType
:
    'oauth' | 'anonymous' | null,
        loginWithGithub(), handleCallback(code), logout(), handleAnonymousAuth(token),
}

// useTestStore — 测试运行（纯前端，不调后端）
{
    apiConfig: {
        style, endpoint, apiKey, model
    }
,
    domain: string,
        results
:
    TestResult[],
        isRunning
:
    boolean,
        progress
:
    number,
        scores
:
    DimensionScores,
        totalScore
:
    number,
        startTest(), cancelTest(), submit(),  // submit 携带 model 字段
}

// useRankingsStore — 排行榜（双模式）
{
    rankingType: 'website' | 'model',
        // 网站模式
        rankings
:
    RankingEntry[],
        websiteDetail
:
    WebsiteSummary | null,
        websiteSubmissions
:
    SubmissionDetail[],
        // 模型模式
        modelRankings
:
    ModelRankingEntry[],
        modelDetail
:
    ModelSummary | null,
        modelSubmissions
:
    SubmissionDetail[],
        // 通用
        total, loading, error, filters, hasMore,
        setRankingType(type), loadRankings(), loadMore(),
        loadWebsiteDetail(id), loadModelDetail(id),
}
```

---

## 7. 评分引擎（前端计算）

### 维度与权重

| 维度              | 满分 | 计算方式               |
|-----------------|:--:|--------------------|
| correctness 正确性 | 75 | 通过数 / 总用例数 × 75    |
| efficiency 效率   | 25 | 基于平均响应时间和 token 效率 |

### 排名加权（后端计算）

**网站排名**：

```
单条加权分 = total_score × version_weight × submitter_weight × model_weight
网站平均分 = AVG(该网站所有提交的加权分)
```

**模型排名**：

```
单条加权分 = total_score × version_weight × submitter_weight × website_weight
模型平均分 = AVG(该模型所有提交的加权分)
```

| 权重类型      |     默认值      | 配置方式                                |
|-----------|:------------:|-------------------------------------|
| OAuth 提交者 |     1.0      | 环境变量 `SUBMITTER_WEIGHT_OAUTH`       |
| 匿名提交者     |     0.7      | 环境变量 `SUBMITTER_WEIGHT_ANONYMOUS`   |
| 各版本权重     | `{"v1":1.0}` | 环境变量 `VERSION_WEIGHTS` (JSON)       |
| 当前测试套件版本  |     `v1`     | 环境变量 `CURRENT_TEST_SUITE`           |
| 模型权重（网站排） |     1.0      | 数据库 `website_models.model_weight`   |
| 网站权重（模型排） |     1.0      | 数据库 `website_models.website_weight` |

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

| 组件  | 平台                 | 配置                           |
|-----|--------------------|------------------------------|
| 前端  | Cloudflare Pages   | `front/` 目录, `npm run build` |
| 后端  | Cloudflare Workers | `api/`, Rust→WASM            |
| 数据库 | Cloudflare D1      | `rank-data`                  |

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

| Secret 名               | 说明                              |
|------------------------|---------------------------------|
| `JWT_SECRET`           | JWT 签名密钥 (HS256)                |
| `GITHUB_CLIENT_ID`     | GitHub OAuth App Client ID      |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth App Client Secret  |
| `TURNSTILE_SECRET_KEY` | Cloudflare Turnstile Secret Key |

> 版本权重和提交者权重通过环境变量配置。模型/网站权重通过 D1 数据库 `website_models` 表管理，无需重启 Worker 即可调整。