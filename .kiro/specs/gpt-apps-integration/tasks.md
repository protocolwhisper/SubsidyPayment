# GPT Apps Integration — 実装タスク

## タスク1: DBマイグレーションと基盤型定義

データベーススキーマの拡張と、GPT Apps統合に必要な全Rust型定義を追加する。

### 要件カバレッジ: 3.2, 3.3, 3.4, 4.2, 6.1, 6.2, 6.3, 6.4

- [x] 1.1: `consents` テーブルのマイグレーションを作成する (P)
  `migrations/0007_consents.sql` を作成し、`consents` テーブル（id, user_id, campaign_id, consent_type, granted, purpose, retention_days, created_at）とインデックスを定義する。設計書コンポーネント8のDDLに従う。

- [x] 1.2: `users.source` カラムのマイグレーションを作成する (P)
  `migrations/0008_add_user_source.sql` を作成し、`users` テーブルに `source TEXT DEFAULT 'web'` カラムを追加する。

- [ ] 1.3: `gpt_sessions` テーブルのマイグレーションを作成する (P)
  `migrations/0009_gpt_sessions.sql` を作成し、`gpt_sessions` テーブル（token, user_id, created_at, expires_at）とインデックスを定義する。設計書コンポーネント1.5のDDLに従う。

- [ ] 1.4: `campaigns.task_schema` カラムのマイグレーションを作成する (P)
  `migrations/0010_add_task_schema.sql` を作成し、`campaigns` テーブルに `task_schema JSONB` カラムを追加する。

- [ ] 1.5: GPT用のリクエスト・レスポンス型を `src/types.rs` に追加する
  設計書のコンポーネント2〜7で定義された全型（`GptSearchParams`, `GptSearchResponse`, `GptServiceItem`, `GptAuthRequest`, `GptAuthResponse`, `GptTaskParams`, `GptTaskResponse`, `GptTaskInputFormat`, `GptCompleteTaskRequest`, `GptConsentInput`, `GptCompleteTaskResponse`, `GptRunServiceRequest`, `GptRunServiceResponse`, `GptUserStatusParams`, `GptUserStatusResponse`, `GptCompletedTaskSummary`, `GptAvailableService`）を追加する。

- [ ] 1.6: `Consent` 型と `GptSession` 型を `src/types.rs` に追加する
  設計書コンポーネント8と1.5で定義された `Consent` 型（sqlx::FromRow付き）と `GptSession` 型を追加する。

- [ ] 1.7: `AppConfig` に `gpt_actions_api_key` フィールドを追加する
  `src/types.rs` の `AppConfig` 構造体に `gpt_actions_api_key: Option<String>` を追加し、`AppConfig::from_env()` で `GPT_ACTIONS_API_KEY` 環境変数から読み込むようにする。

- [ ] 1.8: `UserProfile` に `source` フィールドを追加する
  `src/types.rs` の `UserProfile` 構造体に `source: Option<String>` フィールドを追加し、既存のSQLクエリとの後方互換性を確認する。

---

## タスク2: エラー型拡張

GPT Actions向けの認証エラーとレート制限エラーを `ApiError` に追加する。

### 要件カバレッジ: 8.3, 8.4

- [ ] 2.1: `ApiError::unauthorized()` コンストラクタを追加する
  `src/error.rs` に 401 Unauthorized を返す `unauthorized()` メソッドを追加する。レスポンスボディは既存の `ErrorBody` パターン（`{ error: { code: "unauthorized", message: "..." } }`）に従う。

- [ ] 2.2: `ApiError::rate_limited()` コンストラクタを追加する
  `src/error.rs` に 429 Too Many Requests を返す `rate_limited(retry_after_secs: u64)` メソッドを追加する。`IntoResponse` 実装で `Retry-After` ヘッダーを含める。

---

## タスク3: GPT認証ミドルウェアとセッション管理

APIキー検証ミドルウェアとセッショントークン解決ユーティリティを実装する。

### 要件カバレッジ: 3.2, 3.3, 8.3

- [ ] 3.1: `src/gpt.rs` モジュールを新設し、APIキー認証ミドルウェアを実装する
  `src/gpt.rs` を作成し、`verify_gpt_api_key` ミドルウェア関数を実装する。`Authorization: Bearer <key>` ヘッダーを `AppConfig.gpt_actions_api_key` と照合し、不一致時は `ApiError::unauthorized()` を返す。`src/main.rs` に `mod gpt;` を追加する。

- [ ] 3.2: `resolve_session()` ユーティリティを `src/gpt.rs` に実装する
  セッショントークン（UUID）を受け取り、`gpt_sessions` テーブルから有効期限内のレコードを検索して `user_id` を返す関数を実装する。トークンが無効または期限切れの場合は `ApiError::unauthorized("invalid or expired session token")` を返す。

- [ ] 3.3: 認証ミドルウェアとセッション解決のユニットテストを追加する
  有効なAPIキー、無効なAPIキー、APIキー未設定、有効なセッショントークン、期限切れトークン、存在しないトークンの各ケースをテストする。

---

## タスク4: レート制限ミドルウェア

GPTルート専用のカスタムレート制限ミドルウェアを実装する。

### 要件カバレッジ: 8.4, 8.5

- [ ] 4.1: カスタムトークンバケットレート制限ミドルウェアを `src/gpt.rs` に実装する
  `RateLimiter` 構造体（tokens, max_tokens, last_refill, refill_interval）と `rate_limit_middleware` 関数を実装する。60 req/min（1秒あたり1トークン補充、最大バースト60）の設定で、超過時に `ApiError::rate_limited()` を返す。

- [ ] 4.2: レート制限のユニットテストを追加する
  トークン消費、トークン補充、レート超過時の429レスポンスとRetry-Afterヘッダーの各ケースをテストする。

---

## タスク5: GPTハンドラ実装（サービス検索・ユーザー認証）

サービスディスカバリとユーザー登録/識別のハンドラを実装する。

### 要件カバレッジ: 2.1, 2.2, 2.3, 2.4, 3.1, 3.3, 3.4

- [ ] 5.1: `gpt_search_services` ハンドラを実装する
  `GET /gpt/services` ハンドラを `src/gpt.rs` に実装する。`campaigns`（active=true）と `sponsored_apis`（active=true）を横断検索し、`q` パラメータで名前・スポンサー名のILIKE部分一致、`category` パラメータで `target_tools` フィルタを行う。結果を `GptSearchResponse` として返す。

- [ ] 5.2: `gpt_auth` ハンドラを実装する
  `POST /gpt/auth` ハンドラを `src/gpt.rs` に実装する。メールアドレスで `users` テーブルを検索し、既存ユーザーなら取得、新規なら `source = "gpt_apps"` で挿入する。`gpt_sessions` テーブルにセッショントークンを発行・挿入し、`GptAuthResponse`（session_token含む）を返す。

- [ ] 5.3: サービス検索とユーザー認証のユニットテストを追加する
  キーワード検索、カテゴリフィルタ、空結果、新規ユーザー登録、既存ユーザー識別、セッショントークン発行の各ケースをテストする。

---

## タスク6: GPTハンドラ実装（タスク・サービス実行）

タスク詳細取得、タスク完了、サービス実行、ユーザー状態確認のハンドラを実装する。

### 要件カバレッジ: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4, 5.5, 6.2, 6.3

- [ ] 6.1: `gpt_get_tasks` ハンドラを実装する
  `GET /gpt/tasks/{campaign_id}` ハンドラを実装する。`session_token` から `resolve_session()` で `user_id` を取得し、キャンペーン情報と `has_completed_task()` による完了状態を返す。`task_schema` カラムからタスク入力フォーマットを構築し、未設定時はデフォルトフォーマットを返す。

- [ ] 6.2: `gpt_complete_task` ハンドラを実装する
  `POST /gpt/tasks/{campaign_id}/complete` ハンドラを実装する。`session_token` から `user_id` を解決し、同意情報を `consents` テーブルに記録した後、既存の `complete_task` ロジックを再利用してタスク完了を記録する。`data_sharing_agreed` が `false` の場合もタスクは記録するが、レスポンスでデータ転送がブロックされる旨を伝える。

- [ ] 6.3: `gpt_run_service` ハンドラを実装する
  `POST /gpt/services/{service}/run` ハンドラを実装する。`session_token` から `user_id` を解決し、既存の `run_proxy` ロジック（キャンペーンマッチング→タスク完了確認→スポンサー決済→レスポンス構築）を内部で再利用する。`GptRunServiceResponse` の `message` フィールドにGPTが要約しやすいサマリーを含める。

- [ ] 6.4: `gpt_user_status` ハンドラを実装する
  `GET /gpt/user/status` ハンドラを実装する。`session_token` から `user_id` を解決し、ユーザー情報、完了済みタスク一覧、利用可能サービス一覧を `GptUserStatusResponse` として返す。

- [ ] 6.5: タスク・サービス実行ハンドラのユニットテストを追加する
  タスク詳細取得（完了済み/未完了）、タスク完了（同意あり/なし）、サービス実行（スポンサー決済成功/予算不足）、ユーザー状態確認の各ケースをテストする。

---

## タスク7: ルーター統合と静的ファイル配信

GPTサブルーターを既存のAxumアプリに組み込み、OpenAPIスキーマとプライバシーポリシーの配信エンドポイントを追加する。

### 要件カバレッジ: 1.1, 1.2, 1.3, 1.4, 1.5, 6.1, 9.1, 9.2

- [ ] 7.1: GPTサブルーターを `build_app()` に組み込む
  `src/main.rs` の `build_app()` 関数に、`/gpt` プレフィックスでGPTサブルーターを `Router::nest` で追加する。認証ミドルウェアとレート制限ミドルウェアをGPTルートにのみ適用する。

- [ ] 7.2: OpenAPIスキーマファイルを作成し配信エンドポイントを追加する
  プロジェクトルートに `openapi.yaml`（OpenAPI 3.1.0）を作成する。設計書コンポーネント10の構造に従い、全6エンドポイントの `operationId`, `summary`, `description`, パラメータ、リクエストボディ、レスポンススキーマを完全に定義する。`GET /.well-known/openapi.yaml` エンドポイントを `build_app()` に追加してYAMLファイルを配信する。

- [ ] 7.3: プライバシーポリシーページを作成し配信エンドポイントを追加する
  プロジェクトルートに `privacy.html` を作成する。設計書コンポーネント11の内容（サービス概要、収集データ、利用目的、共有条件、保持期間、ユーザー権利、連絡先）を含める。`GET /privacy` エンドポイントを `build_app()` に追加してHTMLを配信する。

- [ ] 7.4: `.env.example` に `GPT_ACTIONS_API_KEY` を追加する
  `.env.example` ファイルに `GPT_ACTIONS_API_KEY` 環境変数の説明とサンプル値を追加する。

- [ ] 7.5: ルーター統合の統合テストを追加する
  GPTサブルーターへのリクエストが認証ミドルウェアを通過すること、`/.well-known/openapi.yaml` と `/privacy` が正しいContent-Typeで応答すること、既存ルートが影響を受けないことをテストする。

---

## タスク8: GPT構成ドキュメントと最終検証

GPT Builder用のシステムプロンプト・Conversation Startersを文書化し、E2Eフローの最終検証を行う。

### 要件カバレッジ: 7.1, 7.2, 7.3, 7.4, 7.5, 9.3, 9.4

- [ ] 8.1: GPTシステムプロンプトとConversation Startersを文書化する
  設計書コンポーネント13のシステムプロンプト構造（Role & Identity, Core Behavior, Conversation Flow, Security, Constraints）とConversation Starters（4つ）を、GPT Builder UIに設定するためのドキュメントとして `.kiro/specs/gpt-apps-integration/gpt-config.md` に記録する。

- [ ] 8.2: Prometheusメトリクスにラベルを追加する
  GPTエンドポイントのリクエスト数を既存の `Metrics` 構造体で計測できるよう、各GPTハンドラで `respond()` ユーティリティを使用してメトリクスを記録する。エンドポイント名に `gpt_` プレフィックスを付与する。

- [ ] 8.3: E2Eフローの統合テストを実装する
  サービス検索→ユーザー登録→セッショントークン取得→タスク詳細取得→タスク完了（同意込み）→サービス実行の一連のフローを、テスト用DBを使用して統合テストとして実装する。

- [ ] 8.4: OpenAPIスキーマの検証を行う
  `openapi.yaml` がOpenAPI 3.1.0仕様に準拠していることを確認する。全エンドポイントのパス、パラメータ、リクエストボディ、レスポンスが実装と一致していることを検証する。

- [ ]*8.5: GPT Builder上での手動テストを実施する
  ChatGPT UIでCustom GPTを作成し、OpenAPIスキーマをインポートしてAPIキーを設定する。Conversation Startersからの会話フロー、エラーハンドリング、セッション管理が正しく動作することを手動で確認する。

---

## 要件カバレッジマトリクス

| 要件ID | タスク |
|---|---|
| 1.1 | 7.2 |
| 1.2 | 7.2 |
| 1.3 | 7.2 |
| 1.4 | 7.1 |
| 1.5 | 7.2 |
| 2.1 | 5.1 |
| 2.2 | 5.1 |
| 2.3 | 5.1 |
| 2.4 | 5.1 |
| 3.1 | 5.2 |
| 3.2 | 1.7, 3.1, 3.2 |
| 3.3 | 3.2, 5.2 |
| 3.4 | 1.2, 1.8, 5.2 |
| 4.1 | 6.1 |
| 4.2 | 1.4, 6.1 |
| 4.3 | 6.2 |
| 4.4 | 6.2 |
| 4.5 | 6.1, 6.4 |
| 4.6 | 6.2 |
| 5.1 | 6.3 |
| 5.2 | 6.3 |
| 5.3 | 6.3, 6.4 |
| 5.4 | 6.3 |
| 5.5 | 6.3 |
| 6.1 | 1.1, 7.3 |
| 6.2 | 1.1, 6.2 |
| 6.3 | 6.2 |
| 6.4 | 1.1 |
| 7.1 | 8.1 |
| 7.2 | 8.1 |
| 7.3 | 8.1 |
| 7.4 | 8.1 |
| 7.5 | 8.1 |
| 8.1 | 既存 |
| 8.2 | 既存 |
| 8.3 | 2.1, 3.1 |
| 8.4 | 2.2, 4.1 |
| 8.5 | 既存 |
| 9.1 | 7.2 |
| 9.2 | 7.4 |
| 9.3 | 8.2 |
| 9.4 | 7.1 |
