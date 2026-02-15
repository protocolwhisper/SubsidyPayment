# Smart Service Suggestion — ディスカバリログ

## サマリー

**ディスカバリタイプ**: Extension（既存システム拡張）— Integration-focused discovery
**スコープ**: 既存の `gpt-apps-integration` の `GET /gpt/services` エンドポイントを拡張し、予算・意図・タスク嗜好ベースのスマートサジェスト機能を追加する。

### 主要な発見事項

1. **既存の検索ハンドラ `gpt_search_services` はインメモリフィルタリングパターン**を採用 — DBから全アクティブサービスを取得後、Rust側で `q`/`category` フィルタを適用。拡張パラメータも同パターンで追加可能。
2. **`GptSearchParams` は `Deserialize` のみ実装** — `q: Option<String>`, `category: Option<String>` の2フィールド。新パラメータ追加は型定義の拡張のみで対応可能。
3. **セッション解決 `resolve_session()` は汎用関数として分離済み** — 嗜好適用時のユーザー識別に再利用可能。
4. **`campaigns` テーブルには `tags` カラムが未存在** — `target_tools` (text[]) と `required_task` (text) が代替的にカテゴリ/タスクタイプ情報を保持。タグカラム追加が必要。

---

## リサーチログ

### トピック 1: 既存検索ハンドラの構造

**調査内容**: `src/gpt.rs` の `gpt_search_services` 関数の実装パターン

**発見事項**:
- `campaigns` テーブルと `sponsored_apis` テーブルから全アクティブレコードを取得
- `GptServiceItem` に変換後、インメモリで `q`（名前/スポンサー名の部分一致）と `category`（target_tools完全一致）でフィルタ
- レスポンスは `GptSearchResponse { services, total_count, message }`
- `respond()` ユーティリティでメトリクス記録 + レスポンス変換

**設計への影響**:
- 新パラメータ（`max_budget_cents`, `intent`, `session_token`）は同じインメモリフィルタリングチェーンに追加
- `intent` パラメータは `q` の拡張版として、`name` + `required_task` + `target_tools` + `tags` に対するマルチフィールド検索
- `session_token` が指定された場合のみ嗜好ロードを実行（オプショナル）

### トピック 2: DBスキーマの拡張ポイント

**調査内容**: `campaigns` テーブルの現在のスキーマと拡張方法

**発見事項**:
- `campaigns` テーブルの主要カラム: `id`, `name`, `sponsor`, `target_roles`, `target_tools`, `required_task`, `subsidy_per_call_cents`, `budget_total_cents`, `budget_remaining_cents`, `query_urls`, `active`, `task_schema`
- `target_tools` (text[]) がカテゴリ情報を兼務 — タグ機能と重複する可能性
- `users` テーブルには嗜好情報を保持するカラムが未存在

**設計への影響**:
- `campaigns` に `tags text[]` カラムを `ADD COLUMN IF NOT EXISTS` で追加
- 新テーブル `user_task_preferences` を作成（ユーザーID + タスクタイプ + 嗜好レベル）
- `target_tools` と `tags` は共存させ、`tags` が未設定の場合は `target_tools` + `required_task` からデフォルトタグを推定

### トピック 3: 型定義の拡張パターン

**調査内容**: `src/types.rs` の既存GPT型定義パターン

**発見事項**:
- 全GPT型は `src/types.rs` に集約
- リクエスト型: `Deserialize` のみ、レスポンス型: `Serialize + Deserialize`
- `Option<T>` + `#[serde(default)]` パターンでオプショナルフィールドを表現
- `GptSearchParams` は `Query` エクストラクタで使用（URLクエリパラメータ）

**設計への影響**:
- `GptSearchParams` に `max_budget_cents: Option<u64>`, `intent: Option<String>`, `session_token: Option<Uuid>` を追加
- `GptSearchResponse` に `applied_filters: Option<AppliedFilters>`, `available_categories: Option<Vec<String>>` を追加
- `GptServiceItem` に `relevance_score: Option<f64>` を追加
- 新型: `TaskPreference`, `TaskPreferenceLevel`, `GptPreferencesRequest`, `GptPreferencesResponse`

### トピック 4: ルーター統合パターン

**調査内容**: `src/main.rs` の `build_gpt_router` 関数

**発見事項**:
- GPTルーターは `Router::new()` + `.route()` チェーンで構築
- 認証ミドルウェア (`verify_gpt_api_key`) とレート制限ミドルウェア (`rate_limit_middleware`) がレイヤーとして適用
- 全GPTルートは `/gpt/*` プレフィックス配下にネスト

**設計への影響**:
- `/gpt/preferences` (GET/POST) を同じルーターに追加
- 既存のミドルウェアスタック（認証 + レート制限）がそのまま適用される
- OpenAPIスキーマに2つの新operationId (`getPreferences`, `setPreferences`) を追加

### トピック 5: マッチスコア算出アルゴリズム

**調査内容**: ルール/タグベースのスコアリング設計

**発見事項**:
- product.md の非ゴールに「高度な推薦モデル」が明記 — ルール/タグベースから開始
- 現在の検索は boolean フィルタのみ（マッチ or 非マッチ）

**設計への影響**:
- `relevance_score` は 0.0〜1.0 の範囲で算出
- 3つのサブスコアの加重平均: budget_score (0.3), intent_score (0.4), preference_score (0.3)
- budget_score: `1.0 - (subsidy_per_call_cents / max_budget_cents)` (予算内なら高スコア)
- intent_score: マッチしたフィールド数 / 検索対象フィールド数
- preference_score: preferred タスクなら 1.0, neutral なら 0.5, avoided なら 0.0（除外済みなので通常到達しない）

---

## アーキテクチャパターン評価

| パターン | 評価 | 理由 |
|---|---|---|
| インメモリフィルタリング拡張 | ✅ 採用 | 既存パターンとの一貫性、データ量が少ない段階では十分 |
| DBレベルフィルタリング | ❌ 不採用 | 複合フィルタのSQL化は複雑、データ量が少ない段階ではオーバーエンジニアリング |
| 別エンドポイント (`/gpt/smart-search`) | ❌ 不採用 | 既存エンドポイントの後方互換拡張の方がシンプル |
| ユーザー嗜好のJSONBカラム | ❌ 不採用 | 正規化テーブルの方がクエリ・更新が容易 |

---

## リスクと緩和策

| リスク | 影響 | 緩和策 |
|---|---|---|
| インメモリフィルタリングのパフォーマンス劣化 | 低（現段階） | データ量増加時にDBレベルフィルタリングに移行 |
| タグの標準化不足 | 中 | 初期タグセットを定義し、自由入力を制限 |
| 嗜好データの肥大化 | 低 | タスクタイプは有限集合、ユーザーあたり最大10-20レコード |
| 後方互換性の破壊 | 高 | 全新パラメータをOption型で追加、デフォルト動作を既存と同一に |
