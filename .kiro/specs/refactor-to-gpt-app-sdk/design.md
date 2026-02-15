# 技術設計書: refactor-to-gpt-app-sdk

## 概要

既存の ChatGPT Actions（OpenAPI + Custom GPT）実装を、OpenAI Apps SDK（MCPプロトコル）ベースの GPT App に移行するための技術アーキテクチャを定義する。既存の Rust/Axum バックエンドのビジネスロジックは維持し、Node.js MCP サーバーをアダプター層として追加する方式を採用する。

### 要件トレーサビリティ

| 要件ID | 要件名 | 対応コンポーネント |
|---|---|---|
| 1.1–1.5 | MCPサーバー基盤 | MCP Server Entry Point, Express App |
| 2.1–2.6 | MCPツール定義 | Tool Registry（8ツール） |
| 3.1–3.7 | 認証・認可の移行 | OAuth Integration, Token Verifier |
| 4.1–4.6 | UIウィジェット | Widget Resources（3ウィジェット） |
| 5.1–5.4 | 既存コードのリファクタリング | Rust Backend Adaptation |
| 6.1–6.4 | 後方互換性・移行戦略 | Migration Strategy |
| 7.1–7.5 | テスト・検証 | テスト戦略 |
| 8.1–8.5 | デプロイ・運用 | デプロイ構成 |

---

## アーキテクチャパターン & 境界マップ

### 採用パターン: MCPアダプター層（Node.js → Rust HTTP委譲）

既存の Rust/Axum バックエンドを変更せず、Node.js MCP サーバーをアダプター層として前面に配置する。MCP サーバーはツール登録・UIリソース配信・OAuth検証を担い、ビジネスロジックの実行は既存の `/gpt/*` エンドポイントに HTTP で委譲する。

```
┌──────────────────────────────────────────────────────────────┐
│                        ChatGPT                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                 GPT App (MCP Client)                     │ │
│  │  ┌─────────────────────┐  ┌────────────────────────────┐│ │
│  │  │  UIウィジェット       │  │  AIモデル                  ││ │
│  │  │  (sandbox iframe)   │  │  (ツール呼び出し判断)       ││ │
│  │  └──────────┬──────────┘  └──────────┬─────────────────┘│ │
│  └─────────────┼───────────────────────┼──────────────────┘  │
│                │ window.openai         │ MCP (Streamable HTTP)│
└────────────────┼───────────────────────┼─────────────────────┘
                 │                       │
┌────────────────┼───────────────────────┼─────────────────────┐
│                ▼                       ▼                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │          Node.js MCP Server  (mcp-server/)              │ │
│  │                                                          │ │
│  │  ┌──────────────────────────────────────────────────┐   │ │
│  │  │  Express App (main.ts)                            │   │ │
│  │  │  ├── POST /mcp        → StreamableHTTPTransport   │   │ │
│  │  │  ├── GET  /health     → ヘルスチェック             │   │ │
│  │  │  ├── GET  /.well-known/oauth-protected-resource   │   │ │
│  │  │  └── CORS: chatgpt.com, cdn.oaistatic.com        │   │ │
│  │  └──────────────────────────────────────────────────┘   │ │
│  │                                                          │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌────────────────┐  │ │
│  │  │ OAuth Layer  │ │ Tool Registry│ │ Widget         │  │ │
│  │  │ (auth/)      │ │ (tools/)     │ │ Resources      │  │ │
│  │  │              │ │ 8 MCP tools  │ │ (widgets/)     │  │ │
│  │  │ Token verify │ │ Zod schemas  │ │ 3 HTML bundles │  │ │
│  │  │ Well-known   │ │ annotations  │ │ registerApp    │  │ │
│  │  │ metadata     │ │              │ │ Resource       │  │ │
│  │  └──────────────┘ └──────┬───────┘ └────────────────┘  │ │
│  │                          │                               │ │
│  │  ┌───────────────────────┼──────────────────────────┐   │ │
│  │  │  Backend Client (backend-client.ts)              │   │ │
│  │  │  Authorization: Bearer {MCP_INTERNAL_API_KEY}    │   │ │
│  │  │  → HTTP fetch → Rust /gpt/* endpoints            │   │ │
│  │  └──────────────────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                        │                                      │
│                Render  │ (同一リージョン)                      │
│                        │                                      │
│  ┌─────────────────────┼──────────────────────────────────┐  │
│  │                     ▼                                   │  │
│  │  Rust/Axum Backend (既存・変更最小)                      │  │
│  │  ┌───────────────────────────────────────────────────┐  │  │
│  │  │  /gpt/* サブルーター                               │  │  │
│  │  │  verify_gpt_api_key ミドルウェア（内部通信認証）      │  │  │
│  │  │  rate_limit_middleware（60 req/min）                │  │  │
│  │  │  8 ハンドラ（ビジネスロジック変更なし）               │  │  │
│  │  └───────────────────────────────────────────────────┘  │  │
│  │                                                         │  │
│  │  /campaigns, /tasks/complete, /proxy/* (変更なし)        │  │
│  └───────────────────────┬─────────────────────────────────┘  │
│                          │                                     │
│                          ▼                                     │
│                ┌──────────────────┐                            │
│                │   PostgreSQL     │                            │
│                │   (変更なし)      │                            │
│                └──────────────────┘                            │
└────────────────────────────────────────────────────────────────┘
```

### 境界定義

| 境界 | 内側 | 外側 | インターフェース |
|---|---|---|---|
| MCP プロトコル境界 | MCP Server 全体 | ChatGPT (MCP Client) | `POST /mcp` (Streamable HTTP) |
| OAuth 認証境界 | 認証済みツール（7ツール） | 公開ツール（search_services） | `securitySchemes` per tool |
| 内部通信境界 | MCP Server → Rust Backend | 外部アクセス | `Authorization: Bearer {MCP_INTERNAL_API_KEY}` |
| ウィジェットサンドボックス | iframe 内 UI | ChatGPT ホスト | `window.openai` / `App` bridge API |

---

## 技術スタック & アラインメント

### MCP Server（新規）

| 項目 | 技術 | バージョン | 理由 |
|---|---|---|---|
| ランタイム | Node.js | 20 LTS | ext-apps SDK 公式サポート |
| 言語 | TypeScript | 5.6 | 型安全、既存フロントエンドと一致 |
| MCP SDK | @modelcontextprotocol/sdk | ^1.26.0 | MCP プロトコル実装 |
| Apps 拡張 | @modelcontextprotocol/ext-apps | ^1.0.1 | registerAppTool/Resource |
| Web フレームワーク | Express | ^4.21.0 | 軽量、StreamableHTTP 対応 |
| CORS | cors | ^2.8.5 | Express CORS ミドルウェア |
| バリデーション | Zod | ^3.25.0 | ツール入力スキーマ |
| HTTP クライアント | 組み込み fetch | — | Node.js 20 標準 |
| ログ | pino | ^9.0.0 | 構造化 JSON ログ |
| ウィジェットビルド | Vite + vite-plugin-singlefile | 5.4 / ^2.0.0 | HTML インラインバンドル |
| テスト | Vitest | ^3.0.0 | TypeScript ネイティブ |

### 既存スタックとの整合性

| 項目 | 既存 | MCP Server 追加分 | 整合性 |
|---|---|---|---|
| バックエンド言語 | Rust (Axum 0.8) | TypeScript (Express) | ⚠️ 異なるが HTTP で疎結合 |
| DB | PostgreSQL (SQLx) | 直接アクセスなし | ✅ Rust 経由のみ |
| 認証 | GPT_ACTIONS_API_KEY | OAuth 2.1 (Auth0) + 内部 API キー | ✅ レイヤー分離 |
| デプロイ | Render | Render（新サービス追加） | ✅ 同一プラットフォーム |
| フロントエンド | React + Vite | Vite（ウィジェットビルド） | ✅ ビルドツール共通 |
| メトリクス | Prometheus 0.14 | pino ログ → Render ログ | ⚠️ 段階的統合 |

### 新規依存関係

```json
{
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.26.0",
    "@modelcontextprotocol/ext-apps": "^1.0.1",
    "express": "^4.21.0",
    "cors": "^2.8.5",
    "zod": "^3.25.0",
    "pino": "^9.0.0",
    "pino-pretty": "^13.0.0",
    "jsonwebtoken": "^9.0.0",
    "jwks-rsa": "^3.1.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "tsx": "^4.0.0",
    "vite": "^5.4.0",
    "vite-plugin-singlefile": "^2.0.0",
    "vitest": "^3.0.0",
    "@types/express": "^5.0.0",
    "@types/cors": "^2.8.0",
    "@types/node": "^22.0.0"
  }
}
```

---

## コンポーネント & インターフェース契約

### コンポーネント1: MCP Server Entry Point

**対応要件**: 1.1, 1.2, 1.4, 1.5, 8.4, 8.5

**責務**: Express アプリケーションの初期化、MCP トランスポートの設定、ヘルスチェック、CORS を提供する。

**ファイル**: `mcp-server/src/main.ts`

**インターフェース**:

```typescript
// Express app 初期化
// PORT: 環境変数 (デフォルト: 3001)
// CORS: chatgpt.com, cdn.oaistatic.com, web-sandbox.oaiusercontent.com

// POST /mcp — MCP Streamable HTTP エンドポイント
// リクエストごとに新しい McpServer + StreamableHTTPServerTransport を生成（ステートレス）

// GET /health — ヘルスチェック
// レスポンス: { status: "ok", version: string, uptime: number }

// GET /.well-known/oauth-protected-resource — OAuth メタデータ（コンポーネント5で詳述）
```

**動作**:
1. 環境変数から設定を読み込み（`BackendConfig` 型）
2. Express アプリを作成し CORS ミドルウェアを適用
3. `/mcp` ルートで `StreamableHTTPServerTransport` を生成
4. `createServer(config)` で McpServer を初期化（ツール・リソース登録）
5. `server.connect(transport)` → `transport.handleRequest(req, res, req.body)`

**設定型**:

```typescript
interface BackendConfig {
  /** Rust バックエンドの URL (例: https://subsidypayment.onrender.com) */
  rustBackendUrl: string;
  /** MCP→Rust 内部通信用 API キー (= GPT_ACTIONS_API_KEY) */
  mcpInternalApiKey: string;
  /** Auth0 ドメイン (例: your-tenant.auth0.com) */
  auth0Domain: string;
  /** Auth0 API identifier (audience) */
  auth0Audience: string;
  /** MCP サーバーの公開 URL */
  publicUrl: string;
  /** ポート番号 */
  port: number;
}
```

---

### コンポーネント2: MCP Server Factory

**対応要件**: 1.1, 2.1

**責務**: McpServer インスタンスを生成し、全ツールとリソースを登録する。

**ファイル**: `mcp-server/src/server.ts`

**インターフェース**:

```typescript
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

/**
 * McpServer を生成し、全ツール・リソースを登録して返す。
 * リクエストごとに呼び出される（ステートレス設計）。
 */
function createServer(config: BackendConfig): McpServer;
```

**動作**:
1. `new McpServer({ name: "subsidypayment", version: "1.0.0" })` で生成
2. `registerAllTools(server, config)` で8ツールを登録
3. `registerAllResources(server)` で3ウィジェットリソースを登録
4. McpServer インスタンスを返却

---

### コンポーネント3: Backend Client

**対応要件**: 1.3, 5.1, 5.3

**責務**: MCP ツールハンドラから Rust/Axum バックエンドの `/gpt/*` エンドポイントへ HTTP リクエストを送信し、レスポンスを型安全に返す。

**ファイル**: `mcp-server/src/backend-client.ts`

**インターフェース**:

```typescript
/**
 * Rust バックエンド /gpt/* エンドポイントへの型安全な HTTP クライアント。
 * 全メソッドは Authorization: Bearer {mcpInternalApiKey} ヘッダーを付与する。
 */
class BackendClient {
  constructor(config: BackendConfig);

  /** GET /gpt/services */
  searchServices(params: SearchServicesParams): Promise<GptSearchResponse>;

  /** POST /gpt/auth */
  authenticateUser(payload: AuthenticateUserPayload): Promise<GptAuthResponse>;

  /** GET /gpt/tasks/{campaignId} */
  getTaskDetails(campaignId: string, sessionToken: string): Promise<GptTaskResponse>;

  /** POST /gpt/tasks/{campaignId}/complete */
  completeTask(campaignId: string, payload: CompleteTaskPayload): Promise<GptCompleteTaskResponse>;

  /** POST /gpt/services/{service}/run */
  runService(service: string, payload: RunServicePayload): Promise<GptRunServiceResponse>;

  /** GET /gpt/user/status */
  getUserStatus(sessionToken: string): Promise<GptUserStatusResponse>;

  /** GET /gpt/preferences */
  getPreferences(sessionToken: string): Promise<GptPreferencesResponse>;

  /** POST /gpt/preferences */
  setPreferences(payload: SetPreferencesPayload): Promise<GptSetPreferencesResponse>;
}
```

**エラー処理**:
- Rust バックエンドが 4xx/5xx を返した場合、レスポンスボディの `error.code` と `error.message` を取得
- MCP ツールの `isError: true` レスポンスに変換
- ネットワークエラーは `{ code: "backend_unavailable", message: "..." }` に変換

---

### コンポーネント4: MCP ツール群

**対応要件**: 2.1–2.6, 3.5, 3.6

**責務**: 8つの MCP ツールを `registerAppTool` で登録し、BackendClient 経由で Rust バックエンドに委譲する。

**ファイル**: `mcp-server/src/tools/` ディレクトリ

#### ツール一覧と定義

**4.1: search_services** — `tools/search-services.ts`

```typescript
// 対応要件: 2.1, 2.3, 2.6, 3.6
registerAppTool(server, "search_services", {
  title: "サービス検索",
  description: "利用可能なスポンサー付きサービスを検索する。ユーザーがサービスを探している時に呼び出す。",
  inputSchema: {
    q: z.string().optional().describe("検索キーワード"),
    category: z.string().optional().describe("カテゴリフィルタ"),
    max_budget_cents: z.number().optional().describe("最大予算(セント)"),
    intent: z.string().optional().describe("検索意図"),
  },
  annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "noauth" }],  // 認証不要
  _meta: {
    ui: { resourceUri: "ui://widget/services-list.html" },
    "openai/toolInvocation/invoking": "サービスを検索中...",
    "openai/toolInvocation/invoked": "サービスが見つかりました",
  },
}, handler);

// handler 戻り値:
// structuredContent: { services, total_count, applied_filters, available_categories }
// content: [{ type: "text", text: response.message }]
// _meta: { full_response: response }
```

**4.2: authenticate_user** — `tools/authenticate-user.ts`

```typescript
// 対応要件: 2.1, 2.3, 3.7
registerAppTool(server, "authenticate_user", {
  title: "ユーザー認証",
  description: "ユーザーを登録または識別する。サービス利用前にメールアドレスとリージョンで登録する。",
  inputSchema: {
    email: z.string().email(),
    region: z.string(),
    roles: z.array(z.string()).optional().default([]),
    tools_used: z.array(z.string()).optional().default([]),
  },
  annotations: { readOnlyHint: false, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["user.write"] }],
  _meta: {
    "openai/toolInvocation/invoking": "ユーザーを認証中...",
    "openai/toolInvocation/invoked": "認証が完了しました",
  },
}, handler);

// handler 戻り値:
// structuredContent: { user_id, email, is_new_user }
// content: [{ type: "text", text: response.message }]
// _meta: { session_token: response.session_token }  // ウィジェット専用
```

**4.3: get_task_details** — `tools/get-task-details.ts`

```typescript
// 対応要件: 2.1, 2.3, 2.6
registerAppTool(server, "get_task_details", {
  title: "タスク詳細取得",
  description: "キャンペーンの必要タスク詳細を取得する。ユーザーがサービスを選択した後に呼び出す。",
  inputSchema: {
    campaign_id: z.string().uuid(),
  },
  annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["tasks.read"] }],
  _meta: {
    ui: { resourceUri: "ui://widget/task-form.html" },
    "openai/toolInvocation/invoking": "タスク情報を取得中...",
    "openai/toolInvocation/invoked": "タスク情報を取得しました",
  },
}, handler);
```

**4.4: complete_task** — `tools/complete-task.ts`

```typescript
// 対応要件: 2.1, 2.3
registerAppTool(server, "complete_task", {
  title: "タスク完了",
  description: "タスクを完了し同意を記録する。ユーザーがタスクに必要な情報を提供した後に呼び出す。",
  inputSchema: {
    campaign_id: z.string().uuid(),
    task_name: z.string(),
    details: z.string().optional(),
    consent: z.object({
      data_sharing_agreed: z.boolean(),
      purpose_acknowledged: z.boolean(),
      contact_permission: z.boolean(),
    }),
  },
  annotations: { readOnlyHint: false, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["tasks.write"] }],
  _meta: {
    "openai/toolInvocation/invoking": "タスクを記録中...",
    "openai/toolInvocation/invoked": "タスクが完了しました",
  },
}, handler);
```

**4.5: run_service** — `tools/run-service.ts`

```typescript
// 対応要件: 2.1, 2.3
registerAppTool(server, "run_service", {
  title: "サービス実行",
  description: "スポンサー決済でサービスを実行する。タスク完了済みのユーザーがサービスを実行する際に呼び出す。",
  inputSchema: {
    service: z.string(),
    input: z.string(),
  },
  annotations: { readOnlyHint: false, destructiveHint: false, openWorldHint: true },
  securitySchemes: [{ type: "oauth2", scopes: ["services.execute"] }],
  _meta: {
    "openai/toolInvocation/invoking": "サービスを実行中...",
    "openai/toolInvocation/invoked": "サービスの実行が完了しました",
  },
}, handler);

// handler 戻り値:
// structuredContent: { service, payment_mode, sponsored_by, tx_hash }
// content: [{ type: "text", text: response.message }]
// _meta: { output: response.output }  // 大きなデータはウィジェット専用
```

**4.6: get_user_status** — `tools/get-user-status.ts`

```typescript
// 対応要件: 2.1, 2.3, 2.6
registerAppTool(server, "get_user_status", {
  title: "ユーザー状態確認",
  description: "ユーザーの登録状態、完了済みタスク、利用可能サービスを一括で確認する。",
  inputSchema: {},
  annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["user.read"] }],
  _meta: {
    ui: { resourceUri: "ui://widget/user-dashboard.html" },
    "openai/toolInvocation/invoking": "ステータスを確認中...",
    "openai/toolInvocation/invoked": "ステータスを取得しました",
  },
}, handler);
```

**4.7: get_preferences** — `tools/get-preferences.ts`

```typescript
// 対応要件: 2.1, 2.3, 2.6
registerAppTool(server, "get_preferences", {
  title: "設定取得",
  description: "ユーザーのタスク設定（好み）を取得する。",
  inputSchema: {},
  annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["user.read"] }],
  _meta: {
    "openai/toolInvocation/invoking": "設定を取得中...",
    "openai/toolInvocation/invoked": "設定を取得しました",
  },
}, handler);
```

**4.8: set_preferences** — `tools/set-preferences.ts`

```typescript
// 対応要件: 2.1, 2.3
registerAppTool(server, "set_preferences", {
  title: "設定変更",
  description: "ユーザーのタスク設定（好み）を更新する。preferred/neutral/avoided のレベルで設定。",
  inputSchema: {
    preferences: z.array(z.object({
      task_type: z.string(),
      level: z.enum(["preferred", "neutral", "avoided"]),
    })),
  },
  annotations: { readOnlyHint: false, destructiveHint: false, openWorldHint: false },
  securitySchemes: [{ type: "oauth2", scopes: ["user.write"] }],
  _meta: {
    "openai/toolInvocation/invoking": "設定を更新中...",
    "openai/toolInvocation/invoked": "設定を更新しました",
  },
}, handler);
```

#### ツールハンドラ共通パターン

```typescript
// 認証済みツールの共通フロー:
// 1. authInfo からユーザー情報を取得（OAuth トークンの sub/email）
// 2. BackendClient の authenticateUser() でセッショントークンを取得/更新
// 3. セッショントークンを使用して Rust バックエンドの該当エンドポイントを呼び出し
// 4. レスポンスを structuredContent / content / _meta に分離して返却

// エラー時:
// - 認証エラー → isError: true, _meta に WWW-Authenticate ヘッダー情報
// - バックエンドエラー → isError: true, content にエラーメッセージ
```

---

### コンポーネント5: OAuth Integration

**対応要件**: 3.1–3.7

**責務**: Auth0 をOAuth 2.1 認可サーバーとして統合し、トークン検証・ユーザー識別を行う。

**ファイル**: `mcp-server/src/auth/`

#### 5.1: OAuth メタデータエンドポイント — `auth/oauth-metadata.ts`

**対応要件**: 3.1, 3.2

```typescript
// GET /.well-known/oauth-protected-resource
// レスポンス:
interface OAuthProtectedResourceMetadata {
  resource: string;  // MCP サーバーの公開 URL
  authorization_servers: string[];  // [Auth0 ドメイン URL]
  scopes_supported: string[];  // ["user.read", "user.write", "tasks.read", "tasks.write", "services.execute"]
}

// GET /.well-known/oauth-authorization-server は Auth0 が提供
// Auth0 ドメイン: https://{AUTH0_DOMAIN}/.well-known/oauth-authorization-server
```

#### 5.2: トークン検証 — `auth/token-verifier.ts`

**対応要件**: 3.4, 3.5, 3.7

```typescript
/**
 * Auth0 発行の JWT アクセストークンを検証する。
 * JWKS エンドポイントから公開鍵を取得し、署名・有効期限・audience を検証。
 */
class TokenVerifier {
  constructor(config: { domain: string; audience: string });

  /**
   * Bearer トークンを検証し、ユーザー情報を返す。
   * 検証失敗時は null を返す。
   */
  verify(token: string): Promise<AuthInfo | null>;
}

interface AuthInfo {
  /** Auth0 の sub クレーム (例: "auth0|abc123") */
  sub: string;
  /** メールアドレス（Auth0 のカスタムクレームまたは userinfo から） */
  email: string;
  /** 認可されたスコープ */
  scopes: string[];
  /** 生のアクセストークン */
  token: string;
}
```

#### 5.3: ユーザー識別フロー

```
OAuth トークン受信
    ↓
TokenVerifier.verify(token)
    ↓ AuthInfo { sub, email, scopes }
    ↓
BackendClient.authenticateUser({ email, region: "auto" })
    ↓ GptAuthResponse { session_token, user_id }
    ↓
session_token を使用して以降の BackendClient メソッドを呼び出し
```

**設計判断**: OAuth の `sub` クレームではなく `email` をキーとして既存の `users` テーブルと統合する。これにより既存の Actions 経由ユーザーとの互換性を維持できる。将来的に `users` テーブルに `auth0_sub` カラムを追加してより堅牢な識別に移行可能。

#### 5.4: Auth0 設定要件

**対応要件**: 3.3, 3.4

| 項目 | 設定値 |
|---|---|
| アプリケーション種別 | Regular Web Application |
| Dynamic Client Registration | 有効 |
| PKCE | S256 必須 |
| Allowed Callback URLs | `https://chatgpt.com/connector_platform_oauth_redirect`, `https://platform.openai.com/apps-manage/oauth` |
| Token Endpoint Auth Method | `none`（パブリッククライアント） |
| API Audience | `https://subsidypayment-mcp.onrender.com` |
| スコープ | `user.read`, `user.write`, `tasks.read`, `tasks.write`, `services.execute` |

#### 5.5: 認証エラーレスポンス

**対応要件**: 3.5

```typescript
// 認証が必要なツールで認証情報が不足している場合:
return {
  content: [{ type: "text", text: "このアクションを実行するにはログインが必要です。" }],
  _meta: {
    "mcp/www_authenticate": [
      `Bearer resource_metadata="${config.publicUrl}/.well-known/oauth-protected-resource"`
    ]
  },
  isError: true,
};
```

---

### コンポーネント6: UIウィジェットリソース

**対応要件**: 4.1–4.6

**責務**: ChatGPT 内にレンダリングされるリッチ UI を MCP リソースとして提供する。

**ファイル**: `mcp-server/src/widgets/`

#### 6.1: リソース登録パターン

```typescript
import {
  registerAppResource,
  RESOURCE_MIME_TYPE,
} from "@modelcontextprotocol/ext-apps/server";
import fs from "node:fs/promises";

// 各ウィジェットの HTML は Vite でビルドされたインラインバンドル
// ビルド成果物: mcp-server/dist/widgets/*.html

registerAppResource(
  server,
  "services-list",                           // リソース名
  "ui://widget/services-list.html",           // URI
  { mimeType: RESOURCE_MIME_TYPE },           // "text/html;profile=mcp-app"
  async () => ({
    contents: [{
      uri: "ui://widget/services-list.html",
      mimeType: RESOURCE_MIME_TYPE,
      text: await fs.readFile("dist/widgets/services-list.html", "utf-8"),
      _meta: {
        ui: {
          prefersBorder: true,
          csp: {
            connectDomains: [],  // ウィジェットから直接 API 呼び出しなし（callTool 経由）
          },
        },
      },
    }],
  })
);
```

#### 6.2: サービス検索結果ウィジェット — `widgets/services-list.html`

**対応要件**: 4.2, 4.5, 4.6

**UI構成**:
- サービスカード一覧（グリッドレイアウト）
- 各カード: サービス名、スポンサー名、補助金額、カテゴリタグ、関連度スコア
- カード選択 → `callTool("get_task_details", { campaign_id })` を発火
- `setWidgetState({ selectedServiceId })` で選択状態を永続化

**データソース**:
- `window.openai.toolOutput` → `structuredContent.services` 配列

#### 6.3: タスク完了フォームウィジェット — `widgets/task-form.html`

**対応要件**: 4.3, 4.5, 4.6

**UI構成**:
- タスク説明テキスト
- 動的入力フィールド（`task_input_format.required_fields` から生成）
- 同意チェックボックス3種:
  - データ共有への同意
  - 利用目的の確認
  - 連絡許可
- 送信ボタン → `callTool("complete_task", { ... })` を発火
- 完了済み表示（`already_completed: true` の場合）

**データソース**:
- `window.openai.toolOutput` → `structuredContent` (タスク詳細)

#### 6.4: ユーザーステータスダッシュボード — `widgets/user-dashboard.html`

**対応要件**: 4.4, 4.5, 4.6

**UI構成**:
- ユーザー情報セクション（email、登録日）
- 完了済みタスク一覧（テーブル形式: キャンペーン名、タスク名、完了日）
- 利用可能サービス一覧（カード: サービス名、スポンサー、ready状態）
- ready なサービスの「実行」ボタン → `callTool("run_service", { service, input })` を発火
- `sendFollowUpMessage` でフォローアップ提案

**データソース**:
- `window.openai.toolOutput` → `structuredContent` (ステータス情報)

#### 6.5: ウィジェット共通設計

```typescript
// ウィジェット内の共通初期化コード:
const app = window.openai;

// テーマ対応
const isDark = app.theme?.appearance === "dark";
document.body.classList.toggle("dark", isDark);

// ツール出力からデータを取得
const data = app.toolOutput;

// 状態の復元
const savedState = app.widgetState;

// レスポンシブ対応
const maxHeight = app.maxHeight;
document.body.style.maxHeight = `${maxHeight}px`;

// 高さ通知
app.notifyIntrinsicHeight(document.body.scrollHeight);
```

**スタイル方針**:
- CSS カスタムプロパティでダーク/ライトモード対応
- `max-width: 100%` でモバイル対応
- フォント: system-ui フォールバック
- カラー: 中立的なグレースケール + アクセントカラー

---

### コンポーネント7: TypeScript 型定義

**対応要件**: 2.2, 5.4

**責務**: Rust バックエンドのレスポンス型を TypeScript インターフェースとして定義し、型安全を保証する。

**ファイル**: `mcp-server/src/types.ts`

```typescript
// --- Rust バックエンドのレスポンス型に対応 ---

interface GptSearchResponse {
  services: GptServiceItem[];
  total_count: number;
  message: string;
  applied_filters?: AppliedFilters;
  available_categories?: string[];
}

interface GptServiceItem {
  service_type: "campaign" | "sponsored_api";
  service_id: string;
  name: string;
  sponsor: string;
  required_task: string | null;
  subsidy_amount_cents: number;
  category: string[];
  active: boolean;
  tags: string[];
  relevance_score: number | null;
}

interface AppliedFilters {
  budget: number | null;
  intent: string | null;
  category: string | null;
  keyword: string | null;
  preferences_applied: boolean;
}

interface GptAuthResponse {
  session_token: string;
  user_id: string;
  email: string;
  is_new_user: boolean;
  message: string;
}

interface GptTaskResponse {
  campaign_id: string;
  campaign_name: string;
  sponsor: string;
  required_task: string;
  task_description: string;
  task_input_format: GptTaskInputFormat;
  already_completed: boolean;
  subsidy_amount_cents: number;
  message: string;
}

interface GptTaskInputFormat {
  task_type: string;
  required_fields: string[];
  instructions: string;
}

interface GptCompleteTaskResponse {
  task_completion_id: string;
  campaign_id: string;
  consent_recorded: boolean;
  can_use_service: boolean;
  message: string;
}

interface GptRunServiceResponse {
  service: string;
  output: string;
  payment_mode: "sponsored" | "user_direct";
  sponsored_by: string | null;
  tx_hash: string | null;
  message: string;
}

interface GptUserStatusResponse {
  user_id: string;
  email: string;
  completed_tasks: GptCompletedTaskSummary[];
  available_services: GptAvailableService[];
  message: string;
}

interface GptCompletedTaskSummary {
  campaign_id: string;
  campaign_name: string;
  task_name: string;
  completed_at: string;
}

interface GptAvailableService {
  service: string;
  sponsor: string;
  ready: boolean;
}

interface GptPreferencesResponse {
  user_id: string;
  preferences: TaskPreference[];
  updated_at: string | null;
  message: string;
}

interface GptSetPreferencesResponse {
  user_id: string;
  preferences_count: number;
  updated_at: string;
  message: string;
}

interface TaskPreference {
  task_type: string;
  level: "preferred" | "neutral" | "avoided";
}

// --- Backend エラーレスポンス ---

interface BackendErrorResponse {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}
```

---

### コンポーネント8: Rust バックエンド適応

**対応要件**: 5.1, 5.2, 5.3, 5.4, 6.2, 6.3

**責務**: 既存の Rust/Axum バックエンドに最小限の変更を加え、MCP サーバーからの内部通信を受け付ける。

**変更ファイル**:

| ファイル | 変更内容 |
|---|---|
| `src/main.rs` | CORS 許可オリジンに MCP サーバー URL を追加（環境変数経由） |
| `.env.example` | `MCP_SERVER_URL` を追加 |

**変更しないファイル**（明示的に変更不要）:

| ファイル | 理由 |
|---|---|
| `src/gpt.rs` | 全ハンドラ変更なし。MCP サーバーは既存エンドポイントを HTTP で呼び出す |
| `src/types.rs` | 全型定義変更なし。レスポンスは `structuredContent` としてそのまま利用 |
| `src/error.rs` | エラー型変更なし。MCP 側でエラーマッピング |
| `openapi.yaml` | 移行完了まで維持。完了後に削除（要件 5.2） |

---

## データフロー

### E2E フロー: サービス検索 → タスク実行 → サービス実行

```
ユーザー → ChatGPT → [search_services ツール]
                        ↓ MCP Streamable HTTP
                      MCP Server
                        ↓ fetch GET /gpt/services?q=...
                      Rust Backend → DB(campaigns, sponsored_apis)
                        ↓
ユーザー ← ChatGPT ← サービス一覧 + UIウィジェット(services-list)

ユーザー → ウィジェット「サービス選択」→ callTool("authenticate_user")
                        ↓ MCP
                      MCP Server
                        ↓ OAuth トークン検証 (Auth0 JWKS)
                        ↓ fetch POST /gpt/auth { email, region }
                      Rust Backend → DB(users, gpt_sessions)
                        ↓
ユーザー ← ChatGPT ← 認証完了 (session_token は _meta 内で保持)

ユーザー → ChatGPT → [get_task_details ツール]
                        ↓ MCP
                      MCP Server
                        ↓ fetch GET /gpt/tasks/{id}?session_token=xxx
                      Rust Backend → DB(campaigns, task_completions)
                        ↓
ユーザー ← ChatGPT ← タスク詳細 + UIウィジェット(task-form)

ユーザー → ウィジェット「タスク完了+同意」→ callTool("complete_task")
                        ↓ MCP
                      MCP Server
                        ↓ fetch POST /gpt/tasks/{id}/complete { session_token, ... }
                      Rust Backend → DB(task_completions, consents)
                        ↓
ユーザー ← ChatGPT ← 完了確認

ユーザー → ChatGPT → [run_service ツール]
                        ↓ MCP
                      MCP Server
                        ↓ fetch POST /gpt/services/{service}/run { session_token, input }
                      Rust Backend → キャンペーンマッチ → 予算減算 → DB(payments)
                        ↓
ユーザー ← ChatGPT ← サービス実行結果 (output は _meta 経由でウィジェットに)
```

---

## モジュール構成

### 新規ディレクトリ構造

```
mcp-server/
├── src/
│   ├── main.ts                    # Express app, /mcp, /health, CORS
│   ├── server.ts                  # McpServer factory (ツール・リソース登録)
│   ├── backend-client.ts          # Rust /gpt/* HTTP クライアント
│   ├── types.ts                   # TypeScript 型定義 (Rust レスポンス型対応)
│   ├── config.ts                  # BackendConfig 読み込み
│   ├── logger.ts                  # pino ロガー設定
│   ├── auth/
│   │   ├── oauth-metadata.ts      # /.well-known エンドポイント
│   │   └── token-verifier.ts      # Auth0 JWT 検証
│   ├── tools/
│   │   ├── index.ts               # registerAllTools()
│   │   ├── search-services.ts
│   │   ├── authenticate-user.ts
│   │   ├── get-task-details.ts
│   │   ├── complete-task.ts
│   │   ├── run-service.ts
│   │   ├── get-user-status.ts
│   │   ├── get-preferences.ts
│   │   └── set-preferences.ts
│   └── widgets/
│       ├── index.ts               # registerAllResources()
│       ├── src/                   # ウィジェットソース
│       │   ├── services-list.html
│       │   ├── task-form.html
│       │   └── user-dashboard.html
│       └── dist/                  # Vite ビルド成果物
├── package.json
├── tsconfig.json
├── vite.config.ts                 # ウィジェットビルド設定
├── .env.example
└── __tests__/
    ├── tools/
    │   ├── search-services.test.ts
    │   ├── authenticate-user.test.ts
    │   └── ...
    ├── backend-client.test.ts
    └── auth/
        └── token-verifier.test.ts
```

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `render.yaml` | MCP サーバーサービス定義を追加 |
| `.env.example` | MCP 関連の環境変数を追加 |

---

## 移行戦略

**対応要件**: 6.1, 6.4

### Phase 1: MCP 基盤 + ツール（認証なし）

1. `mcp-server/` プロジェクト作成
2. Express + StreamableHTTPServerTransport セットアップ
3. BackendClient 実装
4. 8ツールを `noauth` で登録（全ツール認証不要で開発）
5. MCP Inspector でローカル検証
6. ngrok + ChatGPT 開発者モードで動作確認

### Phase 2: OAuth 統合

1. Auth0 テナント設定（DCR、PKCE、スコープ）
2. `/.well-known/oauth-protected-resource` エンドポイント追加
3. TokenVerifier 実装
4. 7ツールに `securitySchemes: [{ type: "oauth2" }]` を設定
5. `search_services` は `noauth` を維持
6. OAuth フローの E2E 検証

### Phase 3: UIウィジェット

1. 3ウィジェット HTML/JS/CSS 作成
2. Vite ビルド設定
3. `registerAppResource` で登録
4. `_meta.ui.resourceUri` をツールに紐付け
5. ChatGPT 開発者モードで表示検証

### Phase 4: デプロイ + 公開準備

1. Render にMCPサーバーサービスを追加
2. 環境変数設定
3. App Directory メタデータ準備
4. 既存 Custom GPT（Actions）は維持（別個 GPT として並行運用）
5. 移行手順書作成

---

## セキュリティ考慮事項

| 項目 | 対策 |
|---|---|
| OAuth トークン検証 | Auth0 JWKS エンドポイントから公開鍵を取得し、署名・有効期限・audience を検証 |
| MCP→Rust 内部通信 | `MCP_INTERNAL_API_KEY`（= `GPT_ACTIONS_API_KEY`）による Bearer 認証。環境変数管理 |
| session_token の漏洩防止 | `_meta` に格納しモデルには渡さない。ウィジェット専用 |
| 入力バリデーション | Zod スキーマによるツール入力の厳格な検証 |
| CORS | `chatgpt.com`, `cdn.oaistatic.com`, `web-sandbox.oaiusercontent.com` のみ許可 |
| ウィジェットサンドボックス | iframe + CSP で外部リソースアクセスを制限 |
| 同意管理 | 既存の `consents` テーブルによる監査可能な同意記録（変更なし） |
| ログ | pino による構造化ログ。トークン値はマスキング |

---

## テスト戦略

| テストレベル | 対象 | ツール | 対応要件 |
|---|---|---|---|
| ユニットテスト | 各ツールハンドラ、BackendClient、TokenVerifier | Vitest + fetch モック | 7.1 |
| 統合テスト | MCP Server → Rust Backend 通信 | Vitest + 実Rustサーバー | 7.2 |
| E2E テスト | サービス検索→認証→タスク→実行フロー | MCP Inspector | 7.3 |
| 回帰テスト | Rust バックエンド | `cargo test` | 7.4 |
| UI テスト | ウィジェット表示・操作 | ngrok + ChatGPT 開発者モード | 7.5 |
| スキーマ検証 | ツール入力スキーマの正当性 | Zod parse テスト | 7.1 |

---

## 環境変数（MCP Server）

| 変数名 | 必須 | デフォルト | 用途 |
|---|---|---|---|
| `PORT` | No | `3001` | MCP サーバーポート |
| `RUST_BACKEND_URL` | Yes | — | Rust バックエンドの URL |
| `MCP_INTERNAL_API_KEY` | Yes | — | Rust バックエンド認証キー |
| `AUTH0_DOMAIN` | Yes | — | Auth0 テナントドメイン |
| `AUTH0_AUDIENCE` | Yes | — | Auth0 API identifier |
| `PUBLIC_URL` | Yes | — | MCP サーバーの公開 URL |
| `LOG_LEVEL` | No | `info` | ログレベル |
| `NODE_ENV` | No | `development` | 実行環境 |

---

## 後方互換性

- 既存の全 API エンドポイント（`/campaigns`, `/tasks/complete`, `/proxy/*` 等）は変更なし
- 既存の Custom GPT（ChatGPT Actions）は独立して動作し続ける
- 新しい GPT App（MCP）は別個のエントリとして ChatGPT に追加
- データベーススキーマは一切変更なし
- `openapi.yaml` と `/.well-known/openapi.yaml` は移行完了まで維持
- `verify_gpt_api_key` ミドルウェアは MCP 内部通信認証としても機能

---

## Render デプロイ構成

### render.yaml 追加サービス

```yaml
services:
  # 既存: Rust バックエンド
  - type: web
    name: subsidypayment
    # ... (既存設定)

  # 新規: MCP Server
  - type: web
    name: subsidypayment-mcp
    runtime: node
    buildCommand: cd mcp-server && npm ci && npm run build
    startCommand: cd mcp-server && npm start
    envVars:
      - key: RUST_BACKEND_URL
        fromService:
          type: web
          name: subsidypayment
          property: host
      - key: MCP_INTERNAL_API_KEY
        sync: false
      - key: AUTH0_DOMAIN
        sync: false
      - key: AUTH0_AUDIENCE
        sync: false
      - key: PUBLIC_URL
        sync: false
      - key: PORT
        value: "3001"
```
