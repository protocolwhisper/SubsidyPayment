---
app_name: [APP_NAME]
tool_count: [TOOL_COUNT]
---

# ツール仕様

## ツール一覧

| Tool | Title | Description | Input Schema | Output |
|---|---|---|---|---|
| [TOOL_1] | [TITLE] | [DESCRIPTION] | [INPUT_SCHEMA] | [OUTPUT_SHAPE] |
| [TOOL_2] | [TITLE] | [DESCRIPTION] | [INPUT_SCHEMA] | [OUTPUT_SHAPE] |

## 入力スキーマ詳細

### [TOOL_1]

- 入力:
  - [FIELD_1]: [TYPE] [REQUIRED]
  - [FIELD_2]: [TYPE] [REQUIRED]

### [TOOL_2]

- 入力:
  - [FIELD_1]: [TYPE] [REQUIRED]
  - [FIELD_2]: [TYPE] [REQUIRED]

## structuredContent の形

```json
{
  "structuredContent": {
    "key": "value",
    "items": []
  }
}
```

## エラーレスポンス方針

- 入力不足: [ERROR_MESSAGE_FOR_MISSING_INPUT]
- 権限不足: [ERROR_MESSAGE_FOR_UNAUTHORIZED]
- 依存サービス不調: [ERROR_MESSAGE_FOR_DEPENDENCY]
