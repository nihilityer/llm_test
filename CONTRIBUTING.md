# 贡献指南

感谢你考虑为 LLM Test 做贡献！无论是提交测试用例、报告 Bug，还是改进代码，我们都欢迎。

## 贡献测试用例

测试用例是 LLM Test 社区驱动的核心。你的每一次贡献都会让评测更加全面、更贴近真实使用场景。

### 方式一：通过 Issue 提交想法（推荐）

如果你有一个好的测试场景想法，但不熟悉项目的 JSON 格式，只需提交一个 Issue 描述你的想法即可：

1. 打开 [Issues](https://github.com/nihilityer/llm_test/issues) 页面
2. 选择 **"提交测试用例"** 模板
3. 用自然语言填写测试名称、描述、提示词和预期答案
4. 提交后，维护者会将其转换为正式的测试用例格式

### 方式二：开发者直接提交 PR

如果你熟悉项目格式，可以直接提交包含完整 JSON 定义的 PR。

#### JSON 格式说明

测试用例存储在 `front/src/test-suite/v1/cases/` 目录下，每个用例一个 JSON 文件。以 `car_wash.json` 为例：

```json
{
  "id": "tc_car_wash",
  "title": "洗车问题",
  "description": "去离家五十米的洗车店洗车，应该开车去还是走路去",
  "difficulty": "easy",
  "prompt": "去离家五十米的洗车店洗车，应该开车去还是走路去？请做出判断。",
  "parameters": {
    "type": "object",
    "properties": {
      "transportation": {
        "type": "string",
        "description": "选择的出行方式",
        "enum": ["开车去", "走路去"]
      },
      "reason": {
        "type": "string",
        "description": "选择理由"
      }
    },
    "required": ["transportation"]
  },
  "verify": {
    "arguments": {
      "transportation": "开车去"
    }
  }
}
```

**字段说明：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 唯一标识符，格式 `tc_xxx`，建议使用英文下划线命名 |
| `title` | string | 中文标题，简洁明了 |
| `description` | string | 测试场景说明和考察的能力 |
| `difficulty` | string | 难度：`easy` / `medium` / `hard` |
| `prompt` | string | 提交给 AI 模型的完整提示词，应包含明确的判断要求 |
| `parameters` | object | JSON Schema 格式的参数定义，AI 需通过 tool call 返回结构化结果 |
| `verify.arguments` | object | 期望的正确答案，与 `parameters` 中的字段对应 |

**注意事项：**

- `parameters` 使用 [JSON Schema](https://json-schema.org/) 格式。AI 模型通过 function calling / tool use 机制输出符合此 schema 的结构化参数
- `verify.arguments` 中的键名和值必须与 `parameters.properties` 一致，且值必须在对应字段的 `enum` 范围内
- 如果 `parameters` 或 `verify` 定义不正确，测试引擎可能无法正确判断 AI 输出，导致评分异常

#### 提交流程

1. Fork 仓库并创建功能分支
2. 在 `front/src/test-suite/v1/cases/` 下新建 JSON 文件，命名与 `id` 一致（如 `tc_your_case.json`）
3. 在 `front/src/test-suite/v1/index.ts` 中引入并注册新用例：

```typescript
import yourCaseJson from './cases/tc_your_case.json'

export const suiteV1: TestSuite = {
  // ...
  testCases: [
    asTestCase(carWashJson),
    asTestCase(schoolCleaningJson),
    asTestCase(yourCaseJson),   // 新增
  ],
}
```

4. 运行 `cd front && npm run build` 确保构建通过
5. 提交 PR

> **提示**：添加或删除测试用例会改变评分基准。新增用例应确保其难度评定准确，避免影响排名公正性。

## 报告 Bug

发现问题？提交 Bug 报告请使用 **"报告 Bug"** Issue 模板。为提高效率，请提供：

- 清晰的问题描述和复现步骤
- 浏览器和操作系统信息
- 相关的截图或日志

## 功能建议

有新功能想法？使用 **"功能建议"** Issue 模板提交即可。

## 开发环境搭建

### 前端

```bash
cd front
npm install
npm run dev        # 开发模式，默认 http://localhost:5173
```

需要 Node.js 18+、npm 9+。

### 后端

```bash
cd api
# 需要 Rust toolchain + wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown
cargo build
npx wrangler dev    # 本地运行 Worker，默认 http://localhost:8787
```

需要 Rust 1.80+、Cloudflare Wrangler CLI。

> **注意**：本地开发需要创建自己的 Cloudflare 项目配置（`wrangler.toml` 中的 D1 数据库、OAuth App、Turnstile 密钥等）。不要使用生产环境密钥。

#### D1 数据库初始化

```bash
wrangler d1 execute rank-data --file=api/schema.sql
```

#### 完整架构文档

项目架构、全部 API 接口规范、数据库设计、评分算法详见 **[DESIGN.md](./DESIGN.md)**。

## 代码风格

### 前端

- 使用 ESLint + Oxlint + Prettier
- 运行 `npm run lint` 检查代码
- 运行 `npm run format` 格式化
- TypeScript 严格模式
- 命名：camelCase（变量/函数）、PascalCase（组件）、kebab-case（文件名）

### 后端

- 使用 `rustfmt`
- 运行 `cargo fmt` 格式化
- 命名：snake_case（变量/函数）、PascalCase（类型）

## Pull Request 流程

1. Fork 仓库并创建功能分支
2. 进行修改并确保所有 lint 和构建检查通过
3. PR 标题清晰描述变更内容
4. PR 描述中选择正确的变更类型
5. 关联相关 Issue（如有）
6. 等待 Review
