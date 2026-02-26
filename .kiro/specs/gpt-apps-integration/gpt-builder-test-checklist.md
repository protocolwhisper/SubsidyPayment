# GPT Builder 手動テストチェックリスト

タスク 8.5 に基づく、ChatGPT UI 上での GPT Builder 手動テスト手順書。

---

## 前提条件

- [ ] バックエンドサーバーが起動済み（`PUBLIC_BASE_URL` でアクセス可能）
- [ ] `GPT_ACTIONS_API_KEY` が環境変数に設定済み
- [ ] `DATABASE_URL` が設定済みでDBにアクティブなキャンペーンが存在する
- [ ] `cargo test gpt_builder_preflight` が全てパスしている

---

## 1. GPT Builder セットアップ検証

### 1.1 OpenAPI スキーマインポート

- [ ] ChatGPT → Explore GPTs → Create → Configure タブを開く
- [ ] Actions → Create new action → Import from URL
- [ ] URL: `https://<PUBLIC_BASE_URL>/.well-known/openapi.yaml` を入力
- [ ] **検証**: 6つの Action が表示される:
  - [ ] `searchServices`
  - [ ] `authenticateUser`
  - [ ] `getTaskDetails`
  - [ ] `completeTask`
  - [ ] `runService`
  - [ ] `getUserStatus`

### 1.2 認証設定

- [ ] Authentication → API Key を選択
- [ ] Auth Type: Bearer
- [ ] API Key: `GPT_ACTIONS_API_KEY` の値を入力
- [ ] **検証**: 認証設定が保存される

### 1.3 基本設定

- [ ] Name: `SnapFuel アシスタント`
- [ ] Description: `スポンサー付き x402 サービスの検索・タスク実行・支払いを支援します`
- [ ] Instructions: `gpt-config.md` セクション2のシステムプロンプトを貼り付け
- [ ] Conversation starters: 4つ設定（`gpt-config.md` セクション3参照）
- [ ] Privacy policy: `https://<PUBLIC_BASE_URL>/privacy`

### 1.4 保存

- [ ] 公開設定: 「Only people with a link」
- [ ] Save をクリック
- [ ] **検証**: GPT が正常に保存される

---

## 2. Conversation Starter テスト

### 2.1 「利用可能なスポンサー付きサービスを探す」

- [ ] Conversation Starter をクリック
- [ ] **検証**: GPT が `searchServices` Action を呼び出す
- [ ] **検証**: サービス一覧が日本語で表示される
- [ ] **検証**: スポンサー名、必要タスク、補助金額が明示される

### 2.2 「タスクを実行してサービスを無料で使う」

- [ ] Conversation Starter をクリック
- [ ] **検証**: GPT がまず `searchServices` でサービスを検索する
- [ ] サービスを選択する
- [ ] **検証**: GPT がメールアドレスを尋ねる
- [ ] メールアドレスを入力する
- [ ] **検証**: GPT が `authenticateUser` を呼び出す
- [ ] **検証**: GPT が `getTaskDetails` を呼び出してタスク詳細を表示する
- [ ] タスクに必要な情報を入力する
- [ ] **検証**: GPT が同意確認（データ共有、利用目的、連絡許可）を行う
- [ ] 同意する
- [ ] **検証**: GPT が `completeTask` を呼び出す
- [ ] **検証**: GPT が `runService` を呼び出してサービスを実行する
- [ ] **検証**: サービス実行結果が表示される

### 2.3 「自分のアカウント状態を確認する」

- [ ] Conversation Starter をクリック
- [ ] **検証**: GPT がメールアドレスを尋ねる（未認証の場合）
- [ ] メールアドレスを入力する
- [ ] **検証**: GPT が `authenticateUser` → `getUserStatus` を呼び出す
- [ ] **検証**: 完了済みタスクと利用可能サービスが表示される

### 2.4 「特定のカテゴリのサービスを検索する」

- [ ] Conversation Starter をクリック
- [ ] **検証**: GPT がカテゴリを尋ねる
- [ ] カテゴリ（例: `design`）を入力する
- [ ] **検証**: GPT が `searchServices` の `category` パラメータで検索する
- [ ] **検証**: フィルタされた結果が表示される

---

## 3. エラーハンドリングテスト

### 3.1 セッション期限切れ (401)

- [ ] 期限切れの session_token でリクエストを発生させる（長時間放置後に操作）
- [ ] **検証**: GPT が「セッションの有効期限が切れました」と日本語で案内する
- [ ] **検証**: GPT が `authenticateUser` を再実行して新しいトークンを取得する

### 3.2 存在しないキャンペーン (404)

- [ ] 存在しないキャンペーンIDでタスク詳細を取得しようとする
- [ ] **検証**: GPT が「指定されたキャンペーンが見つかりませんでした」と日本語で案内する

### 3.3 タスク未完了でサービス実行 (412)

- [ ] タスクを完了せずにサービス実行を試みる
- [ ] **検証**: GPT が「サービスを利用するにはタスクの完了が必要です」と日本語で案内する

---

## 4. セキュリティ検証

- [ ] **検証**: session_token がユーザーに表示されない
- [ ] **検証**: tx_hash や支払い金額の詳細がユーザーに表示されない
- [ ] **検証**: API キーがユーザーに表示されない
- [ ] **検証**: メールアドレスが不必要に繰り返し表示されない

---

## 5. セッション管理検証

- [ ] 新規ユーザーで `authenticateUser` → `is_new_user: true` が返る
- [ ] 同じメールで再度 `authenticateUser` → `is_new_user: false` が返る
- [ ] session_token が後続のリクエスト（getTaskDetails, completeTask, runService, getUserStatus）で正しく使用される

---

## テスト結果サマリ

| カテゴリ | 合格 | 不合格 | 備考 |
|---|---|---|---|
| セットアップ | | | |
| Conversation Starters | | | |
| エラーハンドリング | | | |
| セキュリティ | | | |
| セッション管理 | | | |

**テスト実施日**: ____
**テスト実施者**: ____
**総合判定**: 合格 / 不合格
