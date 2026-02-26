# MCP UX/認証/誘導フロー 改善分析レポート（2026-02-26）

## 1. 目的

本レポートは、SnapFuel の MCP サーバーを GPT App 一般公開する前提で、以下の課題に対する現状分析・根本原因・解決方針を整理する。

- メール認証必須が UX 障壁になっている
- Embedded UI のボタン操作後に「何をすべきか」が伝わらない
- 処理完了後の次アクション提示が弱く、ユーザーが迷う
- `run_service` 実行時に GitHub 実行フローへ到達しない
- インスタンス description / ツール description が短く、誘導力が不足
- ガイド専用ツールがなく、ユーザー迷子時に復帰導線がない

---

## 2. 調査スコープと確認日

- 調査日: 2026-02-26
- 対象:
  - `mcp-server/src/**`
  - `mcp-server/src/widgets/src/**`
  - `src/gpt.rs`（Rust バックエンドの `gpt_run_service`）
  - 既存テスト (`mcp-server/tests/**`)

---

## 3. 現状診断（コード根拠）

### 3.1 認証が必須化しやすい構造

- `AUTH_ENABLED` 未指定時、`AUTH0_DOMAIN` と `AUTH0_AUDIENCE` が両方存在すると OAuth が自動有効化される。
  - 根拠: `mcp-server/src/config.ts` の `resolveAuthEnabled`
- `run_service` / `get_task_details` / `complete_task` / `get_user_status` などは、`authEnabled=true` の場合に `TokenVerifier.verify()` 失敗で `mcp/www_authenticate` を返す。
  - 根拠: 各ツールの `resolveBearerToken` + `verify` 分岐
- noauth セッション自動生成 (`resolveOrCreateNoAuthSessionToken`) は `authEnabled=true` だと停止する。
  - 根拠: `mcp-server/src/tools/session-manager.ts` の `if (config.authEnabled) return null;`

### 3.2 インスタンス説明・ツール説明の情報不足

- MCP サーバー初期化時に `instructions` が未設定。
  - 根拠: `mcp-server/src/server.ts`
- アプリ説明は存在するが短く、迷子復帰フローが明記されていない。
  - 根拠: `mcp-server/app-metadata.json`
- 各ツールの `description` は機能名説明中心で、次アクション誘導が弱い。
  - 根拠: `mcp-server/src/tools/*.ts`

### 3.3 ウィジェット操作後の誘導不足

- `services-list.html` などで `app.callTool` は実装されているが、失敗時・無反応時の案内が弱い。
- 「チャットに入力してください（ask in chat to do tasks）」の全画面共通バナーは未実装。
  - 根拠: `mcp-server/src/widgets/src/*.html`

### 3.4 `run_service` と GitHub 実行の経路不一致

- MCP ツール `create_github_issue` は独立ツールとして実装されており、`run_service` から自動で呼ばれない。
  - 根拠: `mcp-server/src/tools/github-issue.ts`
- Rust 側 `gpt_run_service` は、スポンサー一致後に `sponsored_apis.service_key == service` が存在した場合のみ upstream API を呼ぶ。
  - 根拠: `src/gpt.rs` の `maybe_sponsored_api` lookup
- つまり「`run_service` 実行 = GitHub API 呼び出し」ではなく、`service` キー整合と `sponsored_apis` 設定が必要。

---

## 4. 主要課題の根本原因

### 課題A: メール認証必須で到達不能

- 本質: 「認証ON環境ではゲスト導線が実質閉じる」設計。
- 影響: 検証ユーザーが初手で離脱しやすい。

### 課題B: ボタン押下後に無反応に見える

- 本質: ウィジェットが内部でツール呼び出ししても、画面遷移保証がない環境では「反応なし」に見える。
- 影響: ユーザーが詰まる。

### 課題C: 次に何をすべきか不明

- 本質: 各ツール出力に「固定フォーマットの次アクション」がない。
- 影響: 会話ごとに案内品質が揺れる。

### 課題D: `run_service` が GitHub 実行へ到達しない

- 本質: MCP ツール間自動連携不足ではなく、実行経路設計（サービスキー/上流紐付け/導線定義）の不足。

---

## 5. 技術質問への回答

### Q1. MCPツールから他ツールを呼び出せるか？

可能。ただし実装方式は2種類ある。

1. **推奨**: MCPツール内で共通サービス関数・クライアントを直接呼ぶ（同一プロセス内部連携）
2. MCPクライアント経由で別ツールを呼ぶ（複雑で障害点が増えるため通常は非推奨）

今回の問題は 2 が必要というより、`run_service` の中で必要な実行関数を直接呼ぶ設計に統一されていない点が主因。

### Q2. description を拡充すれば解決するか？

**部分的には有効だが、単独では不十分。**  
description 拡充は「モデル誘導」には効くが、実際の経路不整合（service_key 不一致など）はコード修正が必要。

---

## 6. 改善方針（提案）

## 6.1 認証UX改善（最優先）

- 方針: 「探索系はゲスト許可、実行系は段階的認証」
- 実装案:
  - `GUEST_MODE_ENABLED=true`（新規）
  - `search_services`, `get_service_tasks`, `get_task_details`, `get_prompt_guide_flow` は noauth 実行可
  - 実行確定直前 (`complete_task`, `run_service`) で必要条件を判定

## 6.2 全画面共通チャット誘導

- 全ウィジェットに固定バナーを追加:
  - 表示文: `チャットに入力してください（ask in chat to do tasks）`
  - クリックで `sendFollowUpMessage` 実行（推奨プロンプトを投入）

## 6.3 次アクションの強制出力

- 全ツールの `structuredContent` に共通項目追加:
  - `flow_step`
  - `can_proceed`
  - `next_actions[]`（最大3件、各項目に「次に打つべきプロンプト」必須）
- これにより処理後の誘導を常に明示化する。

## 6.4 プロンプトガイド専用ツールの追加

- 新規ツール: `get_prompt_guide_flow`
- 役割:
  - 利用可能ツール一覧
  - 現在ステップの推奨1手
  - コピペ用プロンプト
  - 逸脱防止（不要な別案を出さない）

## 6.5 `run_service` と GitHub 実行の整合

- 推奨方式:
  - `run_service` が `service=github` 系を受けた場合、共通実行層で GitHub Issue 実行まで責務を持つ
  - もしくは `sponsored_apis.service_key` / `campaign.target_tools` / MCP引数 `service` の命名を厳密統一
- 出力強化:
  - `issue_url`, `repo`, `issue_number` 等を返し、成功判定を可視化

---

## 7. 期待フロー（公開目標）への適合性評価

目標フロー 0-5 に対して現状は以下のギャップがある。

- 0: 初回に必ずガイドを出す仕様がない
- 1-3: ユーザー回答テンプレートの強制提示が弱い
- 4: タスク完了後の次プロンプト提示が不定
- 5: `run_service` から GitHub実行表示までの一貫性が不足

提案した 6.1-6.5 を適用すれば、ガイド固定・誘導固定・実行一貫性の3点が揃い、目標フローへ収束可能。

---

## 8. 既存テストとの整合

現行テストは OAuth 必須を前提にした静的アサーションを含むため、改善時は更新が必要。

- `mcp-server/tests/task-5.2-authenticate-user.test.mjs`
- `mcp-server/tests/task-5.3-task-tools.test.mjs`
- `mcp-server/tests/task-5.4-run-service.test.mjs`
- `mcp-server/tests/task-6.3-oauth-enforcement.test.mjs`

実行確認（2026-02-26）:
- 上記関連テストは現行コードで pass 済み（仕様が現在の OAuth 前提に固定されていることを確認）。

---

## 9. リスクと対策

- リスク: noauth 許可拡大で不正利用が増える
  - 対策: 実行系でレート制限・step gating・監査ログ強化
- リスク: 誘導文が長すぎて逆に読まれない
  - 対策: 常に「次に打つ1文」を最上段固定
- リスク: ツール説明拡充だけで満足して経路バグが残る
  - 対策: `run_service -> github` の E2E テストを追加し、成功条件をCIに固定

---

## 10. 結論

今回の不具合群は「説明不足」だけでなく、「認証境界設計」と「実行経路整合」の問題が中心である。  
解決には以下をセットで実施する必要がある。

1. ゲスト導線を許可する認証ポリシー再設計
2. 全画面共通のチャット入力誘導UI
3. 全ツール出力に次アクション候補を強制付与
4. `get_prompt_guide_flow` の新設
5. `run_service` と GitHub 実行経路のコード整合（description拡充を含む）

これらを適用することで、GPT App 公開時に「迷わず最後まで辿り着ける」UXに近づける。

