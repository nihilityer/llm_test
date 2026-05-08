# LLM Test

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](./LICENSE)
[![Website](https://img.shields.io/badge/website-llmtest.top-green.svg)](https://llmtest.top)
[![Frontend](https://img.shields.io/badge/frontend-Vue%203%20%2B%20TypeScript-4FC08D?logo=vue.js)](./front)
[![Backend](https://img.shields.io/badge/backend-Rust%20%2B%20Cloudflare%20Workers-fa7328?logo=rust)](./api)
[![Database](https://img.shields.io/badge/database-Cloudflare%20D1-F38020)](./api/schema.sql)

**LLM Test** 是一个开放、透明、社区驱动的 AI 模型实际能力评测平台。

## 项目初衷

目前主流 AI 基准测试（MMLU、HumanEval、GSM8K 等）已经趋近饱和，各大模型在这些测试上分数差距越来越小，且测试内容与实际使用场景存在较大脱节。与此同时，越来越多的 AI API 服务提供商声称自己的模型具备极强的编码和推理能力，但用户缺乏一个简单、直接的方式来验证这些声明。

LLM Test 的初衷是构建一个**由社区共同维护的实战测试集**，让任何人都能：

- 用自己的 API 端点直接测试 AI 模型的实际推理和判断能力
- 查看不同 API 提供商、不同模型在相同测试下的真实表现排行
- **贡献自己的测试用例**，丰富测试场景，让评测更加全面和贴近真实使用

API Key 仅存储在浏览器内存中，绝不上传至服务器。（端点不允许跨域请求时需要服务器代理请求，服务器仅负责传输，不会打印相关日志）

## 快速开始

**无需安装，直接打开网站即可使用：**

1. 打开 **[llmtest.top](https://llmtest.top)**
2. 填入你的 API 端点 URL、API Key 和模型名称
3. 点击 **"开始测试"**，等待结果
4. 测试完成后可将结果提交到排行榜（支持 GitHub 登录或匿名提交）

## 核心特性

- **多 API 风格支持**：兼容 OpenAI、Anthropic、Gemini 三种 API 风格，也支持任何兼容这些格式的第三方端点
- **浏览器直连**：所有 AI 调用直接从浏览器发起，API Key 不上传后端，保障隐私安全
- **标准化测试套件**：包含判断力和逻辑推理测试用例，自动评分（正确性 + 效率 + 一致性）
- **双维度排行榜**：网站排名（按 API 提供商）和模型排名，加权算法保证公平性
- **社区驱动的测试集**：欢迎提交新的测试用例，让评测更全面、更贴近真实场景

## 项目结构

```
llm-test/
├── front/              # Vue 3 + TypeScript 前端（浏览器直连 AI API）
├── api/                # Rust Cloudflare Worker 后端（认证、提交、排行）
├── DESIGN.md           # 完整架构设计文档
├── DEPLOY.md           # 自行部署指南
├── CONTRIBUTING.md     # 贡献指南
└── LICENSE             # Apache 2.0
```

## 架构概览

前端（Vue 3 + TypeScript，托管于 Cloudflare Pages）在浏览器中直接调用各 AI API 进行测试，评分引擎纯前端计算。后端（Rust → WASM，运行于 Cloudflare Workers）仅提供 5 个 REST API 端点，负责 GitHub OAuth 认证、Turnstile 匿名验证、排行榜查询和结果提交。数据存储于 Cloudflare D1（SQLite）。

完整架构设计、API 接口规范、数据库 schema、排名计算公式详见 **[DESIGN.md](./DESIGN.md)**。

## 贡献

LLM Test 的核心生命力来自社区贡献的测试用例。我们欢迎：

- **提交测试用例**：通过 [Issue 模板](https://github.com/nihilityer/llm_test/issues/new) 提交你的测试想法，或直接提交包含完整 JSON 定义的 PR
- **报告 Bug 和建议**：发现问题或有改进想法，欢迎提交 Issue
- **代码贡献**：Fork 仓库，提交 PR

详细说明请参阅 **[CONTRIBUTING.md](./CONTRIBUTING.md)**。

## 反馈

如果你在使用过程中遇到任何问题，或有功能建议、测试用例想法，欢迎到 [Issues](https://github.com/nihilityer/llm_test/issues) 提交反馈。

## License

## 自行部署

如需将项目部署到你自己的 Cloudflare 账号下，请参考 **[DEPLOY.md](./DEPLOY.md)** 中的完整部署指南。

## License

本项目基于 [Apache License 2.0](./LICENSE) 开源。
