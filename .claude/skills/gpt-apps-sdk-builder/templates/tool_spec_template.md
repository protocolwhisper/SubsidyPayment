---
app_name: [APP_NAME]
tool_count: [TOOL_COUNT]
---

# ツール仕様

## アーキテクチャパターン

- [ ] **Decoupled Data + Render** — データ取得と UI レンダリングを分離（推奨）
- [ ] **Standalone** — 1 ツールでデータ取得 + UI 表示

## ツール一覧

| Tool | Title | Description | Type | outputTemplate |
|---|---|---|---|---|
| [TOOL_1] | [TITLE] | [DESCRIPTION] | data | なし |
| [TOOL_2] | [TITLE] | [DESCRIPTION] | render | あり |
| [TOOL_3] | [TITLE] | [DESCRIPTION] | standalone | あり |

## 入力スキーマ詳細

### [DATA_TOOL]（データ取得）

- 入力:
  - [FIELD_1]: z.string() (required) — [説明]
  - [FIELD_2]: z.number().optional() — [説明]
- annotations:
  - readOnlyHint: true
  - destructiveHint: false
  - openWorldHint: false

### [RENDER_TOOL]（UI レンダリング）

- 入力:
  - items: z.array(z.object({...})) (required) — [DATA_TOOL] の出力を受け取る
  - totalCount: z.number() (required)
  - hasMore: z.boolean() (required)
- _meta:
  - ui.resourceUri: "ui://widget/[WIDGET_FILE]"
  - openai/outputTemplate: "ui://widget/[WIDGET_FILE]"
  - openai/toolInvocation/invoking: "[<=64文字のメッセージ]"
  - openai/toolInvocation/invoked: "[<=64文字のメッセージ]"
- annotations:
  - readOnlyHint: true
  - destructiveHint: false
  - openWorldHint: false

## structuredContent の形

```json
{
  "structuredContent": {
    "items": [
      {
        "id": "string",
        "name": "string"
      }
    ],
    "totalCount": 0,
    "hasMore": false
  }
}
```

**設計原則:**
- UI に必要な状態だけを小さくまとめる
- UI レンダリングは structuredContent から完全に再構築可能にする
- エラー時も structuredContent を空配列・デフォルト値で返す
- 機密情報（トークン、内部ID等）は `_meta` に入れる（Widget のみ可視）

## _meta の形（Widget 専用データ）

```json
{
  "_meta": {
    "sensitiveData": "Widget のみが参照できる大量/機密データ",
    "openai/toolInvocation/invoking": "読み込み中...",
    "openai/toolInvocation/invoked": "完了"
  }
}
```

## エラーレスポンス方針

| ケース | content | structuredContent |
|--------|---------|-------------------|
| 入力不足 | エラーメッセージ（テキスト） | `{ items: [], error: "入力が不足しています" }` |
| 権限不足 | 認証を促すメッセージ | `{ items: [], error: "認証が必要です" }` |
| 依存サービス不調 | リトライを促すメッセージ | `{ items: [], error: "一時的にサービスに接続できません" }` |
| データ0件 | 検索条件の変更を提案 | `{ items: [], totalCount: 0, hasMore: false }` |

## annotations 設計ガイド

| フィールド | 説明 | 設定例 |
|-----------|------|--------|
| readOnlyHint | 読み取り専用操作 | 検索、一覧取得 → true |
| destructiveHint | データ削除/上書き | 削除操作 → true |
| openWorldHint | 外部公開操作 | メール送信 → true |
| idempotentHint | 冪等操作 | 同じ入力で同じ結果 → true |

**注意:** `readOnlyHint: true` + `destructiveHint: false` の場合、ChatGPT は承認プロンプトをスキップする。
