# GPT Apps Integration — 要件定義

## プロジェクト概要

GPT Apps（ChatGPT Custom GPTs + Custom Actions）に対応させ、ToC（エンドユーザー）がGPT Apps上で以下のE2Eフローを完結できるようにする：

1. **サービス選択** — スポンサー付きx402サービスの検索・選択
2. **タスク実行** — スポンサーが要求するタスク（アンケート回答、データ提供等）の実行
3. **支払い・リソース取得** — タスク完了後のスポンサー決済によるリソースアクセス

GPT Appsは ChatGPT UI 上で動作するカスタムGPTであり、Custom Actions（OpenAPI仕様に基づくAPI呼び出し）を通じてSnapFuelバックエンドと連携する。

---

## 要件一覧

### 1. GPT Actions API エンドポイント

**説明**: GPT Apps（Custom GPT）がSnapFuelバックエンドを呼び出すためのAPI群を提供する。

#### 受入基準

- **1.1**: SnapFuelシステムは、OpenAPI 3.1.0仕様に準拠したスキーマファイルを `/.well-known/openapi.yaml` で提供しなければならない（EARS: Ubiquitous）
- **1.2**: OpenAPIスキーマの各エンドポイントには、GPTが呼び出しタイミングを判断できるよう、明確な `operationId` と `description` が記述されていなければならない（EARS: Ubiquitous）
- **1.3**: GPT Actions APIのレスポンスは、GPTが自然言語で要約しやすいフラットなJSON構造でなければならない（EARS: Ubiquitous）
- **1.4**: SnapFuelシステムは、`https://chat.openai.com` からのリクエストに対してCORSヘッダーを返さなければならない（EARS: Ubiquitous）
- **1.5**: OpenAPIスキーマに定義されるエンドポイント数は30以下でなければならない（EARS: Ubiquitous）

---

### 2. サービスディスカバリ（サービス検索・選択）

**説明**: エンドユーザーがGPT上の会話を通じて、利用可能なスポンサー付きサービスを検索・選択できる。

#### 受入基準

- **2.1**: ユーザーがサービス名またはキーワードで検索を要求した場合、SnapFuelシステムはアクティブなキャンペーンおよびSponsored APIの一覧を返さなければならない（EARS: Event-driven）
- **2.2**: ユーザーが機能カテゴリ（例：「スクレイピング」「デザイン」「ストレージ」）で検索を要求した場合、SnapFuelシステムは該当するサービスをフィルタリングして返さなければならない（EARS: Event-driven）
- **2.3**: 検索結果には、各サービスのスポンサー名、必要タスク、補助金額が含まれなければならない（EARS: Ubiquitous）
- **2.4**: スポンサー付きサービスが存在しない場合、SnapFuelシステムは直接支払いオプションを提示しなければならない（EARS: State-driven）

---

### 3. ユーザー登録・識別

**説明**: GPT Apps経由のユーザーを識別し、プロフィール管理を行う。

#### 受入基準

- **3.1**: GPT Appsからの初回アクセス時、SnapFuelシステムはユーザーに基本情報（メールアドレス、リージョン）の入力を要求しなければならない（EARS: Event-driven）
- **3.2**: SnapFuelシステムは、GPT Actions用の認証方式（APIキーまたはOAuth）を提供しなければならない（EARS: Ubiquitous）
- **3.3**: ユーザーが既に登録済みの場合、SnapFuelシステムは既存プロフィールを返し、再入力を求めてはならない（EARS: State-driven）
- **3.4**: ユーザープロフィールには、GPT Apps経由であることを示すソース情報が記録されなければならない（EARS: Ubiquitous）

---

### 4. タスク実行フロー

**説明**: エンドユーザーがGPT上の会話を通じて、スポンサーが要求するタスク（アンケート回答、データ提供、サービス登録等）を実行できる。

#### 受入基準

- **4.1**: ユーザーがサービスを選択した場合、SnapFuelシステムは該当キャンペーンの必要タスク一覧を返さなければならない（EARS: Event-driven）
- **4.2**: SnapFuelシステムは、タスクの種類（アンケート、データ提供、サービス登録等）に応じた入力フォーマットをGPTに提供しなければならない（EARS: Ubiquitous）
- **4.3**: ユーザーがタスクに必要な情報を会話で提供した場合、SnapFuelシステムはタスク完了を記録しなければならない（EARS: Event-driven）
- **4.4**: タスク完了前に、SnapFuelシステムはユーザーに提供データの確認と同意を求めなければならない（EARS: Ubiquitous）
- **4.5**: ユーザーが既にタスクを完了済みの場合、SnapFuelシステムはタスクをスキップし、直接サービス利用に進めなければならない（EARS: State-driven）
- **4.6**: タスク実行中にエラーが発生した場合、SnapFuelシステムはGPTが自然言語でユーザーに説明できるエラーメッセージを返さなければならない（EARS: Unwanted behavior）

---

### 5. 支払い・リソースアクセス

**説明**: タスク完了後、スポンサー決済を通じてx402保護リソースへのアクセスを提供する。

#### 受入基準

- **5.1**: タスクが完了済みのユーザーがサービス実行を要求した場合、SnapFuelシステムはスポンサー決済を実行し、リソースレスポンスを返さなければならない（EARS: Event-driven）
- **5.2**: スポンサー決済が成功した場合、SnapFuelシステムは支払い記録（tx_hash、キャンペーンID、金額）をデータベースに保存しなければならない（EARS: Event-driven）
- **5.3**: スポンサーの予算が不足している場合、SnapFuelシステムはユーザーに直接支払いオプションまたは他のスポンサーサービスを提示しなければならない（EARS: State-driven）
- **5.4**: サービス実行結果は、GPTが自然言語で要約してユーザーに提示できる構造化されたフォーマットで返されなければならない（EARS: Ubiquitous）
- **5.5**: 支払いフロー全体（タスク確認→決済→リソース返却）は、単一のGPT Actions呼び出しで完結しなければならない（EARS: Ubiquitous）

---

### 6. 同意・コンプライアンス

**説明**: GPT Apps経由のデータ収集において、明示的な同意とプライバシー保護を確保する。

#### 受入基準

- **6.1**: SnapFuelシステムは、GPT Appsの公開要件を満たすプライバシーポリシーページを `/privacy` で提供しなければならない（EARS: Ubiquitous）
- **6.2**: ユーザーデータをスポンサーに転送する前に、SnapFuelシステムは利用目的、保持期間、連絡許可の明示的同意を取得しなければならない（EARS: Ubiquitous）
- **6.3**: ユーザーが同意を拒否した場合、SnapFuelシステムはデータ転送を行わず、直接支払いオプションを提示しなければならない（EARS: Event-driven）
- **6.4**: 同意記録は監査可能な形式でデータベースに保存されなければならない（EARS: Ubiquitous）

---

### 7. GPT構成（システムプロンプト・会話設計）

**説明**: Custom GPTのシステムプロンプト、会話フロー、Conversation Startersを設計する。

#### 受入基準

- **7.1**: GPTのシステムプロンプトには、サービス検索→タスク実行→支払い→リソース取得の会話フローが明確に定義されなければならない（EARS: Ubiquitous）
- **7.2**: GPTは、ユーザーの質問に対して自身の知識で回答せず、必ずSnapFuel Actionsを呼び出してデータを取得しなければならない（EARS: Ubiquitous）
- **7.3**: GPTには、4つ以上のConversation Starters（例：「利用可能なサービスを探す」「タスクを実行する」等）が設定されなければならない（EARS: Ubiquitous）
- **7.4**: GPTは、Actions呼び出しがエラーを返した場合、ユーザーに分かりやすい日本語で状況を説明しなければならない（EARS: Unwanted behavior）
- **7.5**: GPTは、スポンサーの存在とスポンサー条件をユーザーに明示的に伝えなければならない（EARS: Ubiquitous）

---

### 8. エラーハンドリング・信頼性

**説明**: GPT Actions経由のリクエストに対する堅牢なエラーハンドリングを提供する。

#### 受入基準

- **8.1**: SnapFuelシステムは、すべてのAPIエラーレスポンスにおいて、HTTPステータスコードと人間が読めるエラーメッセージを含むJSON構造を返さなければならない（EARS: Ubiquitous）
- **8.2**: リクエストバリデーションエラーの場合、SnapFuelシステムは400ステータスと具体的なフィールドエラーを返さなければならない（EARS: Event-driven）
- **8.3**: 認証エラーの場合、SnapFuelシステムは401または403ステータスと明確なエラーメッセージを返さなければならない（EARS: Event-driven）
- **8.4**: レート制限に達した場合、SnapFuelシステムは429ステータスと `Retry-After` ヘッダーを返さなければならない（EARS: State-driven）
- **8.5**: 内部エラーの場合、SnapFuelシステムは500ステータスを返し、詳細な内部情報を漏洩してはならない（EARS: Unwanted behavior）

---

### 9. デプロイ・運用

**説明**: GPT Apps対応のバックエンドをデプロイし、運用可能な状態にする。

#### 受入基準

- **9.1**: GPT Actions用のOpenAPIスキーマは、本番環境のHTTPS URLを `servers` フィールドに含まなければならない（EARS: Ubiquitous）
- **9.2**: GPT Apps関連の環境変数（認証キー等）は `.env.example` に文書化されなければならない（EARS: Ubiquitous）
- **9.3**: GPT Actions APIのリクエスト/レスポンスは、既存のPrometheusメトリクスに統合されなければならない（EARS: Ubiquitous）
- **9.4**: 既存のAPIエンドポイント（`/campaigns/discovery`、`/tasks/complete`、`/proxy/{service}/run` 等）との後方互換性が維持されなければならない（EARS: Ubiquitous）

---

## 非ゴール（スコープ外）

- GPT Store への公開申請プロセス（初期フェーズでは内部/リンク共有のみ）
- OAuth 2.0 による高度な認証（初期はAPIキー認証から開始）
- GPT Apps以外のクライアント（Claude、Codex等）への同時対応
- 高度なレコメンデーションエンジン（ルール/タグベースから開始）
- リアルタイム通知（GPT Apps上ではプル型のみ）

---

## 用語集

| 用語 | 説明 |
|---|---|
| **GPT Apps** | ChatGPT上で動作するCustom GPT。Custom Actionsを通じて外部APIを呼び出せる |
| **Custom Actions** | OpenAPI仕様に基づき、GPTが呼び出すAPIエンドポイント群 |
| **OpenAPI 3.1.0** | Custom Actionsの定義に使用されるAPI仕様フォーマット |
| **Conversation Starters** | GPTを開いた際に表示される定型プロンプト |
| **operationId** | OpenAPIスキーマ内の各エンドポイントの一意識別子。GPTが呼び出し判断に使用 |
| **ToC** | To Consumer。エンドユーザー側 |
| **x402** | HTTP 402ベースのペイメントプロトコル |
| **Sponsor** | ユーザーのタスク実行と引き換えに支払いを肩代わりするエンティティ |
| **Campaign** | スポンサーが作成する募集単位（ターゲット、タスク、予算等を含む） |
