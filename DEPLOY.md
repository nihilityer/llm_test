# 部署指南

本文档介绍如何将 LLM Test 部署到你自己的 Cloudflare 账号下。

## 前置条件

- [Cloudflare 账号](https://dash.cloudflare.com/)
- [Node.js](https://nodejs.org/) 18+ / npm 9+
- [Rust](https://rustup.rs/) 1.80+
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/) (`npm install -g wrangler`)

## 架构概述

```
用户浏览器 ──→ Cloudflare Pages (前端 SPA)
     │
     ├─ AI API 直连 (OpenAI / Anthropic / Gemini)
     │
     └─→ Cloudflare Workers (后端 API) ──→ D1 (数据库)
              │
              ├─ GitHub OAuth
              └─ Turnstile 验证
```

## 部署步骤

### 1. 克隆仓库

```bash
git clone https://github.com/nihilityer/llm_test.git
cd llm_test
```

### 2. 安装 Wrangler 并登录

```bash
npm install -g wrangler
wrangler login
```

### 3. 创建 D1 数据库

```bash
wrangler d1 create llm-test-db
```

命令会输出类似以下内容：

```
✅ Created database 'llm-test-db' with ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

记录这个 database ID 和 database name，稍后填入 `wrangler.toml`。

初始化数据库表结构：

```bash
wrangler d1 execute llm-test-db --file=api/schema.sql
```

### 4. 创建 Turnstile 站点

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 进入 **Turnstile** 页面
3. 点击 **Add site**
4. 站点名称随意，域名填入你的前端域名（如 `example.com`）
5. Widget Mode 选择 **Managed**
6. 创建完成后获取 **Site Key** 和 **Secret Key**

### 5. 创建 GitHub OAuth App

1. 打开 [GitHub Developer Settings](https://github.com/settings/developers)
2. 点击 **New OAuth App**
3. 填写：
   - Application name: 你的平台名称
   - Homepage URL: `https://<你的前端域名>`
   - Authorization callback URL: `https://<你的前端域名>/auth/callback`
4. 注册后获取 **Client ID**
5. 点击 **Generate a new client secret** 获取 **Client Secret**

### 6. 创建 Secrets Store 并添加密钥

```bash
# 创建 Secrets Store
wrangler secret-store create llm-test-secrets

# 添加敏感密钥（命令会交互式提示输入值）
wrangler secret-store secret put llm-test-secrets github-client-secret
wrangler secret-store secret put llm-test-secrets jwt-secret
wrangler secret-store secret put llm-test-secrets turnstile-secret-key
```

- `github-client-secret`: 步骤 5 获取的 GitHub Client Secret
- `jwt-secret`: 一个随机字符串（建议 32 位以上），用于签名 JWT Token
- `turnstile-secret-key`: 步骤 4 获取的 Turnstile Secret Key

记录 Secrets Store ID 和各 secret name，稍后填入 `wrangler.toml`。

### 7. 配置后端 (wrangler.toml)

```bash
cd api
cp wrangler.toml.example wrangler.toml
```

编辑 `wrangler.toml`，将以下 `<YOUR_...>` 占位符替换为你的实际值：

| 占位符 | 说明 | 示例值 |
|--------|------|--------|
| `<YOUR_WORKER_NAME>` | Worker 名称 | `my-llm-test-api` |
| `<YOUR_DOMAIN>` | 你的域名（不含 https://） | `example.com` |
| `<YOUR_DB_NAME>` | D1 数据库名称 | `llm-test-db` |
| `<YOUR_D1_DATABASE_ID>` | D1 数据库 ID | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| `<YOUR_TURNSTILE_SITE_KEY>` | Turnstile Site Key | `0x4AAAAA...` |
| `<YOUR_GITHUB_CLIENT_ID>` | GitHub OAuth Client ID | `Ov23li...` |
| `<YOUR_SECRETS_STORE_ID>` | Secrets Store ID | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| `<YOUR_GITHUB_CLIENT_SECRET_NAME>` | Secrets Store 中的 Client Secret 名称 | `github-client-secret` |
| `<YOUR_JWT_SECRET_NAME>` | Secrets Store 中的 JWT Secret 名称 | `jwt-secret` |
| `<YOUR_TURNSTILE_SECRET_NAME>` | Secrets Store 中的 Turnstile Secret 名称 | `turnstile-secret-key` |

### 8. 配置前端环境变量

```bash
cd ../front
cp .env.example .env.production
```

编辑 `.env.production`：

```env
# API 后端地址（你的 Worker 绑定域名 + /api）
VITE_API_BASE_URL=https://api.<你的域名>/api

# Turnstile Site Key（与 wrangler.toml 中一致）
VITE_TURNSTILE_SITE_KEY=<你的 Turnstile Site Key>
```

### 9. 配置 Cloudflare 自定义域名

确保以下 DNS 记录已配置（在 Cloudflare DNS 或你的 DNS 提供商处）：

- **前端**：你的域名（如 `example.com` / `www.example.com`）指向 Cloudflare Pages
- **后端**：API 子域名（如 `api.example.com`）指向 Cloudflare Workers

### 10. 部署

#### 部署后端 (Cloudflare Workers)

```bash
cd api
wrangler deploy
```

部署成功后，Worker 将响应 `https://api.<你的域名>/api/*` 的请求。

#### 部署前端 (Cloudflare Pages)

选择以下方式之一：

**方式 A：通过 Wrangler CLI**

```bash
cd front
npm install
npm run build
wrangler pages deploy dist --project-name=<你的 Pages 项目名>
```

**方式 B：通过 Cloudflare Dashboard**

1. 在 Cloudflare Dashboard 中创建 Pages 项目
2. 连接到你的 GitHub 仓库
3. 设置构建配置：
   - Build command: `cd front && npm install && npm run build`
   - Output directory: `front/dist`
4. 添加环境变量（Settings > Environment variables）：
   - `VITE_API_BASE_URL` = `https://api.<你的域名>/api`
   - `VITE_TURNSTILE_SITE_KEY` = 你的 Turnstile Site Key

### 11. 验证部署

1. 打开 `https://<你的域名>`
2. 在 API 配置中输入你的 AI 端点 URL 和 API Key
3. 选择一个模型，运行测试
4. 尝试匿名提交和 GitHub 登录提交

## 常见问题

### CORS 报错

确认 `wrangler.toml` 中 `CORS_ORIGINS` 配置的前端域名正确，不含协议头（如 `example.com`）。

### GitHub 登录回调 404

1. 确认 `GITHUB_REDIRECT_URI` 与 GitHub OAuth App 中配置的回调 URL 一致
2. 确认前端路由 `/auth/callback` 可访问

### 匿名提交提示 Turnstile 验证失败

确认 `TURNSTILE_SITE_KEY`（wrangler.toml + 前端 .env.production）和 `TURNSTILE_SECRET_KEY`（Secrets Store）来自同一个 Turnstile 站点。
