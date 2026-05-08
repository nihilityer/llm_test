---
name: 提交测试用例
description: 为 LLM Test 贡献新的测试用例 —— 社区驱动的核心
title: "[测试用例] "
labels: ["test-case"]
assignees: []
---

**感谢你为 LLM Test 贡献测试用例！测试用例是平台的核心生命力，你的贡献将帮助更多人评估 AI 模型的实际能力。**

---

### 测试用例名称
<!-- 简短、清晰的名称，例如"洗车问题"、"大扫除制度" -->

### 测试用例描述
<!-- 这个测试场景考察 AI 哪方面的能力？ -->

### 难度级别
<!-- 在下方写一个：easy / medium / hard -->

### 提示词 (Prompt)
<!-- 提交给 AI 模型的完整提示词，应包含明确的判断要求。 -->

### 预期答案
<!-- AI 模型应该做出的正确判断是什么？请以结构化方式描述。 -->
<!-- 例如：{"transportation": "开车去"} 或 {"judgment": "不合理"} -->

### 评分依据（可选）
<!-- 为什么这个测试用例能有效评估 AI 能力？评分标准是什么？ -->

---

### 开发者补充

如果你熟悉项目格式，也可以直接按 JSON 格式提供完整定义（参考 `front/src/test-suite/v1/cases/` 下的现有用例）：

```json
{
  "id": "tc_your_id",
  "title": "用例名称",
  "description": "用例描述",
  "difficulty": "easy",
  "prompt": "提示词内容",
  "parameters": {
    "type": "object",
    "properties": {
      "answer": {
        "type": "string",
        "description": "参数描述",
        "enum": ["选项A", "选项B"]
      },
      "reason": {
        "type": "string",
        "description": "理由描述"
      }
    },
    "required": ["answer"]
  },
  "verify": {
    "arguments": {
      "answer": "选项A"
    }
  }
}
```

> **注意**：`parameters` 使用 JSON Schema 格式定义 AI 需要输出的结构化参数（通过 tool call / function calling 实现）。`verify/arguments` 为期望的正确答案。确保 `enum` 的值与正确答案一致。
