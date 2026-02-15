# 要件定義: refactor-to-gpt-app-sdk

## プロジェクト概要

既存のChatGPT Actions（OpenAPI仕様ベースのCustom Actions）実装を、OpenAI Apps SDK（MCPプロトコルベース）に移行するリファクタリングを行う。

### 現状

- `src/gpt.rs` に7つのGPT Actionsハンドラを実装済み（`gpt_search_services`, `gpt_auth`, `gpt_get_tasks`, `gpt_complete_task`, `gpt_run_service`, `gpt_user_status`, `gpt_get_preferences`/`gpt_set_preferences`）
- `openapi.yaml` でOpenAPI 3.1.0スキーマを定義し、ChatGPT Custom GPTのCustom Actionsとして公開
- 認証: `GPT_ACTIONS_API_KEY` による単純なBearer トークン認証
- セッション管理: `gpt_sessions` テーブルによる30日間トークン
- UI: テキストのみのレスポンス（GPTが自然言語で要約）

### 目標

- OpenAI Apps SDK（MCPプロトコル）ベースのGPT App実装に移行
- 既存のRust/Axumバックエンドのビジネスロジックは維持し、MCP アダプター層を追加
- リッチUIウィジェットによるインタラクティブな体験を提供
- OAuth 2.1 + PKCEによるセキュアな認証に移行
- ChatGPT App Directoryへの公開準備

---

## 要件一覧

### 1. MCPサーバー基盤

**説明**: OpenAI Apps SDK準拠のMCPサーバーを構築し、既存のRust/Axumバックエンドと連携するアダプター層を提供する。

#### 受入基準

- **1.1**: システムは、`@modelcontextprotocol/sdk` および `@modelcontextprotocol/ext-apps` パッケージを使用したNode.js MCPサーバーを提供しなければならない（EARS: Ubiquitous）
- **1.2**: MCPサーバーは、Streamable HTTP トランスポートを介して `/mcp` エンドポイントでリクエストを受け付けなければならない（EARS: Ubiquitous）
- **1.3**: MCPサーバーは、既存のRust/Axumバックエンド（`/gpt/*` エンドポイント群）にHTTPリクエストを委譲し、レスポンスをMCP形式に変換しなければならない（EARS: Ubiquitous）
- **1.4**: MCPサーバーは、CORSヘッダーで `https://chatgpt.com` および `https://cdn.oaistatic.com` からのリクエストを許可しなければならない（EARS: Ubiquitous）
- **1.5**: MCPサーバーの起動・停止は、既存のRust/Axumバックエンドとは独立して行えなければならない（EARS: Ubiquitous）

---

### 2. MCPツール定義

**説明**: 既存のGPT Actions APIエンドポイントをMCPツールとして再定義し、Apps SDK形式のツール登録を行う。

#### 受入基準

- **2.1**: 以下の既存エンドポイントが、`registerAppTool` を使用してMCPツールとして登録されなければならない: `search_services`（サービス検索）、`authenticate_user`（ユーザー認証）、`get_task_details`（タスク詳細取得）、`complete_task`（タスク完了）、`run_service`（サービス実行）、`get_user_status`（ユーザー状態確認）、`get_preferences`（設定取得）、`set_preferences`（設定変更）（EARS: Ubiquitous）
- **2.2**: 各ツールには、Zodスキーマによる入力バリデーション定義が含まれなければならない（EARS: Ubiquitous）
- **2.3**: 各ツールには、適切な `annotations`（`readOnlyHint`, `destructiveHint`, `openWorldHint`）が設定されなければならない（EARS: Ubiquitous）
- **2.4**: 各ツールのレスポンスは、`structuredContent`（ウィジェット・モデル向けJSON）、`content`（モデル向けテキスト）、`_meta`（ウィジェット専用データ）の3パートで構成されなければならない（EARS: Ubiquitous）
- **2.5**: ツール呼び出し中のローディング表示として、`_meta` に `openai/toolInvocation/invoking` および `openai/toolInvocation/invoked` メッセージが設定されなければならない（EARS: Ubiquitous）
- **2.6**: 読み取り専用ツール（`search_services`, `get_task_details`, `get_user_status`, `get_preferences`）には `readOnlyHint: true` が設定されなければならない（EARS: Ubiquitous）

---

### 3. 認証・認可の移行

**説明**: 現在のAPIキーベースの認証からOAuth 2.1 + PKCE認証に移行し、Apps SDKのセキュリティ要件を満たす。

#### 受入基準

- **3.1**: システムは、`/.well-known/oauth-protected-resource` でOAuth保護リソースメタデータを公開しなければならない（EARS: Ubiquitous）
- **3.2**: システムは、`/.well-known/oauth-authorization-server` でOAuth認可サーバーメタデータを公開しなければならない（EARS: Ubiquitous）
- **3.3**: システムは、Dynamic Client Registration（DCR）をサポートし、ChatGPTクライアントの自動登録を受け付けなければならない（EARS: Ubiquitous）
- **3.4**: OAuth認可フローは、PKCE（S256）を必須としなければならない（EARS: Ubiquitous）
- **3.5**: 認証が必要なツールで認証情報が不足している場合、MCPサーバーは `_meta.mcp/www_authenticate` ヘッダーを含むエラーレスポンスを返さなければならない（EARS: Event-driven）
- **3.6**: 認証不要のツール（`search_services`）は、`securitySchemes: [{ type: "noauth" }]` で公開アクセスを許可しなければならない（EARS: Ubiquitous）
- **3.7**: 認証済みトークンからユーザー識別情報を抽出し、既存の `resolve_session` ロジックと統合しなければならない（EARS: Ubiquitous）

---

### 4. UIウィジェット

**説明**: Apps SDKのリソース機能を活用し、ChatGPT内にリッチなインタラクティブUIを提供する。

#### 受入基準

- **4.1**: ウィジェットリソースは、`text/html;profile=mcp-app` MIMEタイプでMCPリソースとして登録されなければならない（EARS: Ubiquitous）
- **4.2**: サービス検索結果を表示するウィジェットが提供されなければならない。サービス名、スポンサー名、補助金額、カテゴリが視覚的に表示されること（EARS: Ubiquitous）
- **4.3**: タスク完了フォームを表示するウィジェットが提供されなければならない。同意チェックボックスを含む入力フォームが表示されること（EARS: Ubiquitous）
- **4.4**: ユーザーステータスダッシュボードを表示するウィジェットが提供されなければならない。完了済みタスク、利用可能サービスが一覧表示されること（EARS: Ubiquitous）
- **4.5**: ウィジェットは、`window.openai` ブリッジAPIを使用してツール出力データを受け取り、`callTool` で追加のツール呼び出しを行えなければならない（EARS: Ubiquitous）
- **4.6**: ウィジェットは、`setWidgetState` を使用してターン間でUI状態を永続化しなければならない（EARS: Ubiquitous）

---

### 5. 既存コードのリファクタリング

**説明**: 既存のRust/AxumバックエンドをMCPアダプター層から効率的に利用できるよう、必要な整理を行う。

#### 受入基準

- **5.1**: 既存の `/gpt/*` エンドポイント群は、MCPサーバーからのHTTPリクエストを受け付けるために引き続き動作しなければならない（EARS: Ubiquitous）
- **5.2**: `openapi.yaml` ファイルおよび `/.well-known/openapi.yaml` 配信エンドポイントは、GPT App SDK移行完了後に削除されなければならない（EARS: Event-driven）
- **5.3**: GPT Actionsの `verify_gpt_api_key` ミドルウェアは、MCPサーバーからの内部通信認証に転用されるか、OAuth認証で置き換えられなければならない（EARS: Ubiquitous）
- **5.4**: 既存の `GptSearchResponse`, `GptAuthResponse` 等のレスポンス型は、MCPツールの `structuredContent` として再利用可能な形式を維持しなければならない（EARS: Ubiquitous）

---

### 6. 後方互換性・移行戦略

**説明**: 既存ユーザーとシステムへの影響を最小化しながら段階的に移行する。

#### 受入基準

- **6.1**: 移行期間中は、既存のChatGPT Actions（OpenAPI）とApps SDK（MCP）の両方が並行して動作しなければならない（EARS: State-driven）
- **6.2**: 既存のAPI エンドポイント（`/campaigns`, `/tasks/complete`, `/proxy/{service}/run` 等）は変更されてはならない（EARS: Ubiquitous）
- **6.3**: 既存のデータベーススキーマ（`gpt_sessions`, `consents`, `user_task_preferences` テーブル）は維持されなければならない（EARS: Ubiquitous）
- **6.4**: GPT App SDKへの完全移行後、ChatGPT Actions固有のコード（OpenAPIスキーマ配信、GPT Builder設定等）を削除するための手順書が提供されなければならない（EARS: Event-driven）

---

### 7. テスト・検証

**説明**: リファクタリング後の機能を検証し、既存機能の回帰がないことを確認する。

#### 受入基準

- **7.1**: 各MCPツールに対して、正常系・異常系のユニットテストが作成されなければならない（EARS: Ubiquitous）
- **7.2**: MCPサーバーからRust/Axumバックエンドへの通信を検証する統合テストが作成されなければならない（EARS: Ubiquitous）
- **7.3**: E2Eフロー（サービス検索→ユーザー認証→タスク実行→サービス実行）のテストが、MCPプロトコル経由で実行可能でなければならない（EARS: Ubiquitous）
- **7.4**: 既存のRust/Axumバックエンドのテスト（`cargo test`）が全て通過しなければならない（EARS: Ubiquitous）
- **7.5**: UIウィジェットの表示・操作が、ChatGPT開発者モード（ngrok等）で手動検証されなければならない（EARS: Ubiquitous）

---

### 8. デプロイ・運用

**説明**: MCPサーバーとUIウィジェットのデプロイ環境を整備し、ChatGPT App Directoryへの公開準備を行う。

#### 受入基準

- **8.1**: MCPサーバーは、既存のRenderデプロイ環境または新規のデプロイプラットフォームに配置可能でなければならない（EARS: Ubiquitous）
- **8.2**: MCPサーバーの環境変数（OAuth設定、バックエンドURL等）は `.env.example` に文書化されなければならない（EARS: Ubiquitous）
- **8.3**: ChatGPT App Directoryへの提出に必要なメタデータ（アプリ名、説明、アイコン、プライバシーポリシーURL）が準備されなければならない（EARS: Ubiquitous）
- **8.4**: MCPサーバーのヘルスチェックエンドポイントが提供され、既存のPrometheusメトリクスと統合されなければならない（EARS: Ubiquitous）
- **8.5**: MCPサーバーのログは、既存のRustバックエンドと統合可能な構造化ログ形式で出力されなければならない（EARS: Ubiquitous）

---

## 非ゴール（スコープ外）

- Rust/Axumバックエンドのビジネスロジック書き換え（MCPアダプター層で吸収）
- 既存の非GPTエンドポイント（`/campaigns`, `/proxy/*` 等）の変更
- 高度なUIフレームワーク（React等）によるウィジェット構築（初期はバニラHTML/JS/CSS）
- 複数のAIクライアント（Claude, Codex等）への同時対応
- GPT App SDKのRust純正実装（Node.jsアダプター層で対応）

---

## 用語集

| 用語 | 説明 |
|---|---|
| **Apps SDK** | OpenAI Apps SDK。MCPプロトコルをベースに、ChatGPT上でリッチなアプリ体験を提供するフレームワーク |
| **MCP** | Model Context Protocol。AIモデルが外部ツールやリソースと対話するための標準プロトコル |
| **ChatGPT Actions** | 旧方式。OpenAPI仕様に基づきCustom GPTが外部APIを呼び出す仕組み |
| **MCPツール** | MCPプロトコルで定義される実行可能な関数。ChatGPT Actionsの各エンドポイントに相当 |
| **MCPリソース** | MCPプロトコルで定義される静的コンテンツ。UIウィジェット等を配信 |
| **UIウィジェット** | ChatGPT内のサンドボックスiframeで表示されるリッチHTML/JS UI |
| **Streamable HTTP** | MCPの通信トランスポート方式。HTTPベースのストリーミング通信 |
| **OAuth 2.1 + PKCE** | Apps SDKが要求するセキュアな認証方式 |
| **DCR** | Dynamic Client Registration。OAuthクライアントの自動登録 |
| **`registerAppTool`** | `@modelcontextprotocol/ext-apps` が提供するツール登録API |
| **`structuredContent`** | MCPツールレスポンスの構造化データ部分。ウィジェットとモデルの両方が消費 |
| **`_meta`** | MCPツールレスポンスのメタデータ部分。ウィジェット専用で、モデルには渡されない |
| **`window.openai`** | ウィジェット内で使用するブリッジAPI。ツール呼び出し・状態永続化等を提供 |
