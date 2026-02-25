# GPT App 結合テスト用（チャット入力テンプレート）

このページは、SubsidyPayment の結合テストを ChatGPT App 上で再現するための実運用向けプロンプト例をまとめたものです。

対象フロー（6ステップ）:
1. サービス検索&一覧取得
2. 1の中から使いたいサービス選択
3. タスク選択
4. タスク実行
5. タスク完了＆支払い完了表示
6. 最初に指定したサービスの呼び出し＆結果表示

## 前提

- GPT Actions 連携時の operationId: `searchServices`, `authenticateUser`, `getTaskDetails`, `completeTask`, `runService`, `getUserStatus`
- MCP 連携時のツール名: `search_services`, `authenticate_user`, `get_task_details`, `complete_task`, `run_service`, `get_user_status`
- 実装上、支払い確定は通常 `runService` / `run_service` 実行時に行われます。

## 使い方（チャットインターフェース）

- ChatGPT App のチャット欄に、以下の「初回メッセージ」をそのまま貼り付けて送信します。
- 以降は同じスレッドで、必要に応じて「追加入力メッセージ」を送って進行します。

## 1) 初回メッセージ（GPT Actions 版）

```text
これから SubsidyPayment の結合テストを実施します。必ず Action 呼び出しで確認し、あなたの知識で補完しないでください。
目的は次の6ステップです。各ステップで「実行したAction名」「主要入力」「主要出力」を短く報告してください。

【テスト条件】
- キーワード: github
- 予算上限: 50000 cents
- 意図: GitHub Issueを作成したい
- 同意: data_sharing_agreed=true, purpose_acknowledged=true, contact_permission=false
- タスク証跡(details): {"github_username":"octocat","github_repo":"octo-org/subsidy-payment","issue_title":"[Bug] OAuth callback fails on Safari","issue_body":"Repro: 1) Login 2) Redirect loop 3) 401 on callback. Expected: successful callback."}

【実行手順】
1) サービス検索＆一覧取得
- searchServices を使い、q/github + intent + max_budget_cents で検索
- 候補を3件まで、service_id / sponsor / subsidy_amount_cents / required_task を一覧表示

2) 1の中から使うサービス選択
- relevance_score と required_task を見て最適な1件を選定し、選定理由を1行で説明

3) タスク選択
- getTaskDetails を呼び、実行すべき task_name と required_fields を確定

4) タスク実行
- completeTask を実行（上記 details と consent を使用）
- can_use_service / consent_recorded / task_completion_id を表示

5) タスク完了＆支払い完了表示
- getUserStatus を実行し、選択サービスが実行可能状態か確認
- 「実行可能か」「スポンサー名」「次アクション(runService)」を表示

6) 最初に指定したサービスを実際に呼び出し、結果表示
- runService を実行（input: "Create a GitHub issue in octo-org/subsidy-payment with title '[Bug] OAuth callback fails on Safari' and include reproduction steps.")
- service / payment_mode / sponsored_by / message / output要約 を表示
- 最後に6ステップの成否をチェックリストで出力
```

## 2) 初回メッセージ（MCP ツール版）

```text
SubsidyPayment のMCP結合テストを6ステップで実行してください。
必ずツール呼び出し結果のみを使って報告し、推測で補完しないでください。
各ステップで「tool名」「input要約」「output要約」を示してください。

条件:
- q: github
- intent: GitHub Issueを作成したい
- max_budget_cents: 50000
- complete_task の consent:
  - data_sharing_agreed: true
  - purpose_acknowledged: true
  - contact_permission: false
- complete_task の details(JSON文字列):
  {"github_username":"octocat","github_repo":"octo-org/subsidy-payment","issue_title":"[Bug] OAuth callback fails on Safari","issue_body":"Repro: 1) Login 2) Redirect loop 3) 401 on callback. Expected: successful callback."}

手順:
1. search_services で候補を取得
2. 候補から1件選ぶ（理由も示す）
3. get_task_details で必要タスクを取得
4. complete_task でタスク完了
5. get_user_status で完了状態と実行可否を確認
6. run_service でサービスを実行し、結果（Issue作成結果を含む）を表示

最後に、6ステップの pass/fail 一覧を出してください。
```

## チェック観点

- サービス一覧にスポンサー情報・タスク要件・補助額が含まれる
- タスク完了時に同意情報が記録される
- サービス実行時に `payment_mode` と `sponsored_by` が返る
- 最終的にサービス出力が要約される

## 3) 追加入力メッセージ例（チャット欄で使用）

```text
候補1で進めてください。次のステップに進む前に、直前の tool/action の結果を1行で要約してください。
```

```text
タスク実行前に、complete_task/completeTask に渡す payload を表示してから実行してください。
```

```text
最後に pass/fail の根拠として、各ステップで返ってきたキー項目を表でまとめてください。
```
