# Smart Service Suggestion — 技術設計書

## 1. 概要

本設計書は、`smart-service-suggestion` フィーチャーの技術的アーキテクチャを定義する。既存の `gpt-apps-integration` で構築された `GET /gpt/services` エンドポイントを後方互換で拡張し、予算・意図・タスク嗜好ベースのスマートサジェスト機能を追加する。

### アーキテクチャパターン

**インメモリフィルタリング拡張パターン** — 既存の `gpt_search_services` ハンドラが採用するパターン（DBから全アクティブサービスを取得→Rust側でフィルタ）を踏襲し、新しいフィルタ条件（予算・意図・嗜好）とスコアリングロジックを追加する。

**選定理由**: データ量が少ない初期段階ではインメモリ処理で十分であり、既存コードとの一貫性を維持できる。将来的にデータ量が増加した場合はDBレベルフィルタリングに移行可能（`research.md` 参照）。

---

## 2. 要件トレーサビリティ

| 要件ID | 要件名 | 対応コンポーネント |
|---|---|---|
| 1.1–1.4 | 予算ベースのサービスフィルタリング | C1: 拡張検索ハンドラ |
| 2.1–2.4 | 意図ベースのサービスサジェスト | C1: 拡張検索ハンドラ, C4: キャンペーンタグ |
| 3.1–3.5 | タスク嗜好プロフィール | C2: 嗜好管理ハンドラ, C3: 嗜好DBスキーマ |
| 4.1–4.5 | 嗜好ベースのサービスフィルタリング | C1: 拡張検索ハンドラ |
| 5.1–5.4 | キャンペーンタグ管理 | C4: キャンペーンタグ, C5: DBマイグレーション |
| 6.1–6.5 | 拡張検索APIエンドポイント | C1: 拡張検索ハンドラ, C6: 型定義 |
| 7.1–7.4 | GPTシステムプロンプト拡張 | C7: GPT構成更新 |
| 8.1–8.5 | タスク嗜好管理APIエンドポイント | C2: 嗜好管理ハンドラ, C8: OpenAPIスキーマ |
| 9.1–9.4 | 後方互換性・既存機能との整合 | 全コンポーネント |

---

## 3. テクノロジースタック & アラインメント

既存の `gpt-apps-integration` と同一のスタックを使用。新規依存関係の追加なし。

| 項目 | 技術 | 用途 |
|---|---|---|
| Rust / Axum 0.8 | Web フレームワーク | ハンドラ定義、ルーティング |
| SQLx 0.8 / PostgreSQL | データベース | 嗜好テーブル、タグカラム |
| Serde | シリアライズ | 新型定義 |
| UUID | 識別子 | 嗜好レコードID |
| chrono | 日時 | 嗜好更新日時 |

---

## 4. コンポーネント & インターフェース定義

### C1: 拡張検索ハンドラ (`gpt_search_services` の拡張)

**変更対象ファイル**: `src/gpt.rs`
**対応要件**: 1.1–1.4, 2.1–2.4, 4.1–4.5, 6.1–6.5

#### 拡張パラメータ

```
GptSearchParams {
    q: Option<String>,              // 既存: キーワード検索
    category: Option<String>,       // 既存: カテゴリフィルタ
    max_budget_cents: Option<u64>,  // 新規: 予算上限 (要件 1.1)
    intent: Option<String>,         // 新規: 自然言語の意図 (要件 2.1)
    session_token: Option<Uuid>,    // 新規: 嗜好適用用 (要件 4.1)
}
```

#### 拡張レスポンス

```
GptSearchResponse {
    services: Vec<GptServiceItem>,          // 既存
    total_count: usize,                     // 既存
    message: String,                        // 既存
    applied_filters: Option<AppliedFilters>,  // 新規 (要件 6.3)
    available_categories: Option<Vec<String>>, // 新規 (要件 2.3)
}

GptServiceItem {
    // ... 既存フィールド ...
    relevance_score: Option<f64>,  // 新規 (要件 6.5)
}

AppliedFilters {
    budget: Option<u64>,
    intent: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
    preferences_applied: bool,  // 要件 4.5
}
```

#### 処理フロー

```
gpt_search_services(params) {
    1. DB から全アクティブサービスを取得（既存ロジック）
    2. 既存フィルタ適用: q, category（既存ロジック）
    3. 予算フィルタ適用 (要件 1.1):
       if max_budget_cents.is_some() {
           services.retain(|s| s.subsidy_amount_cents <= max_budget_cents)
       }
    4. 意図フィルタ適用 (要件 2.1):
       if intent.is_some() {
           intent_keywords = intent.split_whitespace()
           services.retain(|s| matches_intent(s, intent_keywords))
       }
    5. 嗜好フィルタ適用 (要件 4.1–4.2):
       if session_token.is_some() {
           user_id = resolve_session(db, session_token)
           preferences = load_preferences(db, user_id)
           services.retain(|s| !is_avoided(s, preferences))  // 要件 4.1
           // preferred タスクのスコアブースト (要件 4.2)
       }
    6. スコア算出 (要件 6.5):
       for service in &mut services {
           service.relevance_score = calculate_score(service, params, preferences)
       }
    7. スコア降順ソート (要件 1.4)
    8. メッセージ生成:
       - 0件: 予算緩和/嗜好更新/直接支払いの案内 (要件 1.3, 4.4)
       - 0件 + intent指定: 全カテゴリ一覧を返却 (要件 2.3)
    9. AppliedFilters 構築 (要件 6.3)
    10. レスポンス返却
}
```

#### 意図マッチング関数 `matches_intent`

```
matches_intent(service: &GptServiceItem, intent_keywords: &[&str]) -> bool {
    検索対象フィールド:
    - service.name
    - service.required_task (Some の場合)
    - service.category (各要素)
    - service.tags (新規フィールド、要件 2.4)

    マッチ条件: いずれかのフィールドにいずれかのキーワードが部分一致 (case-insensitive)
}
```

#### スコア算出関数 `calculate_score`

```
calculate_score(service, params, preferences) -> f64 {
    budget_score (重み 0.3):
        max_budget_cents 未指定 → 0.5 (中立)
        指定あり → 1.0 - (subsidy_amount_cents / max_budget_cents).min(1.0)

    intent_score (重み 0.4):
        intent 未指定 → 0.5 (中立)
        指定あり → matched_fields / total_searchable_fields

    preference_score (重み 0.3):
        嗜好未登録 → 0.5 (中立)
        preferred タスク → 1.0
        neutral タスク → 0.5
        avoided → 0.0 (フィルタ済みのため通常到達しない)

    total = budget_score * 0.3 + intent_score * 0.4 + preference_score * 0.3
    (0.0 ≤ total ≤ 1.0)
}
```

---

### C2: 嗜好管理ハンドラ

**変更対象ファイル**: `src/gpt.rs`
**対応要件**: 3.1–3.5, 8.1–8.5

#### `GET /gpt/preferences` — 嗜好取得

```
gpt_get_preferences(
    State(state): State<SharedState>,
    Query(params): Query<GptPreferencesParams>,
) -> Response

GptPreferencesParams {
    session_token: Uuid,
}

GptPreferencesResponse {
    user_id: Uuid,
    preferences: Vec<TaskPreference>,
    updated_at: Option<DateTime<Utc>>,
    message: String,
}

TaskPreference {
    task_type: String,        // "survey", "data_provision", "github_pr", etc.
    level: String,            // "preferred", "neutral", "avoided"
}
```

**処理フロー**:
1. `resolve_session(db, session_token)` でユーザーID取得
2. `user_task_preferences` テーブルからユーザーの嗜好を取得
3. 嗜好が未登録の場合、空配列 + 案内メッセージを返却

#### `POST /gpt/preferences` — 嗜好登録・更新

```
gpt_set_preferences(
    State(state): State<SharedState>,
    Json(payload): Json<GptSetPreferencesRequest>,
) -> Response

GptSetPreferencesRequest {
    session_token: Uuid,
    preferences: Vec<TaskPreference>,
}

GptSetPreferencesResponse {
    user_id: Uuid,
    preferences_count: usize,
    updated_at: DateTime<Utc>,
    message: String,
}
```

**処理フロー**:
1. `resolve_session(db, session_token)` でユーザーID取得
2. 既存の嗜好を全削除（`DELETE FROM user_task_preferences WHERE user_id = $1`）
3. 新しい嗜好を一括挿入
4. 更新日時を記録

---

### C3: 嗜好DBスキーマ

**変更対象ファイル**: `migrations/0011_user_task_preferences.sql`
**対応要件**: 3.1–3.5

```sql
CREATE TABLE IF NOT EXISTS user_task_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('preferred', 'neutral', 'avoided')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, task_type)
);

CREATE INDEX IF NOT EXISTS user_task_preferences_user_id_idx
    ON user_task_preferences(user_id);
```

**設計判断**:
- `UNIQUE (user_id, task_type)` 制約で同一タスクタイプの重複を防止
- `level` は CHECK 制約で `preferred` / `neutral` / `avoided` の3値に限定
- `ON DELETE CASCADE` でユーザー削除時に嗜好も自動削除

---

### C4: キャンペーンタグ

**変更対象ファイル**: `migrations/0012_campaign_tags.sql`
**対応要件**: 5.1–5.4

```sql
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';
```

**デフォルトタグ推定ロジック** (要件 5.3):

検索ハンドラ内で、`tags` が空の場合に `target_tools` と `required_task` からデフォルトタグを生成:

```
infer_tags(campaign) -> Vec<String> {
    if !campaign.tags.is_empty() {
        return campaign.tags
    }
    let mut tags = campaign.target_tools.clone();
    tags.push(campaign.required_task.clone());
    tags.dedup();
    tags
}
```

#### 拡張 CampaignRow

```
CampaignRow {
    // ... 既存フィールド ...
    tags: Vec<String>,  // 新規
}
```

SQLクエリの SELECT に `tags` を追加:
```sql
SELECT id, name, sponsor, required_task, subsidy_per_call_cents, target_tools, active, tags
FROM campaigns WHERE active = true
```

#### 拡張 GptServiceItem

```
GptServiceItem {
    // ... 既存フィールド ...
    tags: Vec<String>,           // 新規 (要件 5.1)
    relevance_score: Option<f64>, // 新規 (要件 6.5)
}
```

---

### C5: DBマイグレーション一覧

| ファイル | 内容 | 対応要件 |
|---|---|---|
| `migrations/0011_user_task_preferences.sql` | ユーザータスク嗜好テーブル | 3.1–3.5 |
| `migrations/0012_campaign_tags.sql` | キャンペーンタグカラム追加 | 5.1–5.4, 9.2 |

---

### C6: 型定義

**変更対象ファイル**: `src/types.rs`
**対応要件**: 6.1–6.5, 3.2, 8.1–8.5

#### 新規型

```rust
// --- 嗜好管理 ---

#[derive(Debug, Deserialize)]
pub struct GptPreferencesParams {
    pub session_token: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GptSetPreferencesRequest {
    pub session_token: Uuid,
    pub preferences: Vec<TaskPreference>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskPreference {
    pub task_type: String,
    pub level: String,  // "preferred" | "neutral" | "avoided"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GptPreferencesResponse {
    pub user_id: Uuid,
    pub preferences: Vec<TaskPreference>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GptSetPreferencesResponse {
    pub user_id: Uuid,
    pub preferences_count: usize,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub message: String,
}

// --- 拡張検索 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct AppliedFilters {
    pub budget: Option<u64>,
    pub intent: Option<String>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub preferences_applied: bool,
}
```

#### 既存型の拡張

```rust
// GptSearchParams に新フィールド追加
#[derive(Debug, Deserialize)]
pub struct GptSearchParams {
    pub q: Option<String>,
    pub category: Option<String>,
    pub max_budget_cents: Option<u64>,   // 新規
    pub intent: Option<String>,          // 新規
    pub session_token: Option<Uuid>,     // 新規
}

// GptSearchResponse に新フィールド追加
#[derive(Debug, Serialize, Deserialize)]
pub struct GptSearchResponse {
    pub services: Vec<GptServiceItem>,
    pub total_count: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_filters: Option<AppliedFilters>,       // 新規
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_categories: Option<Vec<String>>,     // 新規
}

// GptServiceItem に新フィールド追加
#[derive(Debug, Serialize, Deserialize)]
pub struct GptServiceItem {
    // ... 既存フィールド ...
    pub tags: Vec<String>,                              // 新規
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance_score: Option<f64>,                   // 新規
}
```

**後方互換性の保証** (要件 9.1):
- 新フィールドは全て `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]` で定義
- 拡張パラメータ未指定時は `applied_filters: None`, `available_categories: None`, `relevance_score: None` となり、既存レスポンスと同一のJSON構造を返却

---

### C7: GPT構成更新

**変更対象ファイル**: `.kiro/specs/smart-service-suggestion/gpt-config-update.md`
**対応要件**: 7.1–7.4

#### システムプロンプト追加セクション

```
# Smart Service Suggestion

ユーザーがサービスを探している場合、以下の情報を会話で収集し、searchServices の拡張パラメータに変換する:

1. **意図の確認**: 「何をしたいですか？」と尋ね、回答を `intent` パラメータに設定
2. **予算の確認**: 「予算はありますか？（任意）」と尋ね、回答を `max_budget_cents` に変換
3. **嗜好の確認**: 「避けたいタスクはありますか？（例：個人情報の共有、アンケート回答）」と尋ね、必要に応じて setPreferences を呼び出す

ユーザーがサービス名を明示した場合は、従来通り `q` パラメータで検索する。

検索結果に `relevance_score` が含まれる場合、スコアが高いサービスを優先的に提案し、
「あなたの条件に最もマッチするサービスです」のように説明する。

`preferences_applied: true` の場合、「お好みの設定に基づいてフィルタリングしました」と伝える。
```

#### 追加 Conversation Starter

```
| # | Conversation Starter |
|---|---|
| 5 | 自分の好みを設定する |
```

---

### C8: OpenAPIスキーマ拡張

**変更対象ファイル**: `openapi.yaml`
**対応要件**: 8.5, 9.3

#### 既存エンドポイントの拡張: `GET /gpt/services`

パラメータ追加:
```yaml
- name: max_budget_cents
  in: query
  required: false
  schema:
    type: integer
  description: Maximum budget in cents for service filtering
- name: intent
  in: query
  required: false
  schema:
    type: string
  description: Natural language description of what the user wants to do
- name: session_token
  in: query
  required: false
  schema:
    type: string
    format: uuid
  description: Session token to apply user preferences for filtering
```

レスポンス拡張:
```yaml
applied_filters:
  type: object
  properties:
    budget: { type: integer, nullable: true }
    intent: { type: string, nullable: true }
    category: { type: string, nullable: true }
    keyword: { type: string, nullable: true }
    preferences_applied: { type: boolean }
available_categories:
  type: array
  items: { type: string }
  nullable: true
```

`GptServiceItem` 拡張:
```yaml
tags:
  type: array
  items: { type: string }
relevance_score:
  type: number
  format: double
  nullable: true
```

#### 新規エンドポイント: `GET /gpt/preferences`

```yaml
/gpt/preferences:
  get:
    operationId: getPreferences
    summary: Get user task preferences
    description: >-
      Call this to retrieve the user's current task preferences.
      Returns preferred, neutral, and avoided task types.
    parameters:
      - name: session_token
        in: query
        required: true
        schema:
          type: string
          format: uuid
    responses:
      "200":
        description: Preferences retrieved
        content:
          application/json:
            schema:
              type: object
              properties:
                user_id: { type: string, format: uuid }
                preferences:
                  type: array
                  items:
                    type: object
                    properties:
                      task_type: { type: string }
                      level: { type: string, enum: [preferred, neutral, avoided] }
                updated_at: { type: string, format: date-time, nullable: true }
                message: { type: string }
      "401":
        description: Invalid or expired session token
```

#### 新規エンドポイント: `POST /gpt/preferences`

```yaml
  post:
    operationId: setPreferences
    summary: Set user task preferences
    description: >-
      Call this when the user wants to set their task preferences.
      Replaces all existing preferences with the provided list.
    requestBody:
      required: true
      content:
        application/json:
          schema:
            type: object
            required: [session_token, preferences]
            properties:
              session_token:
                type: string
                format: uuid
              preferences:
                type: array
                items:
                  type: object
                  required: [task_type, level]
                  properties:
                    task_type:
                      type: string
                      description: "Task type (e.g. survey, data_provision, github_pr)"
                    level:
                      type: string
                      enum: [preferred, neutral, avoided]
    responses:
      "200":
        description: Preferences updated
        content:
          application/json:
            schema:
              type: object
              properties:
                user_id: { type: string, format: uuid }
                preferences_count: { type: integer }
                updated_at: { type: string, format: date-time }
                message: { type: string }
      "401":
        description: Invalid or expired session token
```

---

## 5. ルーター統合

**変更対象ファイル**: `src/main.rs`

`build_gpt_router` に2ルートを追加:

```
Router::new()
    // ... 既存ルート ...
    .route("/preferences", get(gpt::gpt_get_preferences))
    .route("/preferences", post(gpt::gpt_set_preferences))
    // ... 既存ミドルウェア（認証 + レート制限）はそのまま適用 ...
```

**エンドポイント総数**: 既存6 + 新規2 = 8（上限30以下、要件 9.3 充足）

---

## 6. メトリクス統合

**対応要件**: 9.4

新規ハンドラのメトリクスラベル:
- `gpt_get_preferences` — `GET /gpt/preferences`
- `gpt_set_preferences` — `POST /gpt/preferences`

既存の `respond()` ユーティリティを使用し、`gpt_` プレフィックスで Prometheus メトリクスに統合。

---

## 7. E2E フロー図

```
ユーザー → GPT → SubsidyPayment API

[スマートサジェストフロー]

1. ユーザー: 「Webサイトのスクリーンショットを撮りたい。予算は500円くらい」
   ↓
2. GPT: intent="スクリーンショット", max_budget_cents=500 を抽出
   → GET /gpt/services?intent=スクリーンショット&max_budget_cents=500
   ↓
3. API: 予算内 + 意図マッチのサービスを relevance_score 降順で返却
   ↓
4. GPT: 「以下のサービスが見つかりました（お好みに基づきフィルタリング済み）」

[嗜好設定フロー]

1. ユーザー: 「個人情報の共有は避けたい。GitHubのPRは積極的にやりたい」
   ↓
2. GPT: authenticateUser → session_token 取得
   → POST /gpt/preferences
     { session_token, preferences: [
       { task_type: "data_provision", level: "avoided" },
       { task_type: "github_pr", level: "preferred" }
     ]}
   ↓
3. API: 嗜好を永続化、確認レスポンス返却
   ↓
4. 以降の検索: GET /gpt/services?session_token=xxx
   → data_provision タスクのサービスが自動除外
   → github_pr タスクのサービスが上位にランキング
```

---

## 8. テスト戦略

### ユニットテスト

| テスト | 対応要件 | 内容 |
|---|---|---|
| マイグレーション 0011 スキーマ検証 | 3.1 | user_task_preferences テーブル構造 |
| マイグレーション 0012 スキーマ検証 | 5.1 | campaigns.tags カラム |
| 新規型構築テスト | 6.1 | TaskPreference, AppliedFilters 等の構築 |
| スコア算出テスト | 6.5 | calculate_score の各パターン |
| 意図マッチングテスト | 2.1 | matches_intent の各パターン |
| デフォルトタグ推定テスト | 5.3 | infer_tags の動作 |

### 統合テスト (DATABASE_URL 必要)

| テスト | 対応要件 | 内容 |
|---|---|---|
| 予算フィルタ適用 | 1.1 | max_budget_cents 指定時のフィルタリング |
| 予算未指定時の全件返却 | 1.2 | 既存動作の維持 |
| 予算フィルタ 0件メッセージ | 1.3 | 案内メッセージの検証 |
| 意図検索マッチ | 2.1 | intent パラメータでの検索 |
| 意図検索 0件カテゴリ返却 | 2.3 | available_categories の返却 |
| 嗜好登録 | 3.1 | POST /gpt/preferences |
| 嗜好取得 | 3.3 | GET /gpt/preferences |
| 嗜好更新 | 3.4 | 上書き動作の検証 |
| avoided 除外 | 4.1 | 嗜好適用時のフィルタリング |
| preferred ランキング | 4.2 | スコアブーストの検証 |
| 嗜好未登録時の全件返却 | 4.3 | 既存動作の維持 |
| preferences_applied フラグ | 4.5 | レスポンスフラグの検証 |
| 複合フィルタ AND 条件 | 6.2 | 全フィルタ同時指定 |
| applied_filters レスポンス | 6.3 | フィルタ情報の返却 |
| 拡張パラメータなしの後方互換 | 6.4, 9.1 | 既存動作の完全一致 |
| relevance_score 算出 | 6.5 | スコアの正確性 |

### ルーター統合テスト

| テスト | 対応要件 | 内容 |
|---|---|---|
| /gpt/preferences GET 到達性 | 8.2 | ルーティング確認 |
| /gpt/preferences POST 到達性 | 8.1 | ルーティング確認 |
| 認証ミドルウェア適用 | 8.4 | API キー検証 |
| 既存ルート非破壊 | 9.1 | /health, /gpt/services 等 |
| OpenAPI スキーマ更新 | 8.5 | 新 operationId の存在確認 |
| メトリクス記録 | 9.4 | gpt_ プレフィックスラベル |

### E2E テスト

| テスト | 内容 |
|---|---|
| スマートサジェスト全フロー | 嗜好設定 → 意図+予算検索 → フィルタリング結果検証 |

---

## 9. 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `src/types.rs` | 変更 | GptSearchParams, GptSearchResponse, GptServiceItem 拡張 + 新規型追加 |
| `src/gpt.rs` | 変更 | gpt_search_services 拡張 + gpt_get_preferences, gpt_set_preferences 追加 |
| `src/main.rs` | 変更 | build_gpt_router に /preferences ルート追加 |
| `src/test.rs` | 変更 | 新規テスト追加 |
| `migrations/0011_user_task_preferences.sql` | 新規 | ユーザータスク嗜好テーブル |
| `migrations/0012_campaign_tags.sql` | 新規 | campaigns.tags カラム追加 |
| `openapi.yaml` | 変更 | パラメータ拡張 + /gpt/preferences 追加 |
| `.kiro/specs/smart-service-suggestion/gpt-config-update.md` | 新規 | GPT システムプロンプト更新内容 |
