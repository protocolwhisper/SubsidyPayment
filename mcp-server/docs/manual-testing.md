# MCP UI 手動検証手順（ngrok + ChatGPT 開発者モード）

## 1. 目的

MCP サーバーの E2E フロー（サービス検索 → 認証 → タスク取得 → タスク完了 → サービス実行）と、3つのウィジェット表示を ChatGPT 開発者モード上で確認する。

## 2. 事前準備

1. `mcp-server/.env` を用意し、以下を設定する。
   - `RUST_BACKEND_URL`
   - `MCP_INTERNAL_API_KEY`
   - `AUTH0_DOMAIN`
   - `AUTH0_AUDIENCE`
   - `PUBLIC_URL`（ngrok URL）
2. Rust バックエンドを起動する。
3. MCP サーバーを起動する。
   - `cd mcp-server`
   - `npm run dev`
4. ngrok トンネルを作成する。
   - `ngrok http 3001`
5. ngrok の公開 URL を `PUBLIC_URL` に反映し、MCP サーバーを再起動する。

## 3. ChatGPT 開発者モード設定

1. ChatGPT の開発者モードで新しい App を作成する。
2. MCP サーバー URL に `https://<ngrok-id>.ngrok-free.app/mcp` を設定する。
3. OAuth 設定を有効化し、Auth0 設定（Domain/Audience/Scopes）を合わせる。
4. 接続テストで `/.well-known/oauth-protected-resource` が取得できることを確認する。

## 4. E2E 検証シナリオ

1. `search_services` を実行する。
   - 期待: `services-list` ウィジェットが表示され、カード一覧が出る。
2. サービスカードを選択する。
   - 期待: `get_task_details` が呼ばれ、`task-form` ウィジェットに遷移する。
3. OAuth ログインを完了し `authenticate_user` を実行する。
   - 期待: 認証成功し、以後のツール呼び出しが通る。
4. `task-form` で required fields と同意チェックを入力し送信する。
   - 期待: `complete_task` が成功し、完了レスポンスを得る。
5. `get_user_status` を呼び出し、`user-dashboard` を表示する。
   - 期待: ユーザー情報、完了タスク一覧、利用可能サービス一覧が表示される。
6. ready なサービスの「実行」を押す。
   - 期待: `run_service` が呼ばれ、結果が返る。

## 5. UI チェックリスト

1. `services-list`:
   - サービス名/スポンサー名/補助金額/カテゴリ/関連度が表示される。
   - カード選択で状態が保持される。
2. `task-form`:
   - required fields が動的生成される。
   - 同意チェック3種が必須として機能する。
   - `already_completed=true` の場合に完了表示になる。
3. `user-dashboard`:
   - email、完了済みタスクテーブル、ready 状態付きサービスカードが表示される。
   - 実行ボタンで `run_service` が発火する。

## 6. 失敗時の確認ポイント

1. OAuth エラー時:
   - ツールレスポンスに `_meta["mcp/www_authenticate"]` があるか。
2. バックエンド通信エラー時:
   - MCP サーバーのログに `BackendClientError` が出るか。
3. ウィジェット表示崩れ時:
   - `dist/widgets/*.html` の更新有無と `vite build` の結果を確認する。

## 7. 記録テンプレート

- 実施日:
- 実施環境:
- ngrok URL:
- 実施者:
- 成功シナリオ:
  - [ ] search_services
  - [ ] authenticate_user
  - [ ] get_task_details
  - [ ] complete_task
  - [ ] run_service
- 不具合/メモ:
