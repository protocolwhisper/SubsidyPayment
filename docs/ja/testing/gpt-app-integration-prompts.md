# GPT App 結合テスト手順（自然な日本語プロンプト版）

この手順書は、**誰が見ても迷わず最後まで進める**ことを目的にしています。  
各ステップの「入力例」をそのまま貼り付ければ実行できます。

---

## 0. このテストで確認すること

次の7ステップが通るかを確認します。

1. キャンペーン作成
2. サービス検索
3. サービス選択
4. タスク詳細取得
5. タスク完了
6. 実行可否確認
7. サービス実行

---

## 1. 事前チェック

- GPT App で SubsidyPayment MCP が接続済み
- チャットで MCP ツールが実行できる状態
- 迷ったら `get_prompt_guide_flow` を使う

> 補足: `search_services` が `Invalid API key` で失敗する場合は、バックエンド設定を確認してください（通常は `GPT_API_KEY_ENFORCEMENT=false`）。

---

## 2. 進め方のルール

- 各ステップで返る `structuredContent.next_actions` を最優先で使う
- 不明点が出たら `get_prompt_guide_flow` を呼ぶ
- 推測せず、ツール出力だけで判断する

---

## 3. 7ステップ実行（自然な日本語プロンプト）

## Step 0: まずガイドを出す

### 入力例（コピペ可）

```text
最初の案内をお願いします。get_prompt_guide_flow を context_step=0, service=github で実行してください。
```

### 成功条件

- `flow_step` が返る
- `recommended_next_prompt` が返る
- `next_actions` が1件以上ある

---

## Step 1: キャンペーンを作る

### 入力例（コピペ可）

```text
キャンペーンを新規作成したいです。create_campaign_from_goal を
purpose=GitHub Issue作成導線の検証,
sponsor=SubsidyPayment Demo,
target_roles=["developer"],
target_tools=["github"],
required_task=product_feedback,
subsidy_per_call_cents=5000,
budget_cents=50000
で実行してください。
```

### 成功条件

- `campaign_id` が返る
- `service_key=github` のキャンペーンが作成される
- `next_actions` が返る

### うまくいかないとき

```text
キャンペーン作成で詰まったので、次の操作を案内してください。get_prompt_guide_flow を context_step=1, service=github で実行してください。
```

---

## Step 2: サービスを探す

### 入力例（コピペ可）

```text
GitHubのIssue作成に使える候補を探したいです。search_services を q=github, intent=GitHub Issueを作成したい, max_budget_cents=50000, campaign_id=<campaign_id> で実行してください。
```

### 成功条件

- 候補サービス一覧が返る
- `next_actions` が返る

### うまくいかないとき

```text
次に何をすればよいか案内してください。get_prompt_guide_flow を context_step=2, service=github, campaign_id=<campaign_id> で実行してください。
```

---

## Step 3: 選んだサービスのタスク一覧を見る

> Step2 の結果から `service_key` を1つ選んで使います（例: `github`）

### 入力例（コピペ可）

```text
このサービスで必要なタスクを確認したいです。get_service_tasks を service_key=github で実行してください。
```

### 成功条件

- `tasks` が返る
- `campaign_id` が確認できる

### うまくいかないとき

```text
この段階の正しい進め方を教えてください。get_prompt_guide_flow を context_step=3, service=github で実行してください。
```

---

## Step 4: タスク詳細を取得する

> Step3 で確認した `campaign_id` を使います

### 入力例（コピペ可）

```text
タスクの入力要件を詳しく確認したいです。get_task_details を campaign_id=<campaign_id> で実行してください。
```

### 成功条件

- `required_task` / `task_input_format` が返る
- `next_actions` が返る

### うまくいかないとき

```text
このステップで次にやることを教えてください。get_prompt_guide_flow を context_step=4, campaign_id=<campaign_id>, service=github で実行してください。
```

---

## Step 5: タスクを完了する

### 入力例（コピペ可）

```text
タスク完了を登録したいです。complete_task を次の内容で実行してください。
- campaign_id: <campaign_id>
- task_name: product_feedback
- details: {"github_username":"octocat","github_repo":"octo-org/subsidy-payment","issue_title":"[Bug] OAuth callback fails on Safari","issue_body":"Repro: 1) Login 2) Redirect loop 3) 401 on callback. Expected: successful callback."}
- consent: {"data_sharing_agreed":true,"purpose_acknowledged":true,"contact_permission":false}
```

### 成功条件

- `can_use_service=true` または完了成功メッセージ
- `task_completion_id` が返る

### うまくいかないとき

```text
タスク完了に進むための案内をください。get_prompt_guide_flow を context_step=5, campaign_id=<campaign_id>, service=github で実行してください。
```

---

## Step 6: 実行できる状態か確認する

### 入力例（コピペ可）

```text
今の状態を確認したいです。get_user_status を実行してください。
```

### 成功条件

- `available_services` に対象サービスがある
- `next_actions` が返る

### うまくいかないとき

```text
次に何を実行すべきか教えてください。get_prompt_guide_flow を context_step=6, service=github で実行してください。
```

---

## Step 7: 実際にサービスを実行する

### 入力例（コピペ可）

```text
実際にGitHub Issue作成を試したいです。run_service を service=github, input=Create a GitHub issue in octo-org/subsidy-payment with title "[Bug] OAuth callback fails on Safari" and include reproduction steps. で実行してください。
```

### 成功条件

- `payment_mode` が返る
- `sponsored_by` が返る（スポンサー利用時）
- `output` に実行結果が入る

### うまくいかないとき

```text
実行フェーズの進め方を案内してください。get_prompt_guide_flow を context_step=7, service=github で実行してください。
```

---

## 4. run_service が詰まったときの2分切り分け

次の4点を確認します。

1. `run_service` の `service` 値（例: `github`）
2. `campaign.target_tools` に同じキーがあるか
3. `sponsored_apis.service_key` に同じキーがあるか
4. `run_service` の `output` 生値

### 切り分け入力例（コピペ可）

```text
run_service がうまく通らないので切り分けをお願いします。
次の順で報告してください。
1) 実行した service 値
2) 一致している campaign.target_tools
3) 一致している sponsored_apis.service_key
4) run_service の output 生値（省略なし）
5) 次に修正すべき候補を1つ
```

---

## 5. 最終報告テンプレ

### 入力例（コピペ可）

```text
7ステップの結果を次の形式でまとめてください。
- Step1 create_campaign_from_goal: pass/fail（理由1行）
- Step2 search_services: pass/fail（理由1行）
- Step3 get_service_tasks: pass/fail（理由1行）
- Step4 get_task_details: pass/fail（理由1行）
- Step5 complete_task: pass/fail（理由1行）
- Step6 get_user_status: pass/fail（理由1行）
- Step7 run_service: pass/fail（理由1行）
最後に未解決課題があれば最大3件で示してください。
```
