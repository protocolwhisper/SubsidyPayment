# Autonomous Agent Execution — 技術設計書

## 概要

本設計書は、SubsidyPaymentバックエンドに自律型エージェント対応APIを追加し、AIエージェント（Claude、GPT、カスタムボット等）がスコープ付きセッションを通じて、サービス発見→タスク実行→スポンサー決済→リソース取得のE2Eフローを人間の介入なしに自動化できるようにするための技術アーキテクチャを定義する。

### 設計方針

**ハイブリッドアプローチ**を採用する。キャンペーンマッチング・決済フロー・タスク完了等のコアビジネスロジックは `utils.rs` に共通化し、`/gpt/*` と `/agent/*` がそれぞれのアダプタ層から呼び出す。エージェント固有のセッション管理・監査・レスポンス形式は `src/agent.rs` として独立実装する。

### 要件トレーサビリティ

| 要件ID | 要件名 | 対応コンポーネント |
|---|---|---|
| 1.1–1.5 | エージェントAPIインターフェース | Agentルーター、レスポンスエンベロープ |
| 2.1–2.5 | エージェント認証・認可 | 認証ミドルウェア、`agent_sessions` テーブル |
| 3.1–3.5 | 自律的サービスディスカバリ | `agent_search_services` ハンドラ |
| 4.1–4.6 | 自律的タスク実行 | `agent_get_tasks`, `agent_complete_task` ハンドラ |
| 5.1–5.5 | 自律的支払い・リソース取得 | `agent_run_service` ハンドラ |
| 6.1–6.5 | 同意・ユーザー制御 | スコープ検証ミドルウェア、`consents` 拡張 |
| 7.1–7.5 | 安全装置・レート制限 | セッション単位レート制限、重複検知、異常検知 |
| 8.1–8.5 | 監査・オブザーバビリティ | `agent_audit_logs` テーブル、監査ミドルウェア |
| 9.1–9.5 | エラーハンドリング・リカバリ | `AgentErrorResponse` 構造 |

---

## アーキテクチャパターン & 境界マップ

### 採用パターン: エージェント専用サブルーター（ハイブリッド型）

既存のモノリシックAxumサーバーに `/agent/*` プレフィックスのサブルーターを追加する。GPTルーターと並列に配置し、エージェント専用のミドルウェアスタック（認証 + スコープ検証 + セッション単位レート制限 + 監査ログ）を適用する。

```
┌───────────────────────────────────────────────────────────┐
│           自律型エージェント (Claude / GPT / Bot)          │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  HTTP Client  (Authorization: Bearer <AGENT_API_KEY>)│  │
│  └──────────────────────┬──────────────────────────────┘  │
└─────────────────────────┼─────────────────────────────────┘
                          │ HTTPS
                          ▼
┌───────────────────────────────────────────────────────────┐
│                  Axum Server (Render)                      │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  /agent/* サブルーター                               │ │
│  │  ┌────────────────────────────────────────────────┐  │ │
│  │  │  L1: 認証ミドルウェア (APIキー検証)            │  │ │
│  │  │  L2: セッション単位レート制限 (30 req/min)     │  │ │
│  │  │  L3: 監査ログミドルウェア                       │  │ │
│  │  └────────────────────────────────────────────────┘  │ │
│  │                                                        │ │
│  │  POST /agent/auth               → セッション作成      │ │
│  │  GET  /agent/services            → サービス検索        │ │
│  │  GET  /agent/tasks/:id           → タスク詳細取得      │ │
│  │  POST /agent/tasks/:id/complete  → タスク完了          │ │
│  │  POST /agent/services/:svc/run   → サービス実行        │ │
│  │  GET  /agent/flow/status         → フロー状態確認      │ │
│  │  GET  /agent/audit/history       → 操作履歴取得        │ │
│  │  DELETE /agent/session            → セッション無効化   │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  /gpt/* サブルーター (既存、変更なし)                │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  既存ルート (変更なし)                                │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────┐    │
│  │ agent.rs   │  │ types.rs   │  │ utils.rs         │    │
│  │ (エージェント│  │ (共有型 +  │  │ (共有コアロジック)│    │
│  │  ハンドラ)  │  │  Agent型)  │  │                  │    │
│  └────────────┘  └────────────┘  └──────────────────┘    │
└────────────────────────┬──────────────────────────────────┘
                         │
                         ▼
               ┌──────────────────┐
               │   PostgreSQL     │
               │  ┌────────────┐  │
               │  │ users      │  │
               │  │ campaigns  │  │
               │  │ task_comp. │  │  ← execution_mode, agent_session_id 追加
               │  │ payments   │  │  ← agent_session_id 追加
               │  │ consents   │  │  ← granted_via, agent_session_id 追加
               │  │ agent_     │  │  ← 新規テーブル
               │  │  sessions  │  │
               │  │ agent_     │  │  ← 新規テーブル
               │  │  audit_logs│  │
               │  └────────────┘  │
               └──────────────────┘
```

### 境界定義

| 境界 | 内側 | 外側 | インターフェース |
|---|---|---|---|
| Agentサブルーター | Agent専用ハンドラ、認証、レート制限、監査 | 既存ルート、GPTルート | `Router::nest("/agent", ...)` |
| 認証境界 | エージェントAPIキー認証済みリクエスト | 未認証リクエスト | `verify_agent_api_key` ミドルウェア |
| スコープ境界 | セッションスコープ内の操作 | スコープ外操作 | スコープ検証（各ハンドラ内） |
| データ境界 | 同意済みデータのみスポンサーに転送 | 未同意データ | `consents` テーブル + セッションスコープ |
| コアロジック境界 | `utils.rs` の共有関数 | `/gpt/*`、`/agent/*` アダプタ | 関数インターフェース |

---

## 技術スタック & アラインメント

### 既存スタックとの整合性

| 項目 | 既存 | Agent追加分 | 整合性 |
|---|---|---|---|
| Web フレームワーク | Axum 0.8 | Axum 0.8（サブルーター追加） | ✅ 完全整合 |
| 認証 | `verify_gpt_api_key` | 同パターンで `verify_agent_api_key` | ✅ 完全整合 |
| レート制限 | グローバル `RateLimiter` | セッション単位 `HashMap<Uuid, RateLimiter>` | ✅ パターン拡張 |
| DB | PostgreSQL + SQLx 0.8 | 同一DB、マイグレーション追加 | ✅ 完全整合 |
| シリアライズ | Serde | 同一 | ✅ 完全整合 |
| エラー処理 | `ApiError` / `thiserror` | 既存パターン拡張 | ✅ 完全整合 |
| メトリクス | Prometheus 0.14 | ラベル追加（`agent_*`） | ✅ 完全整合 |
| ログ | tracing | 同一（構造化ログ） | ✅ 完全整合 |

### 新規依存関係

| crate | バージョン | 用途 | 要件参照 |
|---|---|---|---|
| `jsonschema` | 0.28+ | タスク入力のJSON Schemaバリデーション | 4.2, 4.3 |

---

## コンポーネント & インターフェース契約

### コンポーネント1: エージェントレスポンスエンベロープ

**対応要件**: 1.2, 1.3, 9.1

**責務**: 全エージェントAPIレスポンスを統一構造でラップし、`next_actions` による自律ナビゲーションを提供する。

**型定義**:

```rust
/// エージェントAPI成功レスポンスのエンベロープ
#[derive(Debug, Serialize)]
pub struct AgentResponse<T: Serialize> {
    pub status: &'static str,       // "success"
    pub data: T,
    pub next_actions: Vec<AgentNextAction>,
}

/// エージェントAPIエラーレスポンス
#[derive(Debug, Serialize)]
pub struct AgentErrorResponse {
    pub status: &'static str,       // "error"
    pub error_code: String,         // 機械判読用コード
    pub message: String,            // 人間用メッセージ
    pub retry_allowed: bool,        // 再試行可否
    pub next_actions: Vec<AgentNextAction>,  // 代替手段
}

/// 次アクションのヒント
#[derive(Debug, Serialize)]
pub struct AgentNextAction {
    pub action: String,             // "search_services", "get_tasks", "complete_task", etc.
    pub endpoint: String,           // "/agent/tasks/{campaign_id}"
    pub method: String,             // "GET" | "POST" | "DELETE"
    pub description: String,        // アクションの説明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,  // 推奨パラメータ
}
```

**`next_actions` 状態マシン**:

```
unauthenticated → [auth]
authenticated   → [search_services, flow_status]
service_selected → [get_tasks, search_services]
task_completed  → [run_service, flow_status]
service_ready   → [run_service, search_services, audit_history]
```

---

### コンポーネント2: エージェントセッション管理

**対応要件**: 2.1, 2.2, 2.3, 2.5, 6.1

**責務**: スコープ・予算上限付きのエージェントセッションを作成・管理する。

#### DBスキーマ

```sql
-- マイグレーション: 0011_agent_sessions.sql
CREATE TABLE IF NOT EXISTS agent_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    scopes JSONB NOT NULL,
    budget_limit_cents BIGINT NOT NULL DEFAULT 0,
    budget_spent_cents BIGINT NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    violation_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS agent_sessions_token_idx ON agent_sessions(token);
CREATE INDEX IF NOT EXISTS agent_sessions_user_id_idx ON agent_sessions(user_id);
CREATE INDEX IF NOT EXISTS agent_sessions_expires_at_idx ON agent_sessions(expires_at);
```

#### スコープ型定義

```rust
/// エージェントセッションのスコープ（ユーザーが設定）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionScopes {
    /// 許可するタスク種別（例: ["survey", "data_provision"]）
    pub allowed_task_types: Vec<String>,
    /// 許可するデータ提供種別（例: ["email", "region"]）
    pub allowed_data_types: Vec<String>,
    /// 許可するサービスカテゴリ（例: ["scraping", "design"]）
    pub allowed_categories: Vec<String>,
}

/// エージェントセッション（DB行）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentSession {
    pub id: Uuid,
    pub token: Uuid,
    pub user_id: Uuid,
    pub agent_id: String,
    #[sqlx(json)]
    pub scopes: AgentSessionScopes,
    pub budget_limit_cents: i64,
    pub budget_spent_cents: i64,
    pub active: bool,
    pub violation_count: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

#### 認証エンドポイント

```rust
/// POST /agent/auth
/// エージェントセッションを作成
async fn agent_auth(
    State(state): State<SharedState>,
    Json(payload): Json<AgentAuthRequest>,
) -> Result<Json<AgentResponse<AgentAuthData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentAuthRequest {
    pub user_id: Uuid,              // 既存ユーザーID
    pub agent_id: String,           // エージェント識別子
    pub scopes: AgentSessionScopes, // 操作スコープ
    pub budget_limit_cents: u64,    // 予算上限
}

#[derive(Debug, Serialize)]
pub struct AgentAuthData {
    pub session_token: Uuid,
    pub user_id: Uuid,
    pub agent_id: String,
    pub scopes: AgentSessionScopes,
    pub budget_limit_cents: u64,
    pub expires_at: DateTime<Utc>,
}
```

#### セッション解決ユーティリティ

```rust
/// セッショントークンからエージェントセッション全体を解決
/// 無効・期限切れ・無効化済みの場合は401を返す
async fn resolve_agent_session(
    db: &PgPool,
    session_token: Uuid,
) -> ApiResult<AgentSession>
// SELECT * FROM agent_sessions
// WHERE token = $1 AND active = true AND expires_at > NOW() AND revoked_at IS NULL
```

---

### コンポーネント3: 認証ミドルウェア

**対応要件**: 2.1, 2.4

**責務**: `/agent/*` ルートへのリクエストに対し、APIキーを検証する。

```rust
/// エージェントAPIキー認証ミドルウェア
/// 環境変数 AGENT_API_KEY と照合
async fn verify_agent_api_key(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError>
```

**設定**: 環境変数 `AGENT_API_KEY` を `AppConfig` に追加。

---

### コンポーネント4: セッション単位レート制限

**対応要件**: 7.1, 7.2, 7.4, 7.5

**責務**: エージェントセッションごとにリクエストレートを制限し、重複・異常を検知する。

```rust
/// セッション単位のレート制限マネージャー
pub struct AgentRateLimitManager {
    /// セッションID → レート制限状態
    limiters: Arc<RwLock<HashMap<Uuid, AgentSessionLimiter>>>,
}

struct AgentSessionLimiter {
    rate_limiter: RateLimiter,           // 既存のトークンバケット（30 req/min）
    recent_requests: Vec<(String, Instant)>, // (service, timestamp) 重複検知用
    scope_violations: u32,               // スコープ外操作のカウント
    last_cleanup: Instant,
}

impl AgentRateLimitManager {
    /// セッション単位でレート制限を確認
    pub async fn check(&self, session_id: Uuid, service: Option<&str>) -> Result<(), ApiError>;

    /// バックグラウンドで期限切れエントリを削除
    pub async fn cleanup_expired(&self);
}
```

**重複検知ロジック**: 同一セッション・同一サービスへのリクエストが5秒以内に再発した場合、409 Conflictを返却。

**異常パターン検知**: 5分間にスコープ外操作が3回以上 → `agent_sessions.violation_count` をインクリメント。閾値（5回）超過で `active = false` に更新しセッションを一時停止。

---

### コンポーネント5: 監査ログミドルウェア

**対応要件**: 8.1, 8.3, 8.4

**責務**: エージェント経由の全APIリクエストを自動的に監査ログに記録する。

#### DBスキーマ

```sql
-- マイグレーション: 0012_agent_audit_logs.sql
CREATE TABLE IF NOT EXISTS agent_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_session_id UUID NOT NULL REFERENCES agent_sessions(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    operation TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    request_summary JSONB,
    response_status SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS agent_audit_logs_session_idx
    ON agent_audit_logs(agent_session_id, created_at DESC);
CREATE INDEX IF NOT EXISTS agent_audit_logs_user_idx
    ON agent_audit_logs(user_id, created_at DESC);
```

#### ミドルウェア設計

```rust
/// 監査ログミドルウェア
/// リクエスト処理後にログを非同期で挿入
async fn agent_audit_middleware(
    State(state): State<SharedState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response
```

**動作**:
1. リクエストからセッショントークンを抽出（ヘッダーまたはボディ）
2. `next.run(request).await` で後続ハンドラを実行
3. レスポンス取得後、非同期で `agent_audit_logs` にINSERT（`tokio::spawn`）
4. メトリクス更新（`agent_requests_total` カウンター）

---

### コンポーネント6: サービス検索ハンドラ

**対応要件**: 3.1, 3.2, 3.3, 3.4, 3.5

**責務**: スコアリング付きサービス検索、嗜好フィルタリング、ページネーションを提供する。

```rust
/// GET /agent/services?q=<keyword>&category=<cat>&offset=0&limit=50&session_token=<uuid>
async fn agent_search_services(
    State(state): State<SharedState>,
    Query(params): Query<AgentSearchParams>,
) -> Result<Json<AgentResponse<AgentSearchData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentSearchParams {
    pub session_token: Uuid,
    pub q: Option<String>,
    pub category: Option<String>,
    /// 嗜好フィルタ: 許容タスク種別（カンマ区切り）
    pub preferred_task_types: Option<String>,
    /// 嗜好フィルタ: 最大所要時間（秒）
    pub max_task_duration_secs: Option<u64>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,       // デフォルト50、最大50
}

#[derive(Debug, Serialize)]
pub struct AgentSearchData {
    pub services: Vec<AgentServiceItem>,
    pub total_count: usize,
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct AgentServiceItem {
    pub service_type: String,
    pub service_id: Uuid,
    pub name: String,
    pub sponsor: String,
    pub required_task: Option<String>,
    pub task_type: Option<String>,
    pub subsidy_amount_cents: u64,
    pub category: Vec<String>,
    pub estimated_task_duration_secs: Option<u64>,
    pub relevance_score: f64,      // 0.0–1.0
    pub active: bool,
}
```

**ロジック**:
1. 既存の `gpt_search_services` 検索ロジック（`utils.rs` に抽出）を呼び出し
2. セッションスコープの `allowed_categories` でフィルタリング
3. `preferred_task_types`、`max_task_duration_secs` で追加フィルタ
4. キーワードマッチによる `relevance_score` を計算（名前完全一致=1.0、部分一致=0.5、スポンサー名一致=0.3）
5. `offset` / `limit` でページネーション
6. `next_actions` に `get_tasks` アクションを含める

---

### コンポーネント7: タスク詳細取得ハンドラ

**対応要件**: 4.1, 4.2, 4.5

**責務**: JSON Schema形式のタスク入力スキーマを返す。

```rust
/// GET /agent/tasks/{campaign_id}?session_token=<uuid>
async fn agent_get_tasks(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Query(params): Query<AgentSessionTokenParam>,
) -> Result<Json<AgentResponse<AgentTaskData>>, AgentApiError>

#[derive(Debug, Serialize)]
pub struct AgentTaskData {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub sponsor: String,
    pub required_task: String,
    pub task_description: String,
    /// JSON Schema形式のタスク入力スキーマ
    pub task_input_schema: serde_json::Value,
    pub already_completed: bool,
    pub subsidy_amount_cents: u64,
}
```

**タスク入力スキーマ例**:
```json
{
  "type": "object",
  "required": ["email", "region"],
  "properties": {
    "email": { "type": "string", "format": "email" },
    "region": { "type": "string", "enum": ["JP", "US", "EU"] },
    "feedback": { "type": "string", "maxLength": 500 }
  }
}
```

**ロジック**:
1. `campaigns.task_schema` から JSON Schema を取得（NULL の場合はデフォルトスキーマ）
2. `has_completed_task()` でユーザーの完了状態を確認
3. 完了済みの場合、`next_actions` に `run_service` を含める
4. 未完了の場合、`next_actions` に `complete_task` を含める

---

### コンポーネント8: タスク完了ハンドラ

**対応要件**: 4.3, 4.4, 4.5, 4.6, 6.2, 6.4, 6.5

**責務**: タスク入力をJSON Schemaでバリデーションし、スコープ検証後にタスク完了を記録する。

```rust
/// POST /agent/tasks/{campaign_id}/complete
async fn agent_complete_task(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<AgentCompleteTaskRequest>,
) -> Result<Json<AgentResponse<AgentCompleteTaskData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentCompleteTaskRequest {
    pub session_token: Uuid,
    pub task_name: String,
    pub task_data: serde_json::Value,   // JSON Schema準拠の入力データ
    pub consent: AgentConsentInput,
}

#[derive(Debug, Deserialize)]
pub struct AgentConsentInput {
    pub data_sharing_agreed: bool,
    pub purpose_acknowledged: bool,
    pub contact_permission: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentCompleteTaskData {
    pub task_completion_id: Uuid,
    pub campaign_id: Uuid,
    pub consent_recorded: bool,
    pub can_use_service: bool,
}
```

**ロジック**:
1. `resolve_agent_session()` でセッションを解決
2. **スコープ検証**: タスク種別がセッションの `allowed_task_types` に含まれているか確認（要件 4.4）
3. **データ種別スコープ検証**: 提供データのキーがセッションの `allowed_data_types` に含まれているか確認（要件 6.2）
4. **JSON Schemaバリデーション**: `task_data` を `campaigns.task_schema` に対してバリデーション（`jsonschema` crate）
5. バリデーション不備 → 400 + フィールドエラー詳細
6. 同意記録（`consents` テーブル。`granted_via = "agent"`, `agent_session_id` 付き）
7. タスク完了記録（`task_completions` テーブル。`execution_mode = "agent"`, `agent_session_id` 付き）
8. `next_actions` に `run_service` を含める

---

### コンポーネント9: サービス実行ハンドラ

**対応要件**: 5.1, 5.2, 5.3, 5.4, 5.5, 7.3

**責務**: スポンサー決済を実行し、構造化レスポンスでリソースを返却する。予算上限チェックを含む。

```rust
/// POST /agent/services/{service}/run
async fn agent_run_service(
    State(state): State<SharedState>,
    Path(service): Path<String>,
    Json(payload): Json<AgentRunServiceRequest>,
) -> Result<Json<AgentResponse<AgentRunServiceData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentRunServiceRequest {
    pub session_token: Uuid,
    pub input: String,
}

#[derive(Debug, Serialize)]
pub struct AgentRunServiceData {
    pub service: String,
    pub output: String,
    pub payment_mode: String,
    pub sponsored_by: Option<String>,
    pub tx_hash: Option<String>,
    /// 実行メタデータ
    pub execution_metadata: AgentExecutionMetadata,
}

#[derive(Debug, Serialize)]
pub struct AgentExecutionMetadata {
    pub response_time_ms: u64,
    pub cost_cents: u64,
    pub budget_remaining_cents: u64,
}
```

**ロジック**:
1. `resolve_agent_session()` でセッションを解決
2. **予算上限チェック**: `budget_spent_cents + service_price <= budget_limit_cents`（要件 5.5）
3. 超過する場合 → 決済を実行せず、`next_actions` に予算追加承認リクエストを含めたエラーを返却
4. 既存の `run_proxy` コアロジック（`utils.rs` に抽出）を呼び出し
5. 決済成功時 → `budget_spent_cents` を更新、`payments` に `agent_session_id` 付きで記録
6. レスポンスに `execution_metadata`（レスポンスタイム、コスト、残予算）を含める
7. `next_actions` に `search_services`（次のサービスを探す）と `audit_history` を含める

---

### コンポーネント10: フローステータスハンドラ

**対応要件**: 1.4

**責務**: エージェントのフロー全体の状態を1回の呼び出しで返す。

```rust
/// GET /agent/flow/status?session_token=<uuid>
async fn agent_flow_status(
    State(state): State<SharedState>,
    Query(params): Query<AgentSessionTokenParam>,
) -> Result<Json<AgentResponse<AgentFlowStatusData>>, AgentApiError>

#[derive(Debug, Serialize)]
pub struct AgentFlowStatusData {
    pub session: AgentSessionSummary,
    pub completed_tasks: Vec<AgentCompletedTaskSummary>,
    pub available_services: Vec<AgentAvailableService>,
    pub budget_summary: AgentBudgetSummary,
}

#[derive(Debug, Serialize)]
pub struct AgentSessionSummary {
    pub agent_id: String,
    pub scopes: AgentSessionScopes,
    pub active: bool,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AgentBudgetSummary {
    pub budget_limit_cents: u64,
    pub budget_spent_cents: u64,
    pub budget_remaining_cents: u64,
}

#[derive(Debug, Serialize)]
pub struct AgentCompletedTaskSummary {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub task_name: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AgentAvailableService {
    pub service: String,
    pub sponsor: String,
    pub ready: bool,
    pub cost_cents: u64,
}
```

---

### コンポーネント11: 監査履歴ハンドラ

**対応要件**: 8.2, 8.5

**責務**: ユーザーのエージェント操作履歴を取得する。

```rust
/// GET /agent/audit/history?session_token=<uuid>&offset=0&limit=50
async fn agent_audit_history(
    State(state): State<SharedState>,
    Query(params): Query<AgentAuditHistoryParams>,
) -> Result<Json<AgentResponse<AgentAuditHistoryData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentAuditHistoryParams {
    pub session_token: Uuid,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct AgentAuditHistoryData {
    pub entries: Vec<AgentAuditEntry>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AgentAuditEntry {
    pub id: Uuid,
    pub operation: String,
    pub endpoint: String,
    pub response_status: u16,
    pub created_at: DateTime<Utc>,
}
```

---

### コンポーネント12: セッション無効化ハンドラ

**対応要件**: 6.3

**責務**: エージェントセッションを即時無効化する。

```rust
/// DELETE /agent/session
async fn agent_revoke_session(
    State(state): State<SharedState>,
    Json(payload): Json<AgentRevokeSessionRequest>,
) -> Result<Json<AgentResponse<AgentRevokeSessionData>>, AgentApiError>

#[derive(Debug, Deserialize)]
pub struct AgentRevokeSessionRequest {
    pub session_token: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AgentRevokeSessionData {
    pub revoked: bool,
    pub revoked_at: DateTime<Utc>,
}
```

**ロジック**: `UPDATE agent_sessions SET active = false, revoked_at = NOW() WHERE token = $1`

---

## 既存テーブルへのスキーマ変更

### マイグレーション: 0013_agent_metadata.sql

```sql
-- task_completions にエージェントメタデータ追加
ALTER TABLE task_completions
    ADD COLUMN IF NOT EXISTS execution_mode TEXT DEFAULT 'human',
    ADD COLUMN IF NOT EXISTS agent_session_id UUID REFERENCES agent_sessions(id) ON DELETE SET NULL;

-- payments にエージェントセッションID追加
ALTER TABLE payments
    ADD COLUMN IF NOT EXISTS agent_session_id UUID REFERENCES agent_sessions(id) ON DELETE SET NULL;

-- consents にエージェントメタデータ追加
ALTER TABLE consents
    ADD COLUMN IF NOT EXISTS granted_via TEXT DEFAULT 'human',
    ADD COLUMN IF NOT EXISTS agent_session_id UUID REFERENCES agent_sessions(id) ON DELETE SET NULL;
```

---

## モジュール構成

### 新規ファイル

| ファイル | 責務 |
|---|---|
| `src/agent.rs` | エージェント専用ハンドラ群（8ハンドラ + 認証MW + レート制限 + 監査MW） |
| `migrations/0011_agent_sessions.sql` | `agent_sessions` テーブル作成 |
| `migrations/0012_agent_audit_logs.sql` | `agent_audit_logs` テーブル作成 |
| `migrations/0013_agent_metadata.sql` | 既存テーブルへのメタデータカラム追加 |

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `src/main.rs` | `mod agent;` 追加、`build_agent_router()` 組み込み |
| `src/types.rs` | エージェント用リクエスト/レスポンス型追加 |
| `src/error.rs` | `AgentApiError` 型（`next_actions` 付きエラー）追加 |
| `src/utils.rs` | GPTハンドラからコアロジックを抽出（`search_services_core()`, `run_service_core()` 等） |
| `Cargo.toml` | `jsonschema` 依存追加 |
| `.env.example` | `AGENT_API_KEY` 追加 |

---

## データフロー

### E2Eフロー: エージェントによる自律実行

```
エージェント → [POST /agent/auth] → agent_sessions に INSERT
                                       ↓ session_token
エージェント → [GET /agent/services?session_token=xxx&q=scraping]
                ↓ スコープフィルタ + スコアリング
              サービス一覧 + next_actions: [get_tasks]
                                       ↓
エージェント → [GET /agent/tasks/{campaign_id}?session_token=xxx]
                ↓ JSON Schema返却
              タスク詳細 + input_schema + next_actions: [complete_task]
                                       ↓
エージェント → [POST /agent/tasks/{campaign_id}/complete]
                ↓ スコープ検証 → JSON Schema バリデーション
                ↓ 同意記録 (granted_via: "agent")
                ↓ タスク完了記録 (execution_mode: "agent")
              完了確認 + next_actions: [run_service]
                                       ↓
エージェント → [POST /agent/services/{service}/run]
                ↓ 予算上限チェック → キャンペーンマッチ
                ↓ スポンサー決済 → budget_spent_cents 更新
                ↓ payments に agent_session_id 付き記録
              リソース + execution_metadata + next_actions: [search_services]

※ 全リクエストで agent_audit_logs に自動記録
```

---

## Prometheusメトリクス追加

```rust
// 新規メトリクス
agent_requests_total{endpoint, status}      // エージェントAPIリクエスト数
agent_sessions_created_total                 // エージェントセッション作成数
agent_sessions_revoked_total                 // エージェントセッション無効化数
agent_budget_spent_cents_total               // エージェント経由の累計決済額
agent_scope_violations_total                 // スコープ外操作の試行数
agent_rate_limit_hits_total                  // レート制限到達数
```

---

## セキュリティ考慮事項

| 項目 | 対策 | 要件 |
|---|---|---|
| APIキー管理 | 環境変数 `AGENT_API_KEY` で管理 | 2.1 |
| セッションスコープ | ユーザーが設定したスコープを全操作で検証 | 2.3, 2.4 |
| 予算上限 | セッション単位の累計決済額を追跡・制限 | 5.5, 7.3 |
| セッション有効期限 | 24時間、期限切れで401 | 2.5 |
| レート制限 | セッション単位で30 req/min | 7.1, 7.2 |
| 重複検知 | 5秒以内の同一サービスリクエストを拒否 | 7.4 |
| 異常検知 | スコープ外操作の連続試行でセッション停止 | 7.5 |
| 同意検証 | データ提供前にスコープ内の同意を確認 | 6.2, 6.4 |
| 監査ログ | 全操作を自動記録 | 8.1 |
| 内部情報保護 | 500エラーで内部詳細を漏洩しない | 9.5 |

---

## テスト戦略

| テストレベル | 対象 | 方法 |
|---|---|---|
| ユニットテスト | スコープ検証ロジック、`next_actions` 生成、JSON Schemaバリデーション | `cargo test` |
| 統合テスト | エージェントE2Eフロー（認証→検索→タスク→決済→履歴） | テスト用DBでのHTTPリクエストテスト |
| セキュリティテスト | スコープ外操作拒否、予算超過拒否、レート制限、セッション無効化 | `cargo test` |
| メトリクステスト | Prometheusカウンター更新 | `cargo test` |

---

## 後方互換性

- 既存の全APIエンドポイント（`/gpt/*` 含む）は変更なし
- 新規エンドポイントは `/agent/*` プレフィックスで完全に分離
- DBマイグレーションは `ADD COLUMN IF NOT EXISTS` / `CREATE TABLE IF NOT EXISTS` で安全
- 既存テーブルへの追加カラムは全て `NULLABLE` または `DEFAULT` 付きで既存レコードに影響なし
- `utils.rs` へのコアロジック抽出は内部リファクタリングであり、外部APIに影響なし
