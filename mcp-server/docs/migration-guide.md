# Actions → Apps SDK 移行後の削除手順書

## 目的

ChatGPT Actions（OpenAPI）から Apps SDK（MCP）へ完全移行した後に、旧方式の設定と配信資産を安全に削除するための運用手順を定義する。

## 前提条件

- MCP サーバー（`subsidypayment-mcp`）が本番運用されている。
- ChatGPT App Directory 連携で主要フロー（検索、認証、タスク完了、サービス実行）が正常動作している。
- 既存ユーザーへの移行告知期間が終了している。

## 削除対象

1. リポジトリ内の `openapi.yaml`
2. Rust サーバーの `/.well-known/openapi.yaml` 配信エンドポイント
3. ChatGPT 側の GPT Builder で設定した Custom Actions 構成

## 削除手順

1. `openapi.yaml` をリポジトリから削除する。
2. `src/main.rs` から `/.well-known/openapi.yaml` を返すルート定義を削除する。
3. GPT Builder の対象 GPT から Actions 設定を削除し、公開状態を停止する。
4. 関連ドキュメント（README や運用手順）から Actions 前提の記述を削除する。
5. 変更を `chore:` プレフィックスのコミットとして分離し、ロールバックしやすい状態でマージする。

## 移行確認チェックリスト

- [ ] MCP ツール 8 種が本番で成功する（`search_services` / `authenticate_user` / `get_task_details` / `complete_task` / `run_service` / `get_user_status` / `get_preferences` / `set_preferences`）。
- [ ] OAuth ログイン要求とトークン検証が本番環境で安定している。
- [ ] ウィジェット 3 種（services-list / task-form / user-dashboard）が ChatGPT 上で描画・操作できる。
- [ ] Rust バックエンドの `cargo test` が全件成功している。
- [ ] `render.yaml` の MCP サービス設定でデプロイが安定している。
- [ ] 監視（ヘルスチェック、エラーログ）で重大アラートが発生していない。

## ロールバック手順

1. 直近の削除コミットを revert し、`openapi.yaml` と `/.well-known/openapi.yaml` ルートを復元する。
2. GPT Builder の旧 Actions 設定を再投入し、公開状態を再開する。
3. `.env` の旧運用値（必要なら `GPT_ACTIONS_API_KEY` 関連）を復元する。
4. `cargo test` と MCP 健全性確認（`GET /health`）を実施し、復旧完了を確認する。
5. 障害報告に「切り戻し時刻」「影響範囲」「再移行の条件」を記録する。
