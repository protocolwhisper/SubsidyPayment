# Autonomous Agent Execution — 実装タスク

## タスク1: DBマイグレーションと基盤型定義

エージェントセッション管理・監査ログのデータベーステーブルと、既存テーブルへのメタデータカラム追加、全Rust型定義を行う。

### 要件カバレッジ: 2.2, 2.3, 2.5, 4.6, 5.2, 6.5, 8.1

- [ ] 1.1: `agent_sessions` テーブルのマイグレーションを作成する (P)
  `migrations/0011_agent_sessions.sql` を作成し、`agent_sessions` テーブル（id, token, user_id, agent_id, scopes, budget_limit_cents, budget_spent_cents, active, violation_count, created_at, expires_at, revoked_at）とインデックス（token, user_id, expires_at）を定義する。設計書コンポーネント2のDDLに従う。

- [ ] 1.2: `agent_audit_logs` テーブルのマイグレーションを作成する (P)
  `migrations/0012_agent_audit_logs.sql` を作成し、`agent_audit_logs` テーブル（id, agent_session_id, agent_id, user_id, operation, endpoint, request_summary, response_status, created_at）とインデックス（session+created_at, user+created_at）を定義する。設計書コンポーネント5のDDLに従う。

- [ ] 1.3: 既存テーブルへのエージェントメタデータカラムを追加する (P)
  `migrations/0013_agent_metadata.sql` を作成し、`task_completions` に `execution_mode TEXT DEFAULT 'human'` と `agent_session_id UUID`、`payments` に `agent_session_id UUID`、`consents` に `granted_via TEXT DEFAULT 'human'` と `agent_session_id UUID` を追加する。全カラムはNULLABLEまたはDEFAULT付きで後方互換を維持する。

- [ ] 1.4: エージェント用の全リクエスト・レスポンス型を `src/types.rs` に追加する
  設計書の全コンポーネントで定義されたエージェント型を追加する。`AgentSessionScopes`, `AgentSession`（sqlx::FromRow付き）, `AgentAuthRequest`, `AgentAuthData`, `AgentSearchParams`, `AgentSearchData`, `AgentServiceItem`, `AgentTaskData`, `AgentCompleteTaskRequest`, `AgentConsentInput`, `AgentCompleteTaskData`, `AgentRunServiceRequest`, `AgentRunServiceData`, `AgentExecutionMetadata`, `AgentFlowStatusData`, `AgentSessionSummary`, `AgentBudgetSummary`, `AgentCompletedTaskSummary`, `AgentAvailableService`, `AgentAuditHistoryParams`, `AgentAuditHistoryData`, `AgentAuditEntry`, `AgentRevokeSessionRequest`, `AgentRevokeSessionData`, `AgentSessionTokenParam` を定義する。

- [ ] 1.5: `AgentResponse<T>` エンベロープ型と `AgentNextAction` 型を追加する
  設計書コンポーネント1の `AgentResponse<T>`（status, data, next_actions）と `AgentNextAction`（action, endpoint, method, description, params）を `src/types.rs` に追加する。

- [ ] 1.6: `AppConfig` に `agent_api_key` フィールドを追加する
  `src/types.rs` の `AppConfig` 構造体に `agent_api_key: Option<String>` を追加し、`AppConfig::from_env()` で `AGENT_API_KEY` 環境変数から読み込むようにする。

- [ ] 1.7: `Cargo.toml` に `jsonschema` 依存を追加する
  `jsonschema` crateを依存関係に追加する。タスク入力のJSON Schemaバリデーションに使用する。

---

## タスク2: エラー型拡張

エージェントAPI向けの構造化エラーレスポンス（`error_code`, `message`, `retry_allowed`, `next_actions`）をサポートするエラー型を追加する。

### 要件カバレッジ: 9.1, 9.2, 9.3, 9.4, 9.5

- [ ] 2.1: `AgentApiError` 型を `src/error.rs` に追加する
  `AgentApiError` 型を定義する。既存の `ApiError` をラップし、`next_actions: Vec<AgentNextAction>` と `retry_allowed: bool` を追加する。`IntoResponse` 実装で `AgentErrorResponse`（status="error", error_code, message, retry_allowed, next_actions）のJSON構造を返す。

- [ ] 2.2: `AgentApiError` に便利コンストラクタを追加する
  `scope_violation(reason, next_actions)` → 403、`budget_exceeded(next_actions)` → 403、`duplicate_request()` → 409、`upstream_timeout(retry_after)` → 504、`payment_pending(poll_endpoint)` → 202 の各コンストラクタを実装する。

- [ ] 2.3: エラー型のユニットテストを追加する
  各コンストラクタが正しいHTTPステータスと `AgentErrorResponse` JSON構造（error_code, retry_allowed, next_actions含む）を返すことをテストする。

---

## タスク3: エージェント認証ミドルウェアとセッション管理

APIキー認証ミドルウェアとスコープ付きセッション解決ユーティリティを実装する。

### 要件カバレッジ: 2.1, 2.2, 2.4, 2.5, 6.3

- [ ] 3.1: `src/agent.rs` モジュールを新設し、APIキー認証ミドルウェアを実装する
  `src/agent.rs` を作成し、`verify_agent_api_key` ミドルウェア関数を実装する。`Authorization: Bearer <key>` ヘッダーを `AppConfig.agent_api_key` と照合し、不一致時は `ApiError::unauthorized()` を返す。`src/main.rs` に `mod agent;` を追加する。

- [ ] 3.2: `resolve_agent_session()` ユーティリティを `src/agent.rs` に実装する
  セッショントークン（UUID）を受け取り、`agent_sessions` テーブルから有効期限内かつ `active = true` かつ `revoked_at IS NULL` のレコードを検索して `AgentSession` を返す関数を実装する。トークンが無効、期限切れ、または無効化済みの場合は401エラー（再認証指示付き `next_actions`）を返す。

- [ ] 3.3: スコープ検証ユーティリティを実装する
  `verify_task_scope(session: &AgentSession, task_type: &str)` と `verify_data_scope(session: &AgentSession, data_keys: &[String])` ユーティリティを `src/agent.rs` に実装する。スコープ外の場合は具体的な違反理由付きの403エラーを返す。

- [ ] 3.4: 認証ミドルウェアとセッション解決のユニットテストを追加する
  有効なAPIキー、無効なAPIキー、APIキー未設定、有効なセッショントークン、期限切れトークン、無効化済みトークン、スコープ内タスク、スコープ外タスクの各ケースをテストする。

---

## タスク4: セッション単位レート制限と安全装置

エージェントセッションごとのレート制限、重複リクエスト検知、異常パターン検知を実装する。

### 要件カバレッジ: 7.1, 7.2, 7.4, 7.5

- [ ] 4.1: `AgentRateLimitManager` を `src/agent.rs` に実装する
  `HashMap<Uuid, AgentSessionLimiter>` ベースのセッション単位レート制限マネージャーを実装する。既存の `RateLimiter` トークンバケットを再利用し、セッションあたり30 req/minの設定を適用する。超過時に429 + `Retry-After` ヘッダーを返す。

- [ ] 4.2: 重複リクエスト検知を実装する
  `AgentSessionLimiter` に `recent_requests: Vec<(String, Instant)>` を追加し、同一セッション・同一サービスへの5秒以内の重複リクエストを検知して409 Conflictを返すロジックを実装する。

- [ ] 4.3: 異常パターン検知とセッション一時停止を実装する
  スコープ外操作のカウントを `AgentSessionLimiter.scope_violations` で追跡し、5分間に3回以上のスコープ外操作で `agent_sessions.violation_count` をインクリメント、閾値（5回）超過で `active = false` に更新してセッションを一時停止する。

- [ ] 4.4: バックグラウンドクリーナータスクを実装する
  `tokio::spawn` + `tokio::time::interval`（5分間隔）で期限切れのレート制限エントリと古い重複検知レコードをクリーンアップするバックグラウンドタスクを `src/agent.rs` に実装する。

- [ ] 4.5: レート制限ミドルウェアを実装する
  `agent_rate_limit_middleware` 関数を実装し、リクエストからセッショントークンを抽出して `AgentRateLimitManager.check()` を呼び出す。セッション解決に失敗した場合はスルー（認証MWで捕捉されるため）する。

- [ ] 4.6: レート制限と安全装置のユニットテストを追加する
  トークン消費・補充、レート超過時の429、重複検知の409、スコープ外操作の累積とセッション停止の各ケースをテストする。

---

## タスク5: 監査ログミドルウェア

エージェント経由の全APIリクエストを自動的に監査ログテーブルに記録するミドルウェアを実装する。

### 要件カバレッジ: 8.1, 8.3, 8.4

- [ ] 5.1: 監査ログミドルウェアを `src/agent.rs` に実装する
  `agent_audit_middleware` 関数を実装する。リクエスト処理前にエンドポイントと操作種別を記録し、レスポンス取得後に `tokio::spawn` で非同期に `agent_audit_logs` テーブルにINSERTする。セッショントークンが解決できない場合（認証エラー等）でも操作を記録する（agent_session_id はNULLABLE対応）。

- [ ] 5.2: Prometheusメトリクスにエージェント用カウンターを追加する
  `src/types.rs` の `Metrics` 構造体に `agent_requests_total`（endpoint, status）、`agent_sessions_created_total`、`agent_sessions_revoked_total`、`agent_budget_spent_cents_total`、`agent_scope_violations_total`、`agent_rate_limit_hits_total` を追加し、レジストリに登録する。

- [ ] 5.3: 監査ミドルウェアとメトリクスのユニットテストを追加する
  リクエスト後にログがINSERTされること、セッション解決失敗時もログが記録されること、Prometheusカウンターが正しくインクリメントされることをテストする。

---

## タスク6: エージェントハンドラ実装（認証・サービス検索）

エージェントセッション作成とスコアリング付きサービス検索のハンドラを実装する。

### 要件カバレッジ: 1.2, 1.3, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4, 3.5, 6.1

- [ ] 6.1: `agent_auth` ハンドラを実装する
  `POST /agent/auth` ハンドラを `src/agent.rs` に実装する。`user_id` で `users` テーブルからユーザー存在を確認し、`AgentAuthRequest` のスコープ・予算上限を含む `agent_sessions` レコードを作成する。`AgentResponse<AgentAuthData>` を返し、`next_actions` に `search_services` と `flow_status` を含める。ユーザーが存在しない場合は404エラーを返す。

- [ ] 6.2: `agent_search_services` ハンドラを実装する
  `GET /agent/services` ハンドラを実装する。`resolve_agent_session()` でセッションを解決し、既存の campaigns + sponsored_apis 検索ロジックを呼び出す。セッションスコープの `allowed_categories` でフィルタリングし、`preferred_task_types` と `max_task_duration_secs` で追加フィルタを適用する。キーワードマッチによる `relevance_score`（名前完全一致=1.0、部分一致=0.5、スポンサー名一致=0.3）を計算し、スコア降順でソートする。`offset` / `limit`（デフォルト50、最大50）でページネーションを適用する。`next_actions` に `get_tasks` を含める。

- [ ] 6.3: `next_actions` ビルダーユーティリティを実装する
  フロー状態（unauthenticated, authenticated, service_selected, task_completed, service_ready）に応じた `AgentNextAction` リストを生成する `build_next_actions()` ユーティリティを `src/agent.rs` に実装する。各状態で利用可能なアクション（endpoint, method, description）を返す。

- [ ] 6.4: 認証・サービス検索ハンドラのユニットテストを追加する
  セッション作成（スコープ設定確認）、検索（キーワード、カテゴリ、嗜好フィルタ）、ページネーション、スコープフィルタリング、スコアリングの各ケースをテストする。

---

## タスク7: エージェントハンドラ実装（タスク・サービス実行）

タスク詳細取得、JSON Schemaバリデーション付きタスク完了、予算上限チェック付きサービス実行のハンドラを実装する。

### 要件カバレッジ: 1.3, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4, 5.5, 6.2, 6.4, 6.5, 7.3

- [ ] 7.1: `agent_get_tasks` ハンドラを実装する
  `GET /agent/tasks/{campaign_id}` ハンドラを実装する。`resolve_agent_session()` でセッションを解決し、キャンペーンの `task_schema` カラムからJSON Schema形式のタスク入力スキーマを取得して返す。`has_completed_task()` による完了状態を確認し、完了済みなら `next_actions` に `run_service`、未完了なら `complete_task` を含める。

- [ ] 7.2: `agent_complete_task` ハンドラを実装する
  `POST /agent/tasks/{campaign_id}/complete` ハンドラを実装する。以下のステップを順に実行する：(1) `resolve_agent_session()` でセッション解決、(2) `verify_task_scope()` でタスク種別のスコープ検証、(3) `verify_data_scope()` で提供データのスコープ検証、(4) `jsonschema` crateで `task_data` を `campaigns.task_schema` に対してバリデーション（不備時は400 + フィールドエラー詳細）、(5) `consents` テーブルに同意記録（`granted_via = "agent"`, `agent_session_id` 付き）、(6) `task_completions` テーブルにタスク完了記録（`execution_mode = "agent"`, `agent_session_id` 付き）。`next_actions` に `run_service` を含める。

- [ ] 7.3: `agent_run_service` ハンドラを実装する
  `POST /agent/services/{service}/run` ハンドラを実装する。以下のステップを順に実行する：(1) `resolve_agent_session()` でセッション解決、(2) 予算上限チェック（`budget_spent_cents + service_price <= budget_limit_cents`。超過時は決済せず追加承認要求エラーを返却）、(3) 既存の `run_proxy` コアロジック（キャンペーンマッチング→タスク完了確認→スポンサー決済）を呼び出し、(4) 決済成功時に `agent_sessions.budget_spent_cents` を更新、(5) `payments` に `agent_session_id` 付きで記録、(6) `AgentExecutionMetadata`（response_time_ms, cost_cents, budget_remaining_cents）を含む `AgentRunServiceData` を返す。`next_actions` に `search_services` と `audit_history` を含める。

- [ ] 7.4: タスク・サービス実行ハンドラのユニットテストを追加する
  タスク詳細取得（完了済み/未完了）、タスク完了（スコープ内/外、バリデーション成功/失敗、同意あり/なし）、サービス実行（スポンサー決済成功/予算不足/予算上限超過）、メタデータ記録の各ケースをテストする。

---

## タスク8: エージェントハンドラ実装（ステータス・管理）

フロー状態確認、監査履歴取得、セッション無効化のハンドラを実装する。

### 要件カバレッジ: 1.4, 6.3, 8.2, 8.5

- [ ] 8.1: `agent_flow_status` ハンドラを実装する
  `GET /agent/flow/status` ハンドラを実装する。`resolve_agent_session()` でセッションを解決し、セッション情報（agent_id, scopes, active, expires_at）、完了済みタスク一覧（task_completions + campaigns JOIN）、利用可能サービス一覧（キャンペーンマッチング + タスク完了状態）、予算サマリー（budget_limit, spent, remaining）を `AgentFlowStatusData` として返す。

- [ ] 8.2: `agent_audit_history` ハンドラを実装する
  `GET /agent/audit/history` ハンドラを実装する。`resolve_agent_session()` でセッションを解決し、`agent_audit_logs` テーブルからそのユーザーの全操作履歴を `created_at DESC` 順で取得する。`offset` / `limit` でページネーションを適用し、`AgentAuditHistoryData` として返す。

- [ ] 8.3: `agent_revoke_session` ハンドラを実装する
  `DELETE /agent/session` ハンドラを実装する。`agent_sessions` テーブルの対象レコードを `active = false`, `revoked_at = NOW()` に更新し、`AgentRevokeSessionData` を返す。

- [ ] 8.4: ステータス・管理ハンドラのユニットテストを追加する
  フロー状態確認（タスク完了前/後の利用可能サービス変化）、監査履歴取得（ページネーション）、セッション無効化（無効化後のリクエスト拒否確認）の各ケースをテストする。

---

## タスク9: ルーター統合と最終検証

エージェントサブルーターを既存Axumアプリに組み込み、環境変数を文書化し、E2Eフローの最終検証を行う。

### 要件カバレッジ: 1.1, 1.5, 9.2, 9.3, 9.4

- [ ] 9.1: `build_agent_router()` を作成し `build_app()` に組み込む
  `src/main.rs` に `build_agent_router()` 関数を作成する。設計書のアーキテクチャ図に従い、認証MW → セッション単位レート制限MW → 監査ログMW のミドルウェアスタックを適用し、8エンドポイントのルーティングを定義する。`build_app()` で `Router::nest("/agent", ...)` によりサブルーターを組み込む。既存の `/gpt/*` ルートおよび内部APIとの後方互換性を維持する。

- [ ] 9.2: `.env.example` に `AGENT_API_KEY` を追加する
  `.env.example` ファイルに `AGENT_API_KEY` 環境変数の説明とサンプル値を追加する。

- [ ] 9.3: コアロジックの共通化リファクタリングを行う
  `src/gpt.rs` の `gpt_search_services` と `gpt_run_service` から、サービス検索ロジックとスポンサー決済フローを `src/utils.rs` に共通関数（`search_services_core()`, `execute_sponsored_service()`）として抽出する。`/gpt/*` と `/agent/*` の両ハンドラからこれらの共通関数を呼び出すようにリファクタリングする。既存のGPTハンドラのテストが引き続きパスすることを確認する。

- [ ] 9.4: E2Eフローの統合テストを実装する
  テスト用DBを使用し、エージェント認証→サービス検索→タスク詳細取得→タスク完了（JSON Schemaバリデーション + 同意記録）→サービス実行（予算上限チェック + スポンサー決済）→フロー状態確認→監査履歴確認→セッション無効化の一連のフローを統合テストとして実装する。各ステップで `next_actions` の正確性、`execution_mode: "agent"` の記録、`agent_session_id` の記録を検証する。

- [ ] 9.5: 安全装置の統合テストを実装する
  レート制限超過（429 + Retry-After）、スコープ外操作拒否（403 + 違反理由）、予算上限超過（決済拒否 + 承認要求）、重複リクエスト拒否（409）、異常パターンによるセッション停止、セッション無効化後のリクエスト拒否の各シナリオをテストする。

- [ ]*9.6: 既存エンドポイントへの影響がないことを回帰テストで確認する
  `/gpt/*` ルートおよび内部API（`/campaigns`, `/tasks/complete`, `/proxy/{service}/run` 等）が変更なく動作することを確認する。

---

## 要件カバレッジマトリクス

| 要件ID | タスク |
|---|---|
| 1.1 | 9.1 |
| 1.2 | 1.5, 6.1 |
| 1.3 | 1.5, 6.3, 7.1 |
| 1.4 | 8.1 |
| 1.5 | 9.1 |
| 2.1 | 1.6, 3.1 |
| 2.2 | 1.1, 3.2, 6.1 |
| 2.3 | 1.1, 1.4, 3.3, 6.1 |
| 2.4 | 3.3 |
| 2.5 | 1.1, 3.2 |
| 3.1 | 6.2 |
| 3.2 | 6.2 |
| 3.3 | 6.2 |
| 3.4 | 6.2 |
| 3.5 | 6.2 |
| 4.1 | 7.1 |
| 4.2 | 1.7, 7.1 |
| 4.3 | 7.2 |
| 4.4 | 3.3, 7.2 |
| 4.5 | 7.1 |
| 4.6 | 1.3, 7.2 |
| 5.1 | 7.3 |
| 5.2 | 1.3, 7.3 |
| 5.3 | 7.3 |
| 5.4 | 7.3 |
| 5.5 | 7.3 |
| 6.1 | 1.4, 6.1 |
| 6.2 | 3.3, 7.2 |
| 6.3 | 8.3 |
| 6.4 | 7.2 |
| 6.5 | 1.3, 7.2 |
| 7.1 | 4.1 |
| 7.2 | 4.1 |
| 7.3 | 7.3 |
| 7.4 | 4.2 |
| 7.5 | 4.3 |
| 8.1 | 1.2, 5.1 |
| 8.2 | 8.2 |
| 8.3 | 5.2 |
| 8.4 | 5.1 |
| 8.5 | 8.2 |
| 9.1 | 2.1 |
| 9.2 | 2.2 |
| 9.3 | 2.2 |
| 9.4 | 2.1 |
| 9.5 | 2.1 |
