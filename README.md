# SubsidyPayment

# Payload Exchange Extended (Campaign + Sponsor Subsidy Layer for x402)

目的: 
x402 の 402 Paywall を Proxy で受け止め、スポンサーが支払いを肩代わりする代わりに、ユーザーのタスク実行やデータ提供を受け取れるようにします。
さらに、スポンサーは「キャンペーン」を作って配布し、ユーザーは主要 AI / 開発 UI 経由でも自然にこの仕組みを使えるようにします。

---

## Product Scope

### 何を実現するか
- x402 で保護されたリソース(API/データ/デジタルコンテンツ等)へのアクセスに対して、次の支払い方法を提供する
  - Sponsor が支払いを全額または一部負担する(代わりにユーザーにタスクやデータ提供を依頼)
  - ユーザーが直接支払う(フォールバック)
- Sponsor は「どのユーザーに」「何をしてほしいか」「いくら負担するか」をキャンペーンとして発行できる
- User は「サービス名」または「やりたい機能・agentに使わせたい機能」から、スポンサー付きの x402 対応サービスを選んで実利用できる
- User はプロフィールと survey 回答を保存して再利用し、毎回同じ入力を繰り返さなくていいようにする。
- Agent 実行中にクレジットが尽きたら通知でタスク実行に誘導できる

---

## Core Concepts

- Resource: x402 で保護された上流の 有料endpoint 
- Proxy: 402 を intercept し、Paywall とタスク導線を出す
- Sponsor: 支払いを肩代わりする主体(企業、将来的には agent も含む)
- Campaign: Sponsor が作る募集単位(対象、目的、予算、タスク、自社サービスへの登録、データ要求、同意条件)
- Offer: Resource 単位のスポンサー条件(割引率、上限、必要タスク、収集データ)
- Action Plugin: タスクやデータ収集を追加できるプラグイン拡張点
- Consent Vault: 明示同意、利用目的、保持期間、連絡可否を管理する層

---

## Requirements (Prioritized)

### P0: まず動くこと(Upstream互換の土台を壊さない)
- x402 Proxy
  - 上流 x402 resource に対するリクエストを proxy し、402 を受けたら Paywall を表示できる
  - sponsor 支払いが成立したら、上流に支払いを行い、resource 応答をユーザーに返す
- Paywall UI
  - 現在有効な sponsor 条件(ある場合)を表示し、タスク選択と実行ができる
  - sponsor がいることをユーザーに明示できる
- Action Plugin System
  - 既存の action を維持しつつ、追加タスクを後から増やせる
  - タスク実行の開始/検証/完了が一貫したインタフェースで扱える
- Resource Discovery
  - 利用可能な x402 resource を検索/閲覧できる
- Direct Payment (Fallback)
  - sponsor がいない、または同意しない場合も、ユーザーが直接支払える導線を残す
- ChatGPT 連携(MCP + Widget)
  - MCP server を提供し、Paywall / Resource のウィジェット表示ができる
  - sponsor がいない、または同意しない場合も、ユーザーが直接支払える導線を残す
- Claudecode/OpenClaw 連携(MCP + Widget)
  - Skills を提供し、Paywall / Resource のウィジェット表示ができる
- Deployment
  - Vercel デプロイを第一級に扱い、iframe でのアセット読み込みが壊れない
  - 必須 env が README に明記され、デプロイで再現できる

完了条件(最低ライン)
- ローカルと本番で、402 -> Paywall -> Action -> Sponsor支払い -> Resource返却 の一連が通る
- sponsor 表示、同意表示が UI で確認できる

---

### P1: 追加要件の中核(プロダクトとして価値が出る部分)
#### ToB: Sponsor Campaign Builder (Chat AI UI)
- Sponsor が自然言語の 1問1答でキャンペーンを作成できる
  - ターゲット属性の入力から、推奨 x402 対応サービスが推薦順に出る
  - 目的入力から、推奨タスクセット、推奨負担額、割引率/上限が提案される
  - 作成者は提案を選ぶだけで publish できる
- Sponsor Dashboard
  - キャンペーン一覧(状態、消化、完了数)
  - Data Inbox(受領データの件数、内容確認、export)

#### ToC: “実際に使える” 入口を用意
- User がサービス名検索で、スポンサー付きかどうかが分かる
- User が「機能」検索で、スポンサー付き tool を選べる(例: scraping, design, storage)
- Profile Vault
  - Email, region, IP type, 利用中サービスなどの基本情報を保存できる
  - survey 回答を保存し、タスク実行時に再利用できる
- Consent / Compliance
  - 明示同意(チェック)なしにデータがスポンサーへ渡らない
  - 利用目的、保持期間、スポンサー連絡可否を必ず表示する
- Notification
  - agent 稼働中などでクレジットが尽きた際、通知でタスク導線へ遷移できる

完了条件(最低ライン)
- Sponsor がキャンペーンを作って公開し、User 側検索/Paywall に露出する
- Sponsor が Data Inbox で結果を確認できる
- User がプロフィール保存と再利用で入力を短縮できる
- 通知から Paywall に飛べる

---

### P2: スケールのための要件(後から効いてくる)
- Recommendation Engine の高度化
  - ルール/タグ中心から、成果データを使った重み更新へ
  - さらに embedding や協調フィルタリングは段階導入
- 不正/低品質対策の強化
  - 人間認証の強化(段階的)
  - タスク proof の強化(外部連携、webhook、再検証)
- 多クライアント統合の本格化
  - ChatGPT 以外(Claude, Codex, OpenClaw等)向けに、共通の HTTP API と SDK を整備
  - UI 埋め込み不可の環境でも “リンク型 paywall” で完結できる
- 分析と監査
  - ファネル(閲覧/開始/完了/支払い)の計測
  - Sponsor の閲覧/エクスポート監査ログ
  - データ redaction と最小化の運用

---

## Data Collection Framework (Action Pluginとして提供)

### Basic personal data
- Email
- Region
- IP type
- SponsorページへのSignup要望(任意)
- botでないことを保証する最低限の人間認証

### Survey data
- Demographics
- Goals / KPIs
- Organization size
- Prompt(必要性がある場合のみ)
- Agents / skills used usually
- Media consumption
- Competitor usage
- Satisfaction of current services
- Price sensitivity
- Switching triggers
- Alternative comparisons

---

## Compliance (Must)
- Explicit user consent (項目単位、スポンサー単位で管理できる)
- Usage purpose disclosure
- Data retention disclosure
- Sponsor contact disclosure (ユーザーが可否を選べる)

---

## Non-Goals (初期ではやらない)
- 完璧な KYC や重い本人確認
- 高度な推薦モデルを最初から(まずはルール/タグで開始)
- 全クライアントへのネイティブUI統合を最初から(まずはMCP + HTTPで吸収)
- 重いデータ基盤連携(まずは export と監査ログ)

---

## Suggested Milestones
- M0: Upstream互換で end-to-end が通る(P0)
- M1: サービス検索とスポンサー表示、実利用ルート(P1のToC前半)
- M2: Campaign Builder Chat と publish、Data Inbox(P1のToB)
- M3: Profile Vault + Consent 完成(P1の運用要件)
- M4: 通知と多クライアント向け API/SDK(P1後半)
- M5: 推薦高度化と不正対策(P2)

---
