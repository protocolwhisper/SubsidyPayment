# Postgres スキーマ（現行）

このページは 2026-02-25 時点の SQLx マイグレーション状態に基づいています。

## マイグレーション基準

- マイグレーションディレクトリ: `migrations/`
- 最新マイグレーション: `0014_zkpassport_verifications.sql`
- バックエンド起動時に自動適用（`sqlx::migrate!("./migrations")`）

## 手動でマイグレーションを実行する場合

```bash
sqlx migrate info
sqlx migrate run
```

必要な環境変数:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:55432/payloadexchange
```

## 全データを削除する（運用スクリプト）

`public` スキーマ内の全テーブルデータを削除し、シーケンスをリセットします。
テーブル定義は維持されます（`_sqlx_migrations` は保持）。

```bash
# 確認あり（推奨）
./scripts/clear-db-data.sh

# 確認なし（CI/自動処理向け）
./scripts/clear-db-data.sh --yes

# 接続先を明示して実行
DATABASE_URL=postgres://postgres:postgres@localhost:55432/payloadexchange ./scripts/clear-db-data.sh --yes
```

注意:
- この操作は取り消せません。実行前に対象DBを必ず確認してください。
- `psql` コマンドが必要です。

## 現在のテーブル一覧

| テーブル | 追加マイグレーション | 用途 |
|---|---|---|
| `users` | `0001` | エンドユーザー / GPT ユーザー |
| `sponsored_apis` | `0001` | スポンサー API 定義 |
| `sponsored_api_calls` | `0001` | スポンサー API 呼び出し履歴 |
| `campaigns` | `0002` | スポンサーキャンペーン本体 |
| `task_completions` | `0003` | キャンペーンタスク完了履歴 |
| `payments` | `0004` | 決済結果記録 |
| `creator_events` | `0005` | クリエイター側イベント計測 |
| `consents` | `0007` | キャンペーン/タスク同意履歴 |
| `gpt_sessions` | `0009` | GPT セッショントークン |
| `user_task_preferences` | `0011` | タスク嗜好設定 |
| `gpt_service_runs` | `0013` | GPT サービス実行履歴 |
| `zkpassport_verifications` | `0014` | zkPassport 検証ライフサイクル |

## 既存テーブルへの追加変更（新規テーブル以外）

- `0006`: `campaigns.sponsor_wallet_address` を追加
- `0008`: `users.source`（デフォルト: `web`）を追加
- `0010`: `campaigns.task_schema`（`jsonb`）を追加
- `0012`: `campaigns.tags`（`text[]`）を追加

## 注意点

- `profiles` テーブルは存在せず、プロフィール情報は `users` に保存されます。
- キャンペーンタグは `campaigns` の配列カラムで管理され、`campaign_tags` テーブルはありません。
- スキーマ変更時は `src/types.rs`、API ハンドラ、テストを同時に更新してください。
