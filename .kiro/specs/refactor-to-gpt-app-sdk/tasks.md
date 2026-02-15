# 実装タスク: refactor-to-gpt-app-sdk

## タスク1: MCPサーバープロジェクトの初期化と基盤セットアップ (P)

**対応要件**: 1.1, 1.5, 8.2, 8.5

`mcp-server/` ディレクトリに Node.js プロジェクトを作成し、MCP SDK、Express、Zod 等の依存関係を設定する。Rust バックエンドとは独立して起動・停止できるプロジェクト構造を確立する。

### タスク1.1

- [x] `mcp-server/package.json` に依存関係（`@modelcontextprotocol/sdk`, `@modelcontextprotocol/ext-apps`, `express`, `cors`, `zod`, `pino`）と devDependencies（`typescript`, `tsx`, `vitest`, `vite`, `vite-plugin-singlefile`）を定義し、`build`、`start`、`dev`、`test` スクリプトを設定する。`mcp-server/tsconfig.json` を作成する。

### タスク1.2

- [x] 環境変数を読み込んで `BackendConfig` 型として返す設定モジュール（`mcp-server/src/config.ts`）と、pino ベースの構造化 JSON ロガー（`mcp-server/src/logger.ts`）を実装する。`.env.example` に全環境変数（`PORT`, `RUST_BACKEND_URL`, `MCP_INTERNAL_API_KEY`, `AUTH0_DOMAIN`, `AUTH0_AUDIENCE`, `PUBLIC_URL`, `LOG_LEVEL`）を文書化する。

---

## タスク2: TypeScript 型定義の実装 (P)

**対応要件**: 2.2, 5.4

- [x] Rust バックエンドのレスポンス型（`GptSearchResponse`, `GptAuthResponse`, `GptTaskResponse`, `GptCompleteTaskResponse`, `GptRunServiceResponse`, `GptUserStatusResponse`, `GptPreferencesResponse`, `GptSetPreferencesResponse`）に対応する TypeScript インターフェースを `mcp-server/src/types.ts` に定義する。リクエストパラメータ型、`BackendErrorResponse` 型、各ツールの Zod スキーマ定義で使用する入力型も含める。設計書コンポーネント7の型定義をそのまま実装する。

---

## タスク3: バックエンドクライアントの実装

**対応要件**: 1.3, 5.1, 5.3

- [x] `mcp-server/src/backend-client.ts` に `BackendClient` クラスを実装する。`Authorization: Bearer {mcpInternalApiKey}` ヘッダーを付与して Rust `/gpt/*` エンドポイント群（8メソッド: `searchServices`, `authenticateUser`, `getTaskDetails`, `completeTask`, `runService`, `getUserStatus`, `getPreferences`, `setPreferences`）に HTTP リクエストを送信する。Rust バックエンドが 4xx/5xx を返した場合のエラーハンドリング（`BackendErrorResponse` パース、ネットワークエラーの `backend_unavailable` 変換）を実装する。

---

## タスク4: MCPサーバーコアの実装

**対応要件**: 1.1, 1.2, 1.4, 8.4

Express アプリケーションと MCP Streamable HTTP トランスポートを統合し、MCPサーバーのエントリポイントを構築する。

### タスク4.1

- [x] `mcp-server/src/main.ts` に Express アプリケーションを作成する。CORS ミドルウェアで `chatgpt.com`、`cdn.oaistatic.com`、`web-sandbox.oaiusercontent.com` からのリクエストを許可する。`GET /health` でステータス・バージョン・uptime を返すヘルスチェックエンドポイントを実装する。`POST /mcp` でリクエストごとに `StreamableHTTPServerTransport` を生成し、`createServer()` で McpServer を初期化して接続する。

### タスク4.2

- [x] `mcp-server/src/server.ts` に `createServer(config)` 関数を実装する。`McpServer` インスタンスを生成し、`registerAllTools(server, config)` と `registerAllResources(server)` を呼び出して全ツール・リソースを登録する。ツール・リソースの登録関数はそれぞれ `tools/index.ts` と `widgets/index.ts` からインポートする。

---

## タスク5: MCPツールの実装

**対応要件**: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.5, 3.6

8つの MCP ツールを `registerAppTool` で登録する。各ツールに Zod スキーマ、annotations、`_meta`（invoking/invoked メッセージ）を設定し、`BackendClient` 経由でレスポンスを `structuredContent` / `content` / `_meta` の3パートに分離して返す。

### タスク5.1

- [x] `mcp-server/src/tools/search-services.ts` に `search_services` ツールを実装する。`securitySchemes: [{ type: "noauth" }]` で認証不要に設定し、`readOnlyHint: true` とする。UIウィジェット紐付け（`_meta.ui.resourceUri`）を設定する。ツール登録の基本パターン（`registerAppTool` + Zod + annotations + 3パートレスポンス）をここで確立し、他ツールのテンプレートとする。

### タスク5.2

- [x] `mcp-server/src/tools/authenticate-user.ts` に `authenticate_user` ツールを実装する。OAuth トークンの `email` を使用して `BackendClient.authenticateUser()` を呼び出し、`session_token` を `_meta` に格納する（モデルには渡さない）。認証情報が不足している場合の `_meta.mcp/www_authenticate` エラーレスポンスもここで実装する。

### タスク5.3

- [x] `mcp-server/src/tools/get-task-details.ts` と `mcp-server/src/tools/complete-task.ts` を実装する。`get_task_details` は `readOnlyHint: true` で task-form ウィジェットに紐付け、`complete_task` は同意情報（`consent` オブジェクト）を含むペイロードを送信する。両ツールとも認証済みの `session_token` を使用して Rust バックエンドを呼び出す。

### タスク5.4

- [x] `mcp-server/src/tools/run-service.ts` に `run_service` ツールを実装する。`openWorldHint: true` を設定する。レスポンスの `output` フィールド（大きなデータの可能性あり）を `_meta` に格納し、`structuredContent` には `service`, `payment_mode`, `sponsored_by`, `tx_hash` のみ含める。

### タスク5.5

- [x] `mcp-server/src/tools/get-user-status.ts`、`mcp-server/src/tools/get-preferences.ts`、`mcp-server/src/tools/set-preferences.ts` を実装する。`get_user_status` は `readOnlyHint: true` で user-dashboard ウィジェットに紐付ける。`get_preferences` は `readOnlyHint: true`。`set_preferences` は preferences 配列を受け取り更新する。`mcp-server/src/tools/index.ts` に `registerAllTools()` を作成し、全8ツールを一括登録する関数をエクスポートする。

---

## タスク6: OAuth 統合

**対応要件**: 3.1, 3.2, 3.3, 3.4, 3.5, 3.7

Auth0 を OAuth 2.1 認可サーバーとして統合し、トークン検証とユーザー識別フローを実装する。

### タスク6.1

- [x] `mcp-server/src/auth/oauth-metadata.ts` に `GET /.well-known/oauth-protected-resource` エンドポイントのハンドラを実装する。`resource`（MCP サーバーの公開 URL）、`authorization_servers`（Auth0 ドメイン）、`scopes_supported` を返す。Auth0 が `/.well-known/oauth-authorization-server` を提供するため、必要に応じてプロキシまたはリダイレクトを設定する。

### タスク6.2

- [x] `mcp-server/src/auth/token-verifier.ts` に `TokenVerifier` クラスを実装する。`jwks-rsa` で Auth0 の JWKS エンドポイントから公開鍵を取得し、`jsonwebtoken` で JWT の署名・有効期限・audience を検証する。検証成功時は `AuthInfo`（`sub`, `email`, `scopes`, `token`）を返し、失敗時は `null` を返す。

### タスク6.3

- [x] 認証が必要な7ツール（`authenticate_user`, `get_task_details`, `complete_task`, `run_service`, `get_user_status`, `get_preferences`, `set_preferences`）に `securitySchemes: [{ type: "oauth2" }]` を適用する。ツールハンドラの先頭で `TokenVerifier.verify()` を呼び出し、認証失敗時は `_meta["mcp/www_authenticate"]` ヘッダーを含む `isError: true` レスポンスを返すようにする。`search_services` は `noauth` を維持する。

---

## タスク7: UIウィジェットの実装

**対応要件**: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6

ChatGPT 内でレンダリングされる3つのリッチ UI ウィジェットを実装し、MCP リソースとして登録する。

### タスク7.1

- [x] `mcp-server/vite.config.ts` に `vite-plugin-singlefile` を使用したウィジェットビルド設定を作成する。各ウィジェット HTML を自己完結型のインラインバンドルとしてビルドする。ウィジェット共通の初期化コード（`window.openai` ブリッジ取得、テーマ対応、ツール出力データ取得、状態復元、高さ通知）を共通 JS モジュールとして作成する。

### タスク7.2

- [x] `mcp-server/src/widgets/src/services-list.html` にサービス検索結果ウィジェットを実装する。`window.openai.toolOutput` から `structuredContent.services` を取得し、サービス名・スポンサー名・補助金額・カテゴリタグ・関連度スコアをカード形式で表示する。カード選択時に `callTool("get_task_details", { campaign_id })` を発火し、`setWidgetState` で選択状態を永続化する。ダーク/ライトモード対応の CSS を含める。

### タスク7.3

- [x] `mcp-server/src/widgets/src/task-form.html` にタスク完了フォームウィジェットを実装する。タスク説明テキスト、`task_input_format.required_fields` から生成される動的入力フィールド、3種の同意チェックボックス（データ共有・利用目的確認・連絡許可）、送信ボタンを配置する。送信時に `callTool("complete_task", { ... })` を発火する。`already_completed: true` の場合は完了済み表示を行う。

### タスク7.4

- [x] `mcp-server/src/widgets/src/user-dashboard.html` にユーザーダッシュボードウィジェットを実装する。ユーザー情報セクション（email、登録日）、完了済みタスク一覧（テーブル形式）、利用可能サービス一覧（カード形式、ready 状態の「実行」ボタン付き）を表示する。「実行」ボタン押下時に `callTool("run_service", { service, input })` を発火する。

### タスク7.5

- [x] `mcp-server/src/widgets/index.ts` に `registerAllResources()` 関数を実装する。`registerAppResource` で3ウィジェット（`services-list`, `task-form`, `user-dashboard`）を `text/html;profile=mcp-app` MIME タイプの MCP リソースとして登録する。ビルドされた HTML ファイルを `dist/widgets/` から読み込む。

---

## タスク8: Rust バックエンドの CORS 適応 (P)

**対応要件**: 5.1, 5.3, 6.1, 6.2, 6.3

- [x] `src/main.rs` の `cors_layer_from_env()` が参照する環境変数に MCP サーバーの URL（`MCP_SERVER_URL`）を CORS 許可オリジンとして追加する。`.env.example` に `MCP_SERVER_URL` を追記する。既存の `/gpt/*` エンドポイント群、データベーススキーマ、`verify_gpt_api_key` ミドルウェアは変更しない。`cargo test` で既存テストが全て通過することを確認する。

---

## タスク9: テストの実装

**対応要件**: 7.1, 7.2, 7.3, 7.4, 7.5

### タスク9.1

- [x] `mcp-server/__tests__/tools/` に各 MCP ツールのユニットテストを作成する。`BackendClient` をモックし、正常系（正しいレスポンス形式、3パート分離）と異常系（バックエンドエラー、認証エラー、入力バリデーションエラー）をカバーする。Vitest を使用する。

### タスク9.2

- [x] `mcp-server/__tests__/backend-client.test.ts` に BackendClient の統合テストを作成する。fetch をモックして HTTP リクエストのヘッダー（Authorization）、パス、パラメータ変換、エラーハンドリング（4xx/5xx レスポンス、ネットワークエラー）を検証する。

### タスク9.3

- [x] E2E フロー（サービス検索→ユーザー認証→タスク取得→タスク完了→サービス実行）を MCP プロトコル経由で実行するテストを作成する。MCP Inspector またはプログラマティックな MCP クライアントを使用する。UIウィジェットの手動検証手順（ngrok + ChatGPT 開発者モードでの確認項目一覧）を `mcp-server/docs/manual-testing.md` に文書化する。

### タスク9.4*

- [x] Rust バックエンドの `cargo test` を実行し、既存テストが全て通過することを確認する。CORS 変更（タスク8）による回帰がないことを検証する。

---

## タスク10: デプロイ構成と移行手順

**対応要件**: 8.1, 8.3, 5.2, 6.4

### タスク10.1

- [x] `render.yaml` に MCP サーバーサービス（`subsidypayment-mcp`）の定義を追加する。ビルドコマンド（`cd mcp-server && npm ci && npm run build`）、起動コマンド（`cd mcp-server && npm start`）、環境変数（`RUST_BACKEND_URL`, `MCP_INTERNAL_API_KEY`, `AUTH0_DOMAIN`, `AUTH0_AUDIENCE`, `PUBLIC_URL`, `PORT`）を設定する。

### タスク10.2

- [x] ChatGPT App Directory 提出用のメタデータ（アプリ名、説明文、カテゴリ、プライバシーポリシー URL、利用規約 URL、アイコン画像の仕様）を `mcp-server/app-metadata.json` に準備する。

### タスク10.3

- [x] Actions → Apps SDK 完全移行後の削除手順書を `mcp-server/docs/migration-guide.md` に作成する。削除対象（`openapi.yaml`、`/.well-known/openapi.yaml` エンドポイント、GPT Builder 設定）、移行確認チェックリスト、ロールバック手順を含める。

---

## 要件カバレッジマトリクス

| 要件ID | タスク |
|---|---|
| 1.1 | 1.1, 4.1, 4.2 |
| 1.2 | 4.1 |
| 1.3 | 3 |
| 1.4 | 4.1 |
| 1.5 | 1.1 |
| 2.1 | 5.1, 5.2, 5.3, 5.4, 5.5 |
| 2.2 | 2, 5.1 |
| 2.3 | 5.1, 5.2, 5.3, 5.4, 5.5 |
| 2.4 | 5.1, 5.2, 5.3, 5.4, 5.5 |
| 2.5 | 5.1, 5.2, 5.3, 5.4, 5.5 |
| 2.6 | 5.1, 5.5 |
| 3.1 | 6.1 |
| 3.2 | 6.1 |
| 3.3 | 6.1 |
| 3.4 | 6.2 |
| 3.5 | 5.2, 6.3 |
| 3.6 | 5.1 |
| 3.7 | 6.2, 6.3 |
| 4.1 | 7.5 |
| 4.2 | 7.2 |
| 4.3 | 7.3 |
| 4.4 | 7.4 |
| 4.5 | 7.2, 7.3, 7.4 |
| 4.6 | 7.2, 7.3, 7.4 |
| 5.1 | 3, 8 |
| 5.2 | 10.3 |
| 5.3 | 3, 8 |
| 5.4 | 2 |
| 6.1 | 8 |
| 6.2 | 8 |
| 6.3 | 8 |
| 6.4 | 10.3 |
| 7.1 | 9.1 |
| 7.2 | 9.2 |
| 7.3 | 9.3 |
| 7.4 | 9.4 |
| 7.5 | 9.3 |
| 8.1 | 10.1 |
| 8.2 | 1.2 |
| 8.3 | 10.2 |
| 8.4 | 4.1 |
| 8.5 | 1.2 |

## タスク依存関係

```
タスク1 (P) ─┬─→ タスク3 ──→ タスク5 ──→ タスク6 ──→ タスク9
             │                                         ↑
タスク2 (P) ─┘                                         │
                                                        │
タスク4 ──────→ タスク5                                  │
              ──→ タスク7 ────────────────────────────→ タスク9
                                                        │
タスク8 (P) ───────────────────────────────────────────→ タスク9
                                                        ↓
                                                   タスク10
```

- **(P)**: 他タスクと並行実行可能
- タスク1, 2, 8 は相互に独立しており並列実行可能
- タスク3 はタスク1（プロジェクト初期化）とタスク2（型定義）に依存
- タスク4 はタスク1（プロジェクト初期化）に依存
- タスク5 はタスク3（BackendClient）とタスク4（MCPサーバーコア）に依存
- タスク6 はタスク4（MCPサーバーコア）に依存
- タスク7 はタスク4（MCPサーバーコア）に依存
- タスク9 はタスク5, 6, 7, 8 に依存
- タスク10 はタスク9 完了後を推奨（ただし10.1は早期着手可能）
