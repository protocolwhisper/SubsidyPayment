# GPT Apps Integration — 技術設計書

## 概要

本設計書は、SnapFuelバックエンドにGPT Apps（Custom GPTs + Custom Actions）対応を追加し、エンドユーザー（ToC）がChatGPT上でサービス選択→タスク実行→スポンサー決済→リソース取得のE2Eフローを完結できるようにするための技術アーキテクチャを定義する。

### 要件トレーサビリティ

| 要件ID | 要件名 | 対応コンポーネント |
|---|---|---|
| 1.1–1.5 | GPT Actions API エンドポイント | OpenAPIスキーマ、GPTルーター、CORS |
| 2.1–2.4 | サービスディスカバリ | `gpt_search_services` ハンドラ |
| 3.1–3.4 | ユーザー登録・識別 | `gpt_auth` ハンドラ、認証ミドルウェア |
| 4.1–4.6 | タスク実行フロー | `gpt_get_tasks`, `gpt_complete_task` ハンドラ |
| 5.1–5.5 | 支払い・リソースアクセス | `gpt_run_service` ハンドラ |
| 6.1–6.4 | 同意・コンプライアンス | `consents` テーブル、同意ガード、プライバシーページ |
| 7.1–7.5 | GPT構成 | システムプロンプト、Conversation Starters |
| 8.1–8.5 | エラーハンドリング | 認証ミドルウェア、レート制限、既存ApiError |
| 9.1–9.4 | デプロイ・運用 | 環境変数、メトリクス、OpenAPIスキーマ |

---

## アーキテクチャパターン & 境界マップ

### 採用パターン: GPT専用サブルーター

既存のモノリシックAxumサーバーに `/gpt/*` プレフィックスのサブルーターを追加する。GPT専用ルートには認証・レート制限ミドルウェアを適用し、既存APIとの関心を分離する。

```
┌─────────────────────────────────────────────────────┐
│                   ChatGPT UI                         │
│  ┌───────────────────────────────────────────────┐   │
│  │           Custom GPT (GPT Apps)               │   │
│  │  システムプロンプト + Conversation Starters    │   │
│  └───────────────┬───────────────────────────────┘   │
│                  │ Custom Actions (OpenAPI 3.1.0)     │
│                  │ Authorization: Bearer <API_KEY>    │
└──────────────────┼──────────────────────────────────┘
                   │ HTTPS
                   ▼
┌─────────────────────────────────────────────────────┐
│              Axum Server (Render)                     │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  /.well-known/openapi.yaml  (静的配信)          │ │
│  │  /privacy                   (静的配信)          │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  /gpt/* サブルーター                            │ │
│  │  ┌───────────────────────────────────────────┐  │ │
│  │  │  認証ミドルウェア (APIキー検証)            │  │ │
│  │  │  レート制限ミドルウェア (60 req/min)       │  │ │
│  │  └───────────────────────────────────────────┘  │ │
│  │                                                   │ │
│  │  GET  /gpt/services       → サービス検索         │ │
│  │  POST /gpt/auth           → ユーザー登録/識別    │ │
│  │  GET  /gpt/tasks/:id      → タスク詳細取得       │ │
│  │  POST /gpt/tasks/:id/complete → タスク完了       │ │
│  │  POST /gpt/services/:service/run → サービス実行  │ │
│  │  GET  /gpt/user/status    → ユーザー状態確認    │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  既存ルート (認証なし、変更なし)                 │ │
│  │  /health, /campaigns, /tasks/complete,           │ │
│  │  /proxy/:service/run, /sponsored-apis, etc.      │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ types.rs │  │ error.rs │  │ utils.rs         │   │
│  │ (共有型) │  │ (共有)   │  │ (共有ユーティリティ)│   │
│  └──────────┘  └──────────┘  └──────────────────┘   │
└───────────────────────┬─────────────────────────────┘
                        │
                        ▼
              ┌──────────────────┐
              │   PostgreSQL     │
              │  ┌────────────┐  │
              │  │ users      │  │  ← source カラム追加
              │  │ campaigns  │  │  ← task_schema カラム追加
              │  │ task_comp. │  │
              │  │ payments   │  │
              │  │ consents   │  │  ← 新規テーブル
              │  │ ...        │  │
              │  └────────────┘  │
              └──────────────────┘
```

### 境界定義

| 境界 | 内側 | 外側 | インターフェース |
|---|---|---|---|
| GPTサブルーター | GPT専用ハンドラ、認証、レート制限 | 既存ルート | `Router::nest("/gpt", ...)` |
| 認証境界 | GPTルート全体 | 公開ルート（health, well-known等） | `verify_gpt_api_key` ミドルウェア |
| データ境界 | 同意済みデータのみスポンサーに転送 | 未同意データ | `consents` テーブル参照 |

---

## 技術スタック & アラインメント

### 既存スタックとの整合性

| 項目 | 既存 | GPT Apps追加分 | 整合性 |
|---|---|---|---|
| Web フレームワーク | Axum 0.8 | Axum 0.8（サブルーター追加） | ✅ 完全整合 |
| 認証 | なし | `axum::middleware::from_fn` | ✅ Axumネイティブ |
| レート制限 | なし | `tower::limit::RateLimitLayer` | ✅ towerエコシステム |
| DB | PostgreSQL + SQLx 0.8 | 同一DB、マイグレーション追加 | ✅ 完全整合 |
| シリアライズ | Serde | 同一 | ✅ 完全整合 |
| エラー処理 | `ApiError` / `thiserror` | 既存パターン拡張 | ✅ 完全整合 |
| メトリクス | Prometheus 0.14 | ラベル追加のみ | ✅ 完全整合 |

### 新規依存関係

追加のcrateは不要。既存の `tower`, `tower-http`, `axum` で全て実装可能。

---

## コンポーネント & インターフェース契約

### コンポーネント1: GPT認証ミドルウェア

**対応要件**: 3.2, 8.3

**責務**: `/gpt/*` ルートへのリクエストに対し、`Authorization: Bearer <api_key>` ヘッダーを検証する。

**インターフェース**:

```rust
/// GPT Actions APIキー認証ミドルウェア
/// 環境変数 GPT_ACTIONS_API_KEY と照合
async fn verify_gpt_api_key(
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError>
```

**動作**:
- `Authorization` ヘッダーが存在しない → 401 `{ error: { code: "unauthorized", message: "API key required" } }`
- `Bearer` プレフィックスが不正 → 401
- APIキーが不一致 → 403 `{ error: { code: "forbidden", message: "Invalid API key" } }`
- 検証成功 → `next.run(request).await` で後続ハンドラに委譲

**設定**:
- 環境変数: `GPT_ACTIONS_API_KEY` — GPT Builder UIに設定するAPIキー
- `AppConfig` に `gpt_actions_api_key: Option<String>` フィールドを追加

---

### コンポーネント1.5: セッショントークン管理

**対応要件**: 3.2, 3.3, 8.3

**責務**: GPT経由のユーザーをセッショントークンで安全に識別する。APIキー認証はGPT全体で共有されるため、ユーザー個別の識別にはセッショントークンを使用する。

**背景（セキュリティ上の動機）**:
- GPT Actions のAPIキーは全ChatGPTユーザーで共有される
- `user_id`（UUID）をリクエストボディで直接渡す方式では、他ユーザーのIDを推測・入力することで、なりすましやデータ閲覧が可能になる
- セッショントークンを `/gpt/auth` で発行し、以降のリクエストではトークンからサーバー側で `user_id` を解決することで、ユーザー識別の安全性を確保する

#### DBスキーマ

```sql
-- マイグレーション: 0009_gpt_sessions.sql
CREATE TABLE IF NOT EXISTS gpt_sessions (
    token UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 days'
);

CREATE INDEX IF NOT EXISTS gpt_sessions_user_id_idx
    ON gpt_sessions(user_id);

CREATE INDEX IF NOT EXISTS gpt_sessions_expires_at_idx
    ON gpt_sessions(expires_at);
```

#### Rust型定義

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GptSession {
    pub token: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

#### セッション解決ユーティリティ

```rust
/// セッショントークンからuser_idを解決する
/// トークンが無効または期限切れの場合は401を返す
async fn resolve_session(
    db: &PgPool,
    session_token: Uuid,
) -> ApiResult<Uuid> {
    // SELECT user_id FROM gpt_sessions
    // WHERE token = $1 AND expires_at > NOW()
    // → 見つからない場合: ApiError::unauthorized("invalid or expired session token")
    // → 見つかった場合: user_id を返却
}
```

#### フロー

1. `/gpt/auth` でユーザー登録/識別時にセッショントークンを発行
2. 以降のリクエストでは `session_token`（UUID）をリクエストボディまたはクエリパラメータで渡す
3. 各ハンドラ内で `resolve_session()` を呼び出し、トークンから `user_id` を解決
4. トークンが無効/期限切れの場合は 401 を返し、GPTが再認証を促す

---

### コンポーネント2: GPTサービス検索ハンドラ

**対応要件**: 2.1, 2.2, 2.3, 2.4

**責務**: キーワード・カテゴリによるサービス検索。キャンペーンとSponsored APIを横断的に検索し、GPTが要約しやすい統合レスポンスを返す。

**インターフェース**:

```rust
/// GET /gpt/services?q=<keyword>&category=<category>
async fn gpt_search_services(
    State(state): State<SharedState>,
    Query(params): Query<GptSearchParams>,
) -> Result<Json<GptSearchResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptSearchParams {
    /// 検索キーワード（サービス名、スポンサー名で部分一致）
    pub q: Option<String>,
    /// カテゴリフィルタ（target_tools に対応: scraping, design, storage 等）
    pub category: Option<String>,
}
```

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptSearchResponse {
    pub services: Vec<GptServiceItem>,
    pub total_count: usize,
    pub message: String,  // GPTが読み上げるサマリー
}

#[derive(Debug, Serialize)]
pub struct GptServiceItem {
    pub service_type: String,       // "campaign" | "sponsored_api"
    pub service_id: Uuid,
    pub name: String,
    pub sponsor: String,
    pub required_task: Option<String>,
    pub subsidy_amount_cents: u64,
    pub category: Vec<String>,
    pub active: bool,
}
```

**ロジック**:
1. `campaigns` テーブルから `active = true` のレコードを取得
2. `sponsored_apis` テーブルから `active = true` のレコードを取得
3. `q` パラメータがある場合、`name` / `sponsor` で部分一致フィルタ（ILIKE）
4. `category` パラメータがある場合、`target_tools` / サービスキーでフィルタ
5. 統合レスポンスを構築

---

### コンポーネント3: GPTユーザー認証ハンドラ

**対応要件**: 3.1, 3.3, 3.4

**責務**: GPT経由のユーザーを登録または既存ユーザーを識別する。メールアドレスベースの冪等操作。

**インターフェース**:

```rust
/// POST /gpt/auth
async fn gpt_auth(
    State(state): State<SharedState>,
    Json(payload): Json<GptAuthRequest>,
) -> Result<Json<GptAuthResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptAuthRequest {
    pub email: String,
    pub region: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub tools_used: Vec<String>,
}
```

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptAuthResponse {
    pub session_token: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub is_new_user: bool,
    pub message: String,
}
```

**ロジック**:
1. `email` で `users` テーブルを検索
2. 既存ユーザーが見つかった場合 → `is_new_user: false`
3. 新規の場合 → `source = "gpt_apps"` で `users` テーブルに挿入、`is_new_user: true`
4. `gpt_sessions` テーブルにセッショントークンを発行・挿入（有効期限: 30日）
5. `session_token` を含むレスポンスを返却

**セキュリティ**: `session_token` はサーバーが発行するランダムUUIDであり、ユーザーが他人のトークンを推測することは実質不可能。GPTのシステムプロンプトで「session_tokenをユーザーに表示しない」と指示する

---

### コンポーネント4: GPTタスク詳細取得ハンドラ

**対応要件**: 4.1, 4.2, 4.5

**責務**: 指定キャンペーンの必要タスク情報と、ユーザーの完了状態を返す。

**インターフェース**:

```rust
/// GET /gpt/tasks/{campaign_id}?session_token=<uuid>
async fn gpt_get_tasks(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Query(params): Query<GptTaskParams>,
) -> Result<Json<GptTaskResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptTaskParams {
    pub session_token: Uuid,
}
```

**セッション解決**: `resolve_session(db, params.session_token)` で `user_id` を取得

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptTaskResponse {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub sponsor: String,
    pub required_task: String,
    pub task_description: String,
    pub task_input_format: GptTaskInputFormat,
    pub already_completed: bool,
    pub subsidy_amount_cents: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GptTaskInputFormat {
    pub task_type: String,           // "survey", "data_provision", "registration"
    pub required_fields: Vec<String>, // ["email", "region", "feedback"]
    pub instructions: String,         // GPTがユーザーに伝える指示
}
```

**ロジック**:
1. `campaign_id` でキャンペーンを取得
2. `has_completed_task()` でユーザーの完了状態を確認
3. `task_schema` カラム（新規追加）からタスク入力フォーマットを構築
4. `task_schema` が未設定の場合はデフォルトフォーマットを返す

---

### コンポーネント5: GPTタスク完了ハンドラ

**対応要件**: 4.3, 4.4, 4.6, 6.2, 6.3

**責務**: タスク完了を記録する。同意確認を含む。

**インターフェース**:

```rust
/// POST /gpt/tasks/{campaign_id}/complete
async fn gpt_complete_task(
    State(state): State<SharedState>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<GptCompleteTaskRequest>,
) -> Result<Json<GptCompleteTaskResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptCompleteTaskRequest {
    pub session_token: Uuid,
    pub task_name: String,
    pub details: Option<String>,
    pub consent: GptConsentInput,
}

#[derive(Debug, Deserialize)]
pub struct GptConsentInput {
    pub data_sharing_agreed: bool,
    pub purpose_acknowledged: bool,
    pub contact_permission: bool,
}
```

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptCompleteTaskResponse {
    pub task_completion_id: Uuid,
    pub campaign_id: Uuid,
    pub consent_recorded: bool,
    pub can_use_service: bool,
    pub message: String,
}
```

**ロジック**:
1. `consent.data_sharing_agreed` が `false` の場合 → タスクは記録するがデータ転送はブロック、`can_use_service: true`（サービス自体は利用可能）
2. 同意情報を `consents` テーブルに記録
3. 既存の `complete_task` ロジックを内部で呼び出し（`task_completions` テーブルに記録）
4. `can_use_service: true` を返却

---

### コンポーネント6: GPTサービス実行ハンドラ

**対応要件**: 5.1, 5.2, 5.3, 5.4, 5.5

**責務**: タスク完了確認→スポンサー決済→リソース返却を1ステップで実行する統合エンドポイント。

**インターフェース**:

```rust
/// POST /gpt/services/{service}/run
async fn gpt_run_service(
    State(state): State<SharedState>,
    Path(service): Path<String>,
    Json(payload): Json<GptRunServiceRequest>,
) -> Result<Json<GptRunServiceResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptRunServiceRequest {
    pub session_token: Uuid,
    pub input: String,
}
```

**セッション解決**: `resolve_session(db, payload.session_token)` で `user_id` を取得し、既存の `run_proxy` ロジックに渡す

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptRunServiceResponse {
    pub service: String,
    pub output: String,
    pub payment_mode: String,        // "sponsored" | "user_direct"
    pub sponsored_by: Option<String>,
    pub tx_hash: Option<String>,
    pub message: String,             // GPTが読み上げるサマリー
}
```

**ロジック**:
1. 既存の `run_proxy` ロジックを内部で再利用
2. ユーザー存在確認 → キャンペーンマッチング → タスク完了確認 → スポンサー決済 → レスポンス構築
3. スポンサーが見つからない場合 → `message` に直接支払いオプションの案内を含める
4. `message` フィールドにGPTが自然言語で要約できるサマリーを含める

---

### コンポーネント7: GPTユーザー状態確認ハンドラ

**対応要件**: 4.5, 5.3

**責務**: ユーザーの登録状態、タスク完了状態、利用可能サービスを一括で返す。

**インターフェース**:

```rust
/// GET /gpt/user/status?session_token=<uuid>
async fn gpt_user_status(
    State(state): State<SharedState>,
    Query(params): Query<GptUserStatusParams>,
) -> Result<Json<GptUserStatusResponse>, ApiError>
```

**リクエスト型**:

```rust
#[derive(Debug, Deserialize)]
pub struct GptUserStatusParams {
    pub session_token: Uuid,
}
```

**セッション解決**: `resolve_session(db, params.session_token)` で `user_id` を取得

**レスポンス型**:

```rust
#[derive(Debug, Serialize)]
pub struct GptUserStatusResponse {
    pub user_id: Uuid,
    pub email: String,
    pub completed_tasks: Vec<GptCompletedTaskSummary>,
    pub available_services: Vec<GptAvailableService>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GptCompletedTaskSummary {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub task_name: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GptAvailableService {
    pub service: String,
    pub sponsor: String,
    pub ready: bool,  // タスク完了済みで即利用可能か
}
```

---

### コンポーネント8: 同意管理

**対応要件**: 6.1, 6.2, 6.3, 6.4

#### DBスキーマ

```sql
-- マイグレーション: 0007_consents.sql
CREATE TABLE IF NOT EXISTS consents (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    consent_type TEXT NOT NULL CHECK (consent_type IN ('data_sharing', 'contact', 'retention')),
    granted BOOLEAN NOT NULL,
    purpose TEXT,
    retention_days INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS consents_user_campaign_idx
    ON consents(user_id, campaign_id);

CREATE INDEX IF NOT EXISTS consents_user_id_idx
    ON consents(user_id);
```

#### Rust型定義

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Consent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub campaign_id: Uuid,
    pub consent_type: String,
    pub granted: bool,
    pub purpose: Option<String>,
    pub retention_days: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

---

### コンポーネント9: ユーザーソース追跡

**対応要件**: 3.4

#### DBスキーマ変更

```sql
-- マイグレーション: 0008_add_user_source.sql
ALTER TABLE users ADD COLUMN IF NOT EXISTS source TEXT DEFAULT 'web';
```

#### 型変更

`UserProfile` に `source: Option<String>` フィールドを追加。既存のクエリとの後方互換性を維持するため `Option` とする。

---

### コンポーネント10: OpenAPIスキーマ

**対応要件**: 1.1, 1.2, 1.3, 1.5, 9.1

**配信エンドポイント**: `GET /.well-known/openapi.yaml`

**スキーマ構造**:

```yaml
openapi: "3.1.0"
info:
  title: "SnapFuel GPT Actions API"
  description: "スポンサー付きx402サービスの検索、タスク実行、支払いを行うAPI"
  version: "1.0.0"
servers:
  - url: "https://<PUBLIC_BASE_URL>"
    description: "本番環境"
paths:
  /gpt/services:
    get:
      operationId: searchServices
      summary: "利用可能なスポンサー付きサービスを検索する"
      description: "ユーザーがサービスを探している時に呼び出す。キーワードやカテゴリでフィルタリング可能。"
      # ...
  /gpt/auth:
    post:
      operationId: authenticateUser
      summary: "ユーザーを登録または識別する"
      description: "サービス利用前にユーザーのメールアドレスとリージョンで登録する。既存ユーザーの場合は既存情報を返す。"
      # ...
  /gpt/tasks/{campaign_id}:
    get:
      operationId: getTaskDetails
      summary: "キャンペーンの必要タスク詳細を取得する"
      description: "ユーザーがサービスを選択した後、必要なタスクの詳細と入力フォーマットを取得する。"
      # ...
  /gpt/tasks/{campaign_id}/complete:
    post:
      operationId: completeTask
      summary: "タスクを完了し同意を記録する"
      description: "ユーザーがタスクに必要な情報を提供した後に呼び出す。同意情報も同時に記録する。"
      # ...
  /gpt/services/{service}/run:
    post:
      operationId: runService
      summary: "スポンサー決済でサービスを実行する"
      description: "タスク完了済みのユーザーがサービスを実行する。スポンサーが支払いを肩代わりし、リソースを返却する。"
      # ...
  /gpt/user/status:
    get:
      operationId: getUserStatus
      summary: "ユーザーの状態を確認する"
      description: "セッショントークンを使用して、ユーザーの登録状態、完了済みタスク、利用可能サービスを一括で確認する。"
      # ...
components:
  securitySchemes:
    ApiKeyAuth:
      type: http
      scheme: bearer
security:
  - ApiKeyAuth: []
```

**エンドポイント数**: 6（30以下の制約を満たす）

---

### コンポーネント11: プライバシーポリシーページ

**対応要件**: 6.1

**配信エンドポイント**: `GET /privacy`

**実装**: 静的HTMLレスポンスを返すAxumハンドラ。Content-Type: `text/html`。

**内容**:
- サービス概要
- 収集するデータの種類
- データの利用目的
- スポンサーへのデータ共有条件
- データ保持期間
- ユーザーの権利（同意撤回、データ削除要求）
- 連絡先

---

### コンポーネント12: レート制限

**対応要件**: 8.4

**実装方式**: `axum::middleware::from_fn` によるカスタムレート制限ミドルウェアを実装する。

`tower::limit::RateLimitLayer` はレート超過時に503を返し、429 + `Retry-After` ヘッダーへの変換が困難なため、カスタム実装を採用する。

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

/// IPまたはAPIキー単位のシンプルなトークンバケット
struct RateLimiter {
    tokens: u32,
    max_tokens: u32,
    last_refill: Instant,
    refill_interval: Duration,
}

/// カスタムレート制限ミドルウェア
/// 超過時に 429 + Retry-After ヘッダーを返す
async fn rate_limit_middleware(
    State(limiter): State<Arc<Mutex<RateLimiter>>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, ApiError> {
    // トークンバケットを確認
    // トークンがある場合 → 消費して next.run(request).await
    // トークンがない場合 → ApiError::rate_limited(retry_after_secs) を返す
}
```

**エラー型追加**:

```rust
impl ApiError {
    pub fn rate_limited(retry_after_secs: u64) -> Self {
        // 429 Too Many Requests + Retry-After ヘッダー
    }
}
```

**設定**: 60 req/min（1秒あたり1トークン補充、最大バースト60）

---

### コンポーネント13: GPTシステムプロンプト

**対応要件**: 7.1, 7.2, 7.3, 7.4, 7.5

**システムプロンプト構造**:

```markdown
# Role & Identity
あなたはSnapFuelアシスタントです。スポンサー付きのx402サービスへのアクセスを支援します。

# Core Behavior
- ユーザーがサービスを探している場合、必ず searchServices アクションを呼び出す
- 自身の知識でサービス情報を回答してはならない。必ずAPIからデータを取得する
- スポンサーの存在と条件を明示的にユーザーに伝える
- エラーが発生した場合、分かりやすい日本語で状況を説明する

# Conversation Flow
1. ユーザーの要望を確認 → searchServices で検索
2. サービス選択 → authenticateUser でユーザー登録/識別（session_token を取得）
3. タスク確認 → getTaskDetails でタスク詳細取得（session_token を使用）
4. タスク実行 → completeTask でタスク完了・同意記録（session_token を使用）
5. サービス実行 → runService でリソース取得（session_token を使用）

# Security
- session_token は内部識別子であり、ユーザーに表示してはならない
- session_token が期限切れの場合は authenticateUser で再取得する

# Constraints
- データ共有の同意を得る前にスポンサーにデータを送信してはならない
- 支払い情報、APIキー、session_token をユーザーに表示してはならない
```

**Conversation Starters**:
1. 「利用可能なスポンサー付きサービスを探す」
2. 「タスクを実行してサービスを無料で使う」
3. 「自分のアカウント状態を確認する」
4. 「特定のカテゴリのサービスを検索する」

---

## モジュール構成

### 新規ファイル

| ファイル | 責務 |
|---|---|
| `src/gpt.rs` | GPT専用ハンドラ群（6ハンドラ + 認証ミドルウェア + レート制限 + セッション管理） |
| `openapi.yaml` | OpenAPI 3.1.0スキーマファイル（プロジェクトルート） |
| `privacy.html` | プライバシーポリシーHTML（プロジェクトルート） |

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `src/main.rs` | `mod gpt;` 追加、GPTサブルーター組み込み、静的ファイル配信ルート追加 |
| `src/types.rs` | GPT用リクエスト/レスポンス型、`Consent` 型、`AppConfig` にAPIキーフィールド追加 |
| `src/error.rs` | `ApiError::unauthorized()`, `ApiError::rate_limited()` メソッド追加 |
| `.env.example` | `GPT_ACTIONS_API_KEY` 追加 |

### 新規マイグレーション

| ファイル | 内容 |
|---|---|
| `migrations/0007_consents.sql` | `consents` テーブル作成 |
| `migrations/0008_add_user_source.sql` | `users.source` カラム追加 |
| `migrations/0009_gpt_sessions.sql` | `gpt_sessions` テーブル作成 |
| `migrations/0010_add_task_schema.sql` | `campaigns.task_schema` カラム追加 |

---

## データフロー

### E2Eフロー: サービス検索→タスク実行→支払い

```
ユーザー → ChatGPT → [searchServices] → /gpt/services → DB(campaigns, sponsored_apis)
                                                          ↓
ユーザー ← ChatGPT ← サービス一覧レスポンス ←────────────┘

ユーザー → ChatGPT → [authenticateUser] → /gpt/auth → DB(users, gpt_sessions)
                                                       ↓
ユーザー ← ChatGPT ← session_token + ユーザーID ←────┘
                      (session_tokenはGPT内部で保持、ユーザーには非表示)

ユーザー → ChatGPT → [getTaskDetails] → /gpt/tasks/:id?session_token=xxx
                                          ↓ resolve_session() → user_id
                                        DB(campaigns, task_completions)
                                          ↓
ユーザー ← ChatGPT ← タスク詳細 + 完了状態 ←┘

ユーザー → ChatGPT → [completeTask] → /gpt/tasks/:id/complete {session_token}
                                        ↓ resolve_session() → user_id
                                      DB(task_completions, consents)
                                        ↓
ユーザー ← ChatGPT ← 完了確認 ←────────┘

ユーザー → ChatGPT → [runService] → /gpt/services/:service/run {session_token}
                                      ↓ resolve_session() → user_id
                                    キャンペーンマッチ → タスク完了確認
                                      ↓
                                    スポンサー決済 → DB(payments, campaigns budget更新)
                                      ↓
ユーザー ← ChatGPT ← サービス実行結果 ←┘
```

---

## セキュリティ考慮事項

| 項目 | 対策 |
|---|---|
| APIキー管理 | 環境変数 `GPT_ACTIONS_API_KEY` で管理。ハードコード禁止 |
| ユーザー識別 | セッショントークン方式を採用。`user_id` を直接受け取らず、サーバー発行の `session_token` から `resolve_session()` で解決。なりすまし・データ閲覧を防止 |
| セッション有効期限 | 30日間。期限切れトークンは401を返し再認証を促す |
| レート制限 | GPTルートに60 req/minのカスタムトークンバケットを適用。超過時は429 + Retry-After |
| 入力バリデーション | 全リクエストパラメータを `ApiError::validation()` で検証 |
| データ漏洩防止 | 内部エラー詳細を外部に返さない（既存 `ApiError::Internal` パターン） |
| 同意管理 | データ転送前に `consents` テーブルで同意状態を確認 |
| CORS | 既存の `cors_layer_from_env()` で `chat.openai.com` を許可 |

---

## テスト戦略

| テストレベル | 対象 | 方法 |
|---|---|---|
| ユニットテスト | 認証ミドルウェア、検索ロジック、同意ガード | `cargo test` |
| 統合テスト | GPTエンドポイントE2Eフロー | テスト用DBでのHTTPリクエストテスト |
| GPT Builderテスト | OpenAPIスキーマ解釈、会話フロー | ChatGPT UI上での手動テスト |
| スキーマ検証 | OpenAPI 3.1.0準拠 | `swagger-cli validate openapi.yaml` |

---

## 後方互換性

- 既存の全APIエンドポイントは変更なし
- 新規エンドポイントは `/gpt/*` プレフィックスで完全に分離
- DBマイグレーションは `ADD COLUMN IF NOT EXISTS` / `CREATE TABLE IF NOT EXISTS` で安全
- `users.source` カラムは `DEFAULT 'web'` で既存レコードに影響なし
