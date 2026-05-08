# LLM Test API — 后端服务

Rust Cloudflare Worker 后端，编译为 WebAssembly 运行于 Cloudflare Workers 平台。提供认证、提交和排行榜查询等 REST API。

## 技术栈

- **语言**：Rust (edition 2021)，编译目标 `wasm32-unknown-unknown`
- **HTTP 框架**：axum 0.8 + worker-rs
- **数据库**：Cloudflare D1（SQLite 兼容）
- **认证**：JWT (HS256) + GitHub OAuth + Cloudflare Turnstile

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/auth/github/login` | GET | 302 重定向到 GitHub OAuth 授权页 |
| `/api/auth/github/callback` | POST | 用 OAuth code 换取 JWT Token |
| `/api/auth/anonymous` | POST | 验证 Turnstile token，返回匿名会话 token（24h 有效） |
| `/api/submissions` | POST | 提交测试跑分结果 |
| `/api/rankings` | GET | 排行榜查询（支持网站排行、模型排行、详情搜索） |

完整 API 请求/响应格式、认证流程、排名计算公式详见 **[DESIGN.md](../DESIGN.md)**。

## 本地开发

```bash
# 安装 Rust 和 WASM 编译目标
rustup target add wasm32-unknown-unknown

# 构建
cargo build

# 本地运行（需要 wrangler）
npx wrangler dev
```

本地运行需要配置 `wrangler.toml` 中的环境变量和 Secrets：

- **vars**：`FRONTEND_URL`、`TURNSTILE_SITE_KEY`、`CURRENT_TEST_SUITE`、`VERSION_WEIGHTS`、`SUBMITTER_WEIGHT_OAUTH`、`SUBMITTER_WEIGHT_ANONYMOUS`
- **secrets**：`JWT_SECRET`、`GITHUB_CLIENT_ID`、`GITHUB_CLIENT_SECRET`、`TURNSTILE_SECRET_KEY`

### 数据库初始化

```bash
wrangler d1 execute rank-data --file=schema.sql
```

## 项目结构

```
api/
├── src/
│   ├── lib.rs              # 入口：路由、CORS、日志
│   ├── auth.rs             # JWT 签名/验证、GitHub OAuth、Turnstile
│   ├── error.rs            # API 错误类型
│   ├── state.rs            # 共享状态（D1 + 环境变量 + 权重配置）
│   ├── handlers/           # 请求处理器
│   │   ├── mod.rs
│   │   ├── auth.rs         # GitHub 登录/回调、匿名认证
│   │   ├── proxy.rs        # LLM 代理（CORS 解决）
│   │   ├── rankings.rs     # GET /api/rankings
│   │   └── submit.rs       # POST /api/submissions
│   ├── models/             # 数据结构定义
│   └── db/                 # 数据库访问层
├── schema.sql              # D1 数据库 DDL
├── Cargo.toml
└── wrangler.toml           # Cloudflare Worker 配置
```

## 部署

```bash
wrangler deploy
```

## 更多信息

- 完整架构文档：[DESIGN.md](../DESIGN.md)
- 贡献指南：[CONTRIBUTING.md](../CONTRIBUTING.md)
