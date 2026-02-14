# Smart Service Suggestion — 実装タスク

## タスク 1: DBマイグレーション — ユーザータスク嗜好テーブル

**対応要件**: 3.1, 3.2, 3.5
**対応コンポーネント**: C3, C5
**概要**: ユーザーのタスク嗜好を永続化するための `user_task_preferences` テーブルを作成する。

- [x] 1.1: `migrations/0011_user_task_preferences.sql` を作成する。`user_task_preferences` テーブルに `id` (UUID PK), `user_id` (FK → users), `task_type` (TEXT NOT NULL), `level` (TEXT NOT NULL, CHECK IN preferred/neutral/avoided), `created_at`, `updated_at` カラムと `UNIQUE(user_id, task_type)` 制約、`user_id` インデックスを定義する。
- [x] 1.2: マイグレーション 0011 のスキーマ検証テストを `src/test.rs` に追加する。テーブル名、カラム、制約、インデックスの存在を検証する。

## タスク 2: DBマイグレーション — キャンペーンタグカラム

**対応要件**: 5.1, 5.4, 9.2
**対応コンポーネント**: C4, C5
**概要**: キャンペーンテーブルにタグ配列カラムを追加する。

- [x] 2.1: `migrations/0012_campaign_tags.sql` を作成する。`ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}'` を定義する。
- [x] 2.2: マイグレーション 0012 のスキーマ検証テストを `src/test.rs` に追加する。ALTER TABLE、カラム名、型、デフォルト値を検証する。

## タスク 3: 型定義の拡張と新規型追加 (P)

**対応要件**: 3.2, 6.1, 6.3, 6.5, 8.1, 8.2
**対応コンポーネント**: C6
**概要**: `src/types.rs` に嗜好管理型と拡張検索型を追加する。

- [x] 3.1: `GptSearchParams` に `max_budget_cents: Option<u64>`, `intent: Option<String>`, `session_token: Option<Uuid>` フィールドを追加する。
- [x] 3.2: `GptSearchResponse` に `applied_filters: Option<AppliedFilters>`, `available_categories: Option<Vec<String>>` フィールドを `#[serde(skip_serializing_if = "Option::is_none")]` 付きで追加する。
- [x] 3.3: `GptServiceItem` に `tags: Vec<String>`, `relevance_score: Option<f64>` フィールドを追加する。`relevance_score` は `#[serde(skip_serializing_if = "Option::is_none")]` を付与する。
- [x] 3.4: 新規型 `AppliedFilters`, `TaskPreference`, `GptPreferencesParams`, `GptSetPreferencesRequest`, `GptPreferencesResponse`, `GptSetPreferencesResponse` を `src/types.rs` に追加する。
- [x] 3.5: 全新規型の構築テストを `src/test.rs` に追加する。各型がインスタンス化可能であることを検証する。

## タスク 4: 嗜好管理ハンドラ

**対応要件**: 3.1, 3.3, 3.4, 3.5, 8.1, 8.2, 8.3, 8.4
**対応コンポーネント**: C2
**概要**: ユーザーのタスク嗜好を取得・登録・更新するハンドラを `src/gpt.rs` に実装する。

- [x] 4.1: `gpt_get_preferences` ハンドラを実装する。`session_token` からユーザーを識別し、`user_task_preferences` テーブルから嗜好を取得して返却する。嗜好未登録時は空配列と案内メッセージを返す。
- [x] 4.2: `gpt_set_preferences` ハンドラを実装する。`session_token` からユーザーを識別し、既存嗜好を全削除後、新しい嗜好を一括挿入する。更新日時を記録し、確認レスポンスを返す。
- [x] 4.3: 両ハンドラの署名テストを `src/test.rs` に追加する。
- [x] 4.4: 嗜好登録・取得・更新の統合テストを `src/test.rs` に追加する（DATABASE_URL 必要）。嗜好の永続化、上書き動作、空嗜好の返却を検証する。

## タスク 5: 拡張検索ハンドラ — 予算フィルタ

**対応要件**: 1.1, 1.2, 1.3, 1.4
**対応コンポーネント**: C1
**概要**: `gpt_search_services` に予算ベースのフィルタリングとソートを追加する。

- [x] 5.1: `gpt_search_services` に `max_budget_cents` フィルタを追加する。指定時は `subsidy_amount_cents <= max_budget_cents` のサービスのみ残す。未指定時は既存動作を維持する。
- [x] 5.2: 予算フィルタ後に0件の場合、予算緩和や直接支払いを案内するメッセージを生成する。
- [x] 5.3: 予算フィルタの統合テストを `src/test.rs` に追加する。フィルタ適用、未指定時の全件返却、0件メッセージを検証する。

## タスク 6: 拡張検索ハンドラ — 意図フィルタとタグマッチング

**対応要件**: 2.1, 2.2, 2.3, 2.4, 5.2, 5.3
**対応コンポーネント**: C1, C4
**概要**: `gpt_search_services` に意図ベースのキーワードマッチングとタグ活用を追加する。

- [x] 6.1: `CampaignRow` の SELECT クエリに `tags` カラムを追加し、`GptServiceItem` の `tags` フィールドにマッピングする。タグ未設定時は `target_tools` + `required_task` からデフォルトタグを推定する `infer_tags` ロジックを実装する。
- [x] 6.2: `matches_intent` 関数を実装する。`intent` パラメータをスペース区切りでキーワード分割し、`name`, `required_task`, `category`, `tags` に対して case-insensitive 部分一致検索を行う。
- [x] 6.3: 意図検索結果が0件の場合、全アクティブサービスのカテゴリ一覧を `available_categories` として返却するロジックを追加する。
- [x] 6.4: 意図フィルタとタグマッチングの統合テストを `src/test.rs` に追加する。マッチ、非マッチ、0件時のカテゴリ返却を検証する。

## タスク 7: 拡張検索ハンドラ — 嗜好フィルタとスコアリング

**対応要件**: 4.1, 4.2, 4.3, 4.4, 4.5, 6.2, 6.3, 6.5
**対応コンポーネント**: C1
**概要**: `gpt_search_services` にタスク嗜好ベースのフィルタリング、スコアリング、`AppliedFilters` 構築を追加する。

- [x] 7.1: `session_token` が指定された場合、`resolve_session` でユーザーを識別し、`user_task_preferences` から嗜好をロードする。`avoided` タスクタイプのサービスを除外し、`preferred` タスクタイプのサービスにスコアブーストを適用する。
- [x] 7.2: `calculate_score` 関数を実装する。`budget_score` (重み 0.3), `intent_score` (重み 0.4), `preference_score` (重み 0.3) の加重平均で `relevance_score` を算出する。スコア降順でサービスをソートする。
- [x] 7.3: `AppliedFilters` オブジェクトを構築し、レスポンスの `applied_filters` フィールドに設定する。`preferences_applied` フラグを嗜好適用有無に基づいて設定する。
- [x] 7.4: 嗜好フィルタ全除外時のメッセージ生成、複合フィルタ AND 条件、拡張パラメータなしの後方互換性を検証する統合テストを `src/test.rs` に追加する。

## タスク 8: ルーター統合とメトリクス

**対応要件**: 8.4, 8.5, 9.3, 9.4
**対応コンポーネント**: C5（ルーター）
**概要**: 嗜好管理エンドポイントをGPTサブルーターに統合し、メトリクスを記録する。

- [x] 8.1: `src/main.rs` の `build_gpt_router` に `/preferences` の GET/POST ルートを追加する。既存の認証ミドルウェアとレート制限ミドルウェアの配下で動作することを確認する。
- [x] 8.2: ルーター統合テストを `src/test.rs` に追加する。`/gpt/preferences` の到達性、認証ミドルウェア適用、既存ルート非破壊、`gpt_get_preferences` / `gpt_set_preferences` メトリクスラベルの記録を検証する。

## タスク 9: OpenAPIスキーマ拡張

**対応要件**: 8.5, 9.3
**対応コンポーネント**: C8
**概要**: `openapi.yaml` に拡張パラメータと新規エンドポイントを追加する。

- [x] 9.1: `GET /gpt/services` のパラメータに `max_budget_cents`, `intent`, `session_token` を追加する。レスポンススキーマに `applied_filters`, `available_categories` を追加する。`GptServiceItem` に `tags`, `relevance_score` を追加する。
- [x] 9.2: `GET /gpt/preferences` (operationId: `getPreferences`) と `POST /gpt/preferences` (operationId: `setPreferences`) のエンドポイント定義を追加する。
- [x] 9.3: OpenAPIスキーマの検証テストを `src/test.rs` に追加する。新パラメータ、新エンドポイント、operationId、エンドポイント総数 ≤ 30 を検証する。

## タスク 10: GPTシステムプロンプト拡張

**対応要件**: 7.1, 7.2, 7.3, 7.4
**対応コンポーネント**: C7
**概要**: GPT Builder 構成ドキュメントを更新し、スマートサジェスト機能の会話フローを追加する。

- [x] 10.1: `.kiro/specs/smart-service-suggestion/gpt-config-update.md` を作成する。システムプロンプトの追加セクション（意図・予算・嗜好の収集フロー）、新 Conversation Starter（「自分の好みを設定する」）、マッチスコアと嗜好適用状況の説明ガイドラインを記述する。

## タスク 11: E2Eテストと後方互換性検証

**対応要件**: 6.4, 9.1
**対応コンポーネント**: 全コンポーネント
**概要**: スマートサジェスト全フローのE2Eテストと、既存機能の後方互換性を検証する。

- [x] 11.1: E2E統合テストを `src/test.rs` に追加する。嗜好設定 → 意図+予算検索 → フィルタリング結果検証 → スコア降順確認の全フローを1テストで実行する。
- [x] 11.2: 後方互換性テストを `src/test.rs` に追加する。拡張パラメータを一切指定しない場合に、既存の `gpt-apps-integration` と完全に同一のレスポンス構造（`applied_filters: null`, `available_categories: null`, `relevance_score: null`）を返すことを検証する。

---

## 要件カバレッジマトリクス

| 要件ID | タスク |
|---|---|
| 1.1 | 5.1 |
| 1.2 | 5.1 |
| 1.3 | 5.2 |
| 1.4 | 7.2 |
| 2.1 | 6.2 |
| 2.2 | 6.2 |
| 2.3 | 6.3 |
| 2.4 | 6.1 |
| 3.1 | 1.1, 4.1, 4.2 |
| 3.2 | 1.1, 3.4 |
| 3.3 | 4.1 |
| 3.4 | 4.2 |
| 3.5 | 4.1, 4.2 |
| 4.1 | 7.1 |
| 4.2 | 7.1, 7.2 |
| 4.3 | 7.1 |
| 4.4 | 7.4 |
| 4.5 | 7.3 |
| 5.1 | 2.1, 6.1 |
| 5.2 | 3.3, 6.1 |
| 5.3 | 6.1 |
| 5.4 | 2.1 |
| 6.1 | 3.1 |
| 6.2 | 7.4 |
| 6.3 | 7.3 |
| 6.4 | 7.4, 11.2 |
| 6.5 | 7.2 |
| 7.1 | 10.1 |
| 7.2 | 10.1 |
| 7.3 | 10.1 |
| 7.4 | 10.1 |
| 8.1 | 4.2, 9.2 |
| 8.2 | 4.1, 9.2 |
| 8.3 | 4.1, 4.2 |
| 8.4 | 8.1 |
| 8.5 | 9.2 |
| 9.1 | 11.2 |
| 9.2 | 2.1 |
| 9.3 | 8.2, 9.3 |
| 9.4 | 8.2 |
