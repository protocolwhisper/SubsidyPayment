---
name: gpt-apps-sdk-builder
description: GPT Apps SDKを用いたアプリ開発を設計・実装・検証する
---

# GPT Apps SDK Builder

あなたはGPT Apps SDKのアーキテクト兼実装パートナーとして、MCPサーバーとReact UIウィジェットを一貫して設計・実装・検証・公開まで導く。

---

## Prerequisites

- Node.js 18+ / pnpm or npm
- Python 3.10+ (Python サーバーの場合)
- React 18+ / 19+, Vite 7+, Tailwind CSS v4
- `@openai/apps-sdk-ui` (UIコンポーネントライブラリ)
- `@modelcontextprotocol/sdk` + `@modelcontextprotocol/ext-apps`
- 公開URLを用意するトンネルまたはホスティング（例: ngrok, Render）
- ChatGPT の開発者モード有効化
- 環境変数: `PORT`（任意）, `PUBLIC_MCP_URL`（公開時）, `APP_NAME`

---

## Architecture

GPT Apps は 3 コンポーネントで構成される：

```
┌─────────────┐    JSON-RPC 2.0     ┌──────────────────┐    HTTP    ┌─────────────┐
│  Widget UI  │◄──── postMessage ────►│  ChatGPT Host    │◄──────────►│  MCP Server │
│  (iframe)   │                      │                  │           │  (Node/Py)  │
│  React +    │                      │  model decides   │           │  tools +    │
│  Tailwind   │                      │  when to invoke  │           │  resources  │
└─────────────┘                      └──────────────────┘           └─────────────┘
```

### Three-Tier State Model

| Tier | Location | Persistence | Example |
|------|----------|-------------|---------|
| Business Data | MCP Server / Backend | Authoritative | タスク一覧, ユーザー情報 |
| Widget State | `window.openai.widgetState` | Session-scoped | 選択中のタブ, ソート順 |
| UI State | React `useState` | Ephemeral | ホバー状態, 入力中テキスト |

---

## Workflow

### Step 1: 要件ブリーフを作成する

アプリの目的、ユーザー体験、必要なツール、データソース、認証要否を短く整理する。

**Input**: ユーザー要望、対象ユーザー、操作シナリオ
**Output**: `templates/app_brief_template.md` を埋めた要件ブリーフ
**If this fails**: 不明点を列挙し、最小構成の仮ブリーフを作成して前進する

### Step 2: ツール仕様とデータモデルを定義する

ツール名、入力スキーマ、出力形式、状態管理を決定する。**Decoupled Data + Render パターン**を考慮する。

**Input**: 要件ブリーフ
**Output**: `templates/tool_spec_template.md` を埋めたツール仕様
**If this fails**: ツール数を削減し、1ツール単位で段階的に定義する

**重要: Decoupled Data + Render パターン**
- データ取得ツール: `outputTemplate` なし、データだけ返す
- レンダーツール: `outputTemplate` あり、データを受け取りUIに渡す
- これにより不要な iframe リマウントを防止する

### Step 3: UIウィジェットを設計・実装する（React）

React + Tailwind CSS + `@openai/apps-sdk-ui` でウィジェットを構築する。

**Input**: UI要件、ツール仕様、表示モード選択
**Output**: React コンポーネント + Vite ビルド設定
**If this fails**: プレーンHTML (`templates/ui_widget_template.html`) にフォールバック

**技術スタック:**
```
dependencies:
  @openai/apps-sdk-ui: ^0.2.1
  react: ^19.1.1
  react-dom: ^19.1.1
  lucide-react: ^0.536.0    # アイコン
  clsx: ^2.1.1               # クラス名結合
  framer-motion: ^12.23.12   # アニメーション（任意）

devDependencies:
  @tailwindcss/vite: ^4.1.11
  tailwindcss: 4.1.11
  vite: ^7.1.1
  @vitejs/plugin-react: ^4.5.2
  typescript: ^5.9.2
```

**CSS設定 (index.css):**
```css
@import "tailwindcss";
@import "@openai/apps-sdk-ui/css";
@source "../node_modules/@openai/apps-sdk-ui";
@source ".";
```

### Step 4: MCPサーバーを実装する

`registerAppTool` + `registerAppResource` でツールとUIリソースを登録し、/mcp エンドポイントを提供する。

**Input**: ツール仕様、ビルド済みUIウィジェット
**Output**: `templates/node_mcp_server_template.md` を基にしたサーバー実装
**If this fails**: 依存関係、CORS、/mcp ルート、ESM設定を再確認する

**ツールレスポンス 3 層構造:**
| フィールド | 可視範囲 | 用途 |
|-----------|---------|------|
| `structuredContent` | Model + Widget | 簡潔なJSON（outputSchemaに対応） |
| `content` | Model + Widget | Markdown/テキストによる説明文 |
| `_meta` | Widget のみ | 大量/機密データ（モデルには非表示） |

### Step 5: ローカル検証を実行する

MCP Inspectorでツール呼び出しとUI連携を確認する。

```bash
npx @modelcontextprotocol/inspector@latest --server-url http://localhost:8787/mcp --transport http
```

**Input**: ローカルMCP URL（例: http://localhost:8787/mcp）
**Output**: ツールが期待通りの structuredContent を返すこと
**If this fails**: inspectorのログを確認し、ツール定義と入力スキーマを調整する

### Step 6: 公開URLで接続を確認する

公開URLに /mcp を付与してChatGPTに追加し、実際の会話で動作確認する。

**Input**: PUBLIC_MCP_URL
**Output**: ChatGPT上でのツール実行とUI表示
**If this fails**: HTTPS、CORS、/mcp パス、レスポンス形式を再確認する

**ChatGPT への接続手順:**
1. Settings → Apps & Connectors で開発者モードを有効化
2. Connector を作成し、HTTPS URL + `/mcp` を設定
3. 会話の More メニューから Connector を追加

### Step 7: 反復と提出準備を行う

ツール説明、エラーメッセージ、UI導線を改善し、提出ガイドラインに沿って最終確認する。

**Input**: テスト結果、使用ログ、改善点
**Output**: `resources/launch_checklist.md` の完了
**If this fails**: 直近の変更を差し戻し、最小構成で再確認する

---

## Key Concepts

### window.openai API

Widget の iframe 内でグローバルに利用可能な API：

**State & Data:**
```typescript
window.openai.theme          // "light" | "dark"
window.openai.locale         // BCP 47 ロケール
window.openai.displayMode    // "inline" | "fullscreen" | "pip"
window.openai.maxHeight      // ウィジェット最大高さ (px)
window.openai.safeArea       // { insets: { top, bottom, left, right } }
window.openai.userAgent      // { device: { type }, capabilities: { hover, touch } }
window.openai.toolInput      // モデルがツールに渡した引数
window.openai.toolOutput     // structuredContent (ツール結果)
window.openai.toolResponseMetadata  // _meta (ウィジェット専用データ)
window.openai.widgetState    // 永続化されたUI状態
```

**API Methods:**
```typescript
window.openai.callTool(name, args)           // ツール呼び出し
window.openai.setWidgetState(state)          // UI状態永続化
window.openai.sendFollowUpMessage({ prompt }) // フォローアップ送信
window.openai.requestDisplayMode({ mode })   // 表示モード変更
window.openai.requestModal({ title, params }) // モーダル表示
window.openai.requestClose()                  // ウィジェット閉じる
window.openai.openExternal({ href })          // 外部リンク
window.openai.uploadFile(file)                // ファイルアップロード
window.openai.getFileDownloadUrl({ fileId })  // ファイルダウンロード
```

### Display Modes

| Mode | 用途 | 制約 |
|------|------|------|
| **inline** | 会話内カード（デフォルト） | 最大2アクション、ネストスクロール禁止、タブ/ドリルイン禁止 |
| **fullscreen** | 没入型ワークフロー（マップ、エディタ） | ChatGPT コンポーザーが常に下部に表示 |
| **pip** | 並行アクティビティ（ゲーム、ライブ） | モバイルではfullscreenに強制変換 |

### @openai/apps-sdk-ui コンポーネント

```typescript
import { Badge } from "@openai/apps-sdk-ui/components/Badge";
import { Button } from "@openai/apps-sdk-ui/components/Button";
import { CodeBlock } from "@openai/apps-sdk-ui/components/CodeBlock";
import { Checkbox } from "@openai/apps-sdk-ui/components/Checkbox";
import { Input } from "@openai/apps-sdk-ui/components/Input";
import { Textarea } from "@openai/apps-sdk-ui/components/Textarea";
import { Image } from "@openai/apps-sdk-ui/components/Image";
```

**Button:**
```tsx
<Button color="primary" | "secondary" variant="solid" | "outline" | "soft" | "ghost" size="sm" | "md" block uniform>
```

**Badge:**
```tsx
<Badge variant="soft" | "solid" color="primary" | "secondary" | "info" | "danger" pill>
```

### React Hooks for window.openai

`resources/react_hooks_reference.md` に完全なソースコードあり。

| Hook | 用途 | 戻り値 |
|------|------|--------|
| `useOpenAiGlobal(key)` | window.openai の任意プロパティを購読 | `T \| null` |
| `useWidgetState(default)` | widgetState の読み書き | `[state, setState]` |
| `useWidgetProps(default)` | toolOutput の読み取り | `T` |
| `useDisplayMode()` | 現在の表示モード | `DisplayMode \| null` |
| `useMaxHeight()` | 最大高さ | `number \| null` |

### Tool Metadata

```typescript
registerAppTool(server, "tool-name", {
  title: "表示名",
  description: "モデルへの説明",
  inputSchema: zodSchema,
  _meta: {
    ui: {
      resourceUri: "ui://widget/my-widget.html",
      visibility: ["model", "app"],   // 呼び出し可能者
      prefersBorder: true,            // カード枠線ヒント
      domain: "https://myapp.example.com",
      csp: {
        connectDomains: ["https://api.example.com"],
        resourceDomains: ["https://cdn.example.com"],
      },
    },
    "openai/outputTemplate": "ui://widget/my-widget.html",
    "openai/toolInvocation/invoking": "読み込み中...",  // <=64文字
    "openai/toolInvocation/invoked": "完了",            // <=64文字
    "openai/widgetDescription": "インタラクティブな...",
  },
  annotations: {
    readOnlyHint: true,       // 読み取り専用
    destructiveHint: false,   // 破壊的操作
    openWorldHint: false,     // 外部公開
  },
}, handler);
```

---

## Error Handling

| Error | Cause | Fix |
|---|---|---|
| 404 /mcp | ルート設定やパスが不一致 | /mcp ルートとHTTPメソッド対応を確認 |
| CORS error | プリフライト未対応 | OPTIONSを処理しヘッダーを付与 |
| Tool not found | tool名が不一致 | register名と呼び出し名を一致させる |
| UIが表示されない | resourceUri不一致 | UI resourceUri と toolの ui.resourceUri を合わせる |
| structuredContentが空 | レスポンス形式が不一致 | structuredContentに必要なキーを返す |
| Widget state lost | widgetSessionId 未設定 | _meta に openai/widgetSessionId を設定 |
| iframe remount | 同一ツールの再呼び出し | Decoupled Data + Render パターンを適用 |
| PiP → fullscreen | モバイルでの PiP | モバイルでは PiP が fullscreen に強制変換される |

---

## Security Requirements

- API キー、トークン、秘密情報を `structuredContent` / `content` / `_meta` / `widgetState` に埋め込まない
- 認証はサーバー側で強制（クライアント側ヒントに依存しない）
- admin / 破壊的ツールは本人確認なしに公開しない
- ツール入力はサーバー側で必ずバリデーション
- ハンドラは冪等に設計（モデルはリトライする可能性がある）
- 破壊的操作は確認を求める
- `_meta["openai/locale"]` / `_meta["openai/userAgent"]` は認可に使用しない

---

## Examples

### Example 1: React UI付きサービス検索アプリ

**User says**: "補助金サービスを検索してカード表示するウィジェットを作りたい"

**Agent does**:
1. app_brief_template.md を埋め、表示モードは inline カードを選択
2. search_services（データツール）と render_services（レンダーツール）を定義
3. React + `@openai/apps-sdk-ui` で `useWidgetProps` を使ったカードUIを実装
4. Vite でビルドし MCP サーバーに登録、Inspector で検証

### Example 2: UIなしの翻訳ツール

**User says**: "UIなしで翻訳ツールだけを使えるようにしたい。"

**Agent does**:
1. ツール仕様のみ定義し、UI resource を登録しない
2. MCPサーバーに翻訳ツールを登録し、ツール出力を structuredContent で返す
3. 公開URLでChatGPTに接続し、会話内でツールを実行する

### Example 3: Fullscreen マップアプリ

**User says**: "地図上にサービス拠点を表示するインタラクティブなアプリを作りたい"

**Agent does**:
1. 表示モードは fullscreen を選択
2. `requestDisplayMode("fullscreen")` で没入モードを要求
3. Mapbox GL + React で地図コンポーネントを実装
4. `useWidgetState` でピン選択状態を永続化

### Example 4: ショッピングカート（状態同期）

**User says**: "会話を通じてカートに商品を追加できるUIが欲しい"

**Agent does**:
1. `useWidgetState` でカート状態を ChatGPT ホストに永続化
2. `toolOutput` の変更を `useEffect` で検知し、差分マージ
3. UI からは `window.openai.callTool` でサーバー側カートを更新
4. `widgetSessionId` で同一ウィジェットインスタンスを識別

---

## References

- https://developers.openai.com/apps-sdk/
- https://developers.openai.com/apps-sdk/quickstart/
- https://developers.openai.com/apps-sdk/build/chatgpt-ui/
- https://developers.openai.com/apps-sdk/build/mcp-server/
- https://developers.openai.com/apps-sdk/build/state-management/
- https://developers.openai.com/apps-sdk/reference/
- https://developers.openai.com/apps-sdk/concepts/ui-guidelines/
- https://developers.openai.com/apps-sdk/concepts/ux-principles/
- https://developers.openai.com/apps-sdk/plan/components/
- https://developers.openai.com/apps-sdk/plan/tools/
- https://github.com/openai/openai-apps-sdk-examples
- https://openai.github.io/apps-sdk-ui/
- https://www.figma.com/community/file/1560064615791108827
