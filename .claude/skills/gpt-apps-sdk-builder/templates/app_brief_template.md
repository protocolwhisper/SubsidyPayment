---
app_name: [APP_NAME]
app_summary: [ONE_LINE_SUMMARY]
owner: [OWNER_OR_TEAM]
version: [VERSION]
---

# アプリ要件ブリーフ

## 目的

[APP_GOAL]

## 対象ユーザー

[TARGET_USERS]

## 主要な利用シナリオ

1. [SCENARIO_1]
2. [SCENARIO_2]
3. [SCENARIO_3]

## ツール一覧

| Tool | Purpose | Input | Output | Type |
|---|---|---|---|---|
| [TOOL_1] | [PURPOSE] | [INPUT_SCHEMA] | [OUTPUT_SCHEMA] | data / render / standalone |
| [TOOL_2] | [PURPOSE] | [INPUT_SCHEMA] | [OUTPUT_SCHEMA] | data / render / standalone |

### Decoupled Data + Render パターン

- [ ] データ取得と UI レンダリングを分離する（iframe リマウント防止）
- データツール: `[DATA_TOOL]` → outputTemplate なし
- レンダーツール: `[RENDER_TOOL]` → outputTemplate あり

## データソース

- [DATA_SOURCE_1]
- [DATA_SOURCE_2]

## 認証・権限

- 認証方式: [AUTH_METHOD: none / OAuth / API Key]
- ユーザー識別: [USER_ID_STRATEGY]
- 破壊的操作の確認: [CONFIRMATION_STRATEGY]

## UI要件

### 表示モード

- [ ] inline（会話内カード — デフォルト）
- [ ] fullscreen（没入型ワークフロー）
- [ ] pip（並行アクティビティ）

選択理由: [WHY_THIS_MODE]

### UI技術スタック

- [ ] React + Tailwind CSS + @openai/apps-sdk-ui（推奨）
- [ ] プレーンHTML + CSS + UIブリッジ（軽量）
- [ ] UIなし（ツールのみ）

### 主要画面 / コンポーネント

| 画面 | 説明 | 表示モード |
|------|------|-----------|
| [SCREEN_1] | [DESCRIPTION] | inline / fullscreen / pip |
| [SCREEN_2] | [DESCRIPTION] | inline / fullscreen / pip |

### 状態管理設計

| State | Tier | Storage | 説明 |
|-------|------|---------|------|
| [STATE_1] | Business | Server/Backend | [DESCRIPTION] |
| [STATE_2] | Widget | widgetState | [DESCRIPTION] |
| [STATE_3] | UI | useState | [DESCRIPTION] |

### @openai/apps-sdk-ui 使用コンポーネント

- [ ] Button (primary / secondary)
- [ ] Badge (status 表示)
- [ ] Input / Textarea (検索/入力)
- [ ] Image (画像表示)
- [ ] CodeBlock (コード表示)
- [ ] Checkbox (選択)

## ビルド・デプロイ

### Widget ビルド

- ビルドツール: Vite + React + Tailwind v4
- エントリポイント: `src/[WIDGET_NAME]/index.tsx`
- 出力: `dist/[WIDGET_NAME].html`

### MCP サーバー

- ランタイム: Node.js / Python
- フレームワーク: `@modelcontextprotocol/sdk` + `@modelcontextprotocol/ext-apps`
- エンドポイント: `/mcp`
- ポート: [PORT]

### 環境変数

| 変数名 | 説明 | 必須 |
|--------|------|------|
| PORT | サーバーポート | No (default: 8787) |
| PUBLIC_MCP_URL | 公開MCP URL | 公開時 |
| [CUSTOM_ENV_1] | [DESCRIPTION] | [Yes/No] |

## 非機能要件

- 応答時間: [LATENCY_TARGET]
- エラー方針: [ERROR_POLICY]
- アクセシビリティ: WCAG AA 準拠
- セキュリティ: 機密データを structuredContent/widgetState に含めない
