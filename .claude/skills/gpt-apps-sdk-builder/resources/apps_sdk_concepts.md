---
title: Apps SDK 基本概念
---

# Apps SDK 基本概念

## アーキテクチャ概要

GPT Apps は 3 コンポーネントで構成される：

1. **MCP Server** — ChatGPTからの呼び出しを受け、ツールとUIリソースを提供
2. **Widget UI** — ChatGPT の sandboxed iframe 内でレンダリングされる React コンポーネント
3. **ChatGPT Host** — ツール呼び出しのオーケストレーション、widget との JSON-RPC 通信

```
Widget (iframe) ←─ JSON-RPC 2.0 over postMessage ─→ ChatGPT Host ←─ HTTP ─→ MCP Server
```

## 主要コンポーネント

| 要素 | 役割 | 要点 |
|---|---|---|
| McpServer | ChatGPTからの呼び出しを受ける | /mcp でHTTPを受ける |
| registerAppTool | ツールを登録 | inputSchema + _meta + annotations |
| registerAppResource | UIリソースを登録 | resourceUri + MIME type |
| Tool | モデルが呼ぶ機能 | 入力スキーマと出力の一貫性 |
| Resource | UIなどの静的リソース | resourceUri で関連付け |
| window.openai | Widget側API | 状態・メソッド・イベント |

## window.openai API 完全リファレンス

### State & Data Properties

```typescript
type OpenAiGlobals = {
  // Visuals
  theme: "light" | "dark";
  userAgent: {
    device: { type: "mobile" | "tablet" | "desktop" | "unknown" };
    capabilities: { hover: boolean; touch: boolean };
  };
  locale: string;

  // Layout
  maxHeight: number;
  displayMode: "pip" | "inline" | "fullscreen";
  safeArea: { insets: { top: number; bottom: number; left: number; right: number } };

  // State
  toolInput: Record<string, unknown>;
  toolOutput: Record<string, unknown> | null;
  toolResponseMetadata: Record<string, unknown> | null;
  widgetState: Record<string, unknown> | null;
  setWidgetState: (state: Record<string, unknown>) => Promise<void>;
};
```

### Runtime API Methods

```typescript
type API = {
  // Tool invocation
  callTool: (name: string, args: Record<string, unknown>) => Promise<{ result: string }>;

  // Messaging
  sendFollowUpMessage: (args: { prompt: string }) => Promise<void>;

  // File operations
  uploadFile: (file: File) => Promise<{ fileId: string }>;
  getFileDownloadUrl: (args: { fileId: string }) => Promise<{ downloadUrl: string }>;

  // Layout controls
  requestDisplayMode: (args: { mode: "inline" | "pip" | "fullscreen" }) => Promise<{ mode: DisplayMode }>;
  requestModal: (args: { title?: string; params?: unknown }) => Promise<unknown>;
  requestClose: () => Promise<void>;

  // External navigation
  openExternal: (payload: { href: string }) => void;
  setOpenInAppUrl: (payload: { href: string }) => void;
};
```

### Global Type Declaration

```typescript
declare global {
  interface Window {
    openai: API & OpenAiGlobals;
  }
}
```

## Display Modes

### Inline（デフォルト）

会話フロー内に埋め込まれるカード。

- 最大2アクション（プライマリCTA 1 + セカンダリ 1）
- ネストスクロール禁止
- タブ、ドリルイン、ディープナビゲーション禁止
- 動的に高さ可変を許容
- タイトルは任意（リスト/ドキュメント系は推奨）

**Inline Carousel:**
- 3〜8 アイテム
- 画像/ビジュアル必須
- メタデータ最大 2行
- 各アイテムに CTA 最大 1つ

### Fullscreen

没入型体験（マップ、エディタ、詳細ブラウジング）。

```typescript
await window.openai.requestDisplayMode({ mode: "fullscreen" });
```

- ChatGPT コンポーザーが常に下部に表示
- 内部ナビゲーション許容（ただし最小限に）

### Picture-in-Picture (PiP)

並行アクティビティ用のフローティングウィンドウ。

- ゲーム、ビデオ、ライブセッション
- スクロールで viewport 上部に固定
- **モバイルでは PiP は fullscreen に強制変換**

## Three-Tier State Model

### Tier 1: Business Data（権威データ）

サーバー/バックエンドに存在。ツール呼び出しで取得・更新。

```
User action → UI calls server tool → Server updates data → Returns snapshot → Widget re-renders
```

### Tier 2: Widget State（セッション永続）

`window.openai.widgetState` / `window.openai.setWidgetState()` で管理。
ツール呼び出し間で保持される。選択状態、ソート順、展開パネルなど。

### Tier 3: UI State（一時的）

React `useState` で管理。ホバー状態、入力中テキスト、アニメーション状態。

### State Flow

```
Server (authoritative data)
    |
ChatGPT Widget
├── Widget State (session-persistent)
└── UI State (ephemeral)
    |
    Rendered View = authoritative data + widget state + UI state
```

## Tool Response Structure

すべてのツールレスポンスは 3 部構成：

| フィールド | 可視範囲 | 用途 |
|-----------|---------|------|
| `structuredContent` | Model + Widget | 簡潔なJSON（outputSchema対応） |
| `content` | Model + Widget | Markdown/テキスト説明文 |
| `_meta` | Widget のみ | 大量/機密データ（モデルには非表示） |

### Decoupled Data + Render パターン

iframe の不要なリマウントを防ぐため、データ取得とレンダリングを分離：

```typescript
// データツール: outputTemplate なし
registerAppTool(server, "fetch_data", {
  title: "Fetch data",
  inputSchema: { query: z.string() },
}, async ({ query }) => ({
  structuredContent: { results: [...] },
  content: [{ type: "text", text: `Found ${results.length} items.` }],
}));

// レンダーツール: outputTemplate あり
registerAppTool(server, "render_widget", {
  title: "Render widget",
  description: "First call fetch_data, then pass its data to this tool.",
  inputSchema: { results: z.array(z.object({...})) },
  _meta: {
    ui: { resourceUri: TEMPLATE_URI },
    "openai/outputTemplate": TEMPLATE_URI,
  },
}, async ({ results }) => ({
  structuredContent: { results },
  content: [{ type: "text", text: `Showing ${results.length} items.` }],
}));
```

## Tool Metadata 完全リファレンス

```typescript
{
  _meta: {
    // MCP Apps 標準フィールド
    ui: {
      resourceUri: "ui://widget/my.html",
      visibility: ["model", "app"],
      prefersBorder: true,
      domain: "https://myapp.example.com",
      csp: {
        connectDomains: ["https://api.example.com"],
        resourceDomains: ["https://cdn.example.com"],
        frameDomains: ["https://*.embed.com"],  // 厳格審査
      },
    },
    // ChatGPT 拡張フィールド
    "openai/outputTemplate": "ui://widget/my.html",
    "openai/toolInvocation/invoking": "読み込み中...",   // <=64文字
    "openai/toolInvocation/invoked": "完了",             // <=64文字
    "openai/fileParams": ["imageField"],
    "openai/widgetDescription": "Shows interactive...",
    "openai/widgetSessionId": "...",
    "openai/closeWidget": true,
  },
  annotations: {
    readOnlyHint: boolean,
    destructiveHint: boolean,
    openWorldHint: boolean,
    idempotentHint: boolean,
  },
}
```

## Client-Supplied Metadata（ChatGPT ホストから提供）

```typescript
_meta["openai/locale"]          // BCP 47 ロケール文字列
_meta["openai/userAgent"]       // ユーザーエージェントヒント
_meta["openai/userLocation"]    // 粗い位置情報
_meta["openai/subject"]         // 匿名化ユーザーID
_meta["openai/session"]         // 匿名化会話ID
_meta["openai/widgetSessionId"] // 安定したウィジェットインスタンスID
```

**注意:** これらはヒントであり、認可判断に使用してはならない。

## @openai/apps-sdk-ui コンポーネントライブラリ

### インストール

```bash
npm install @openai/apps-sdk-ui
```

### CSS セットアップ

```css
@import "tailwindcss";
@import "@openai/apps-sdk-ui/css";
@source "../node_modules/@openai/apps-sdk-ui";
@source ".";
```

### 利用可能なコンポーネント

| Component | Import | 主な Props |
|-----------|--------|-----------|
| Badge | `@openai/apps-sdk-ui/components/Badge` | variant, color, pill |
| Button | `@openai/apps-sdk-ui/components/Button` | color, variant, size, block, uniform |
| CodeBlock | `@openai/apps-sdk-ui/components/CodeBlock` | language, showLineNumbers, wrapLongLines |
| Checkbox | `@openai/apps-sdk-ui/components/Checkbox` | checked, onCheckedChange, label |
| Input | `@openai/apps-sdk-ui/components/Input` | (standard input props) |
| Textarea | `@openai/apps-sdk-ui/components/Textarea` | rows, (standard textarea props) |
| Image | `@openai/apps-sdk-ui/components/Image` | src, alt, className |

### Button バリエーション

```tsx
// Color: primary, secondary
// Variant: solid, outline, soft, ghost
// Size: sm, md

<Button color="primary" variant="solid" size="md">Primary</Button>
<Button color="secondary" variant="outline" size="sm">Secondary</Button>
<Button color="secondary" variant="soft" size="sm" uniform>Icon</Button>
```

### Badge バリエーション

```tsx
// Variant: soft, solid
// Color: primary, secondary, info, danger

<Badge variant="soft" color="info" pill>Info</Badge>
<Badge variant="solid" color="primary" pill>Active</Badge>
```

### Design Tokens（CSS カスタムプロパティ）

Apps SDK UI はグレースケール (`--gray-0` 〜 `--gray-1000`)、アルファ透過、セマンティックカラートークンを提供。Tailwind のユーティリティクラスと組み合わせて使用する。

セマンティッククラス例:
- `bg-surface` — 背景サーフェス
- `border-default` — デフォルトボーダー
- `text-primary` — プライマリテキスト
- `text-secondary` — セカンダリテキスト
- `bg-subtle` — サブトル背景

## Visual Design Rules

### カラー
- システム定義パレットを使用（テキスト、アイコン、空間要素）
- ブランドアクセントはロゴ、アイコン、プライマリボタンのみ
- カスタムグラデーション、パターンオーバーレイ禁止
- 背景色・テキスト色のブランドカラーオーバーライド禁止

### タイポグラフィ
- プラットフォームネイティブフォント継承（SF Pro / Roboto / system-ui）
- **カスタムフォント禁止**（いかなる表示モードでも）
- bold/italic/highlight はコンテンツ領域内のみ
- body (16px) と body-small (14px) を基本に

### スペーシング
- システムグリッド: 4px 単位
- 適切なパディング（最小 12px）
- システム角丸仕様に従う

### アイコン・画像
- モノクロ・アウトラインスタイルのシステムアイコン
- アプリロゴを UI に含めない（ChatGPT が自動付与）
- 画像のアスペクト比統一
- すべての画像に alt テキスト

## Accessibility Requirements

- WCAG AA コントラスト比（通常テキスト 4.5:1、大テキスト 3:1）
- テキスト 200% リサイズでレイアウト崩壊しない
- タップターゲット最小 44x44px
- セマンティック HTML + ARIA ラベル
- キーボードフルアクセス（Tab / Enter / Escape）

## UX 3 原則

### 1. Conversational Leverage（会話活用）
- UI は会話を補完するもの。置き換えではない
- 自然言語で十分な情報は UI に出さない
- フォーム入力より対話で情報収集

### 2. Native Integration（ネイティブ統合）
- ChatGPT に溶け込む外観・挙動
- ブランディングを主張しすぎない
- アプリアイコン・ラベルは ChatGPT が付与

### 3. Composability（構成可能性）
- 1 UI = 1 目的
- 最小限の入出力
- 複数アプリとの組み合わせを阻害しない

## Anti-Patterns

- 長文の静的コンテンツ → 要約カード + 会話で詳細
- 複雑なマルチステップウィザード → Fullscreen か会話で段階的に
- 広告・プロモーション → 機能価値で訴求
- 機密データのカード表示 → 要約のみ、詳細は認証後
- ChatGPT ネイティブ機能の重複 → プラットフォーム機能を活用
