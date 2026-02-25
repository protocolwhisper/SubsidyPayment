# PayloadExchange ドキュメント

PayloadExchange は、AI エージェント開発者とスポンサーをつなぎ、検証可能な利用データと直接的なユーザー接点を提供する代わりに、計算コストを補助するマーケットプレイスです。

---

## 概要

### PayloadExchange とは

PayloadExchange は、AI エージェントのツール利用をマイクロペイメントで収益化できるスポンサード・コンピュート基盤です。スポンサーは、汎用的な代替手段ではなく自社 API やサービスの利用を促進するために、開発者へ報酬を支払います。

**主要コンセプト:**
- **Sponsored Compute**: スポンサーが AI エージェントの運用コストを補助
- **Micropayments**: API コールやツール利用ごとに暗号資産でリアルタイム決済
- **Verified Data**: 検証済みの利用メトリクスとアトリビューションをスポンサーに提供

### 背景課題

AI エージェント開発には、次のような継続的コストが発生します。
- **LLM API コスト**: 推論時のトークン消費
- **ツール/API コスト**: 外部サービス利用料金（検索 API、DB クエリ等）
- **インフラコスト**: エージェント実行用の計算資源

利用量に比例してコストが増えるため、運用規模の拡大が難しくなります。

### ソリューション

PayloadExchange は、次のマーケットプレイスモデルでこの課題を解決します。

1. **スポンサー**がツール利用に対するマイクロペイメント付きキャンペーンを作成
2. **開発者**がスポンサーツールをエージェントに統合
3. **プラットフォーム**が x402 プロトコルで決済を処理
4. **検証レイヤー**が正当利用を確認し不正を抑止

---

## 現在の実装ステータス（2026-02-25 時点）

### Rust バックエンド（`src/`）
- Core ルート: `/campaigns`, `/proxy/{service}/run`, `/sponsored-apis`, `/payments`, `/creator/metrics`
- GPT ルート: `/gpt/services`, `/gpt/auth`, `/gpt/tasks/{campaign_id}`, `/gpt/tasks/{campaign_id}/complete`, `/gpt/tasks/{campaign_id}/zkpassport/init`, `/gpt/user/status`, `/gpt/user/record`, `/gpt/preferences`
- Discovery エイリアス: `/agent/discovery/services`, `/claude/discovery/services`, `/openclaw/discovery/services`
- zkPassport ルート: `/verify/zkpassport`, `/zkpassport/session/{verification_token}`, `/zkpassport/session/{verification_token}/submit`

### MCP サーバー（`mcp-server/`）
- 通信/API: `/mcp`（Streamable HTTP）+ `/health` + OAuth メタデータエンドポイント
- 登録ツール（14個）:
  - `search_services`
  - `authenticate_user`
  - `create_campaign_from_goal`
  - `get_service_tasks`
  - `get_task_details`
  - `start_zkpassport_verification`
  - `complete_task`
  - `run_service`
  - `get_user_status`
  - `get_user_record`
  - `get_preferences`
  - `set_preferences`
  - `weather`
  - `github_issue`
- ウィジェットリソース（6個）:
  - `services-list`
  - `services-list-v2`
  - `service-tasks`
  - `task-form`
  - `service-access`
  - `user-dashboard`

### データベース / マイグレーション状況
- 最新マイグレーション: `0014_zkpassport_verifications.sql`
- スマートサジェスト関連（`0011`, `0012`, `0013`）は実装済み

### Kiro 仕様進捗
- `gpt-apps-integration`: 33/33 完了
- `smart-service-suggestion`: 32/32 完了
- `refactor-to-gpt-app-sdk`: 27/27 完了
- `autonomous-agent-execution`: 0/41（未実装）

---

## コアコンセプト

### Skills / Tools

**Skill**（Tool / Function）は、LLM から呼び出せる実行可能機能です。Skill は JSON Schema で定義され、関数シグネチャ・引数・戻り値を明示します。

**Skill 定義例:**
```json
{
  "name": "get_weather",
  "description": "Retrieve current weather conditions",
  "parameters": {
    "type": "object",
    "properties": {
      "city": {
        "type": "string",
        "description": "City name"
      }
    },
    "required": ["city"]
  }
}
```

**実行フロー:**
1. ユーザーが LLM にプロンプトを送信
2. LLM が適切な Skill を選定
3. LLM が引数付きで Skill 実行を要求
4. Skill が実行され結果を返却
5. LLM が結果を応答へ反映

**PayloadExchange での意味**: スポンサーは、競合ツールではなく自社 Skill が使われるよう、開発者に報酬インセンティブを提供します。

### Model Context Protocol (MCP)

**Model Context Protocol** は、AI ツールを言語モデルへ接続するための標準インターフェースです。MCP により、OpenAI / Anthropic / Google など複数の LLM に対して、プロバイダー固有実装なしで接続できます。

**主な利点:**
- **Interoperability**: 一度実装すれば複数 LLM 基盤で再利用可能
- **Standardization**: ツール統合の一貫したインターフェース
- **Extensibility**: 新規ツールや機能の追加が容易

**PayloadExchange 実装**: スポンサーツールを MCP サーバーとして配布し、MCP 対応エージェントへ即時統合可能にします。

---

## システムアーキテクチャ

### トランザクションフロー

```
┌─────────────────┐
│ Sponsor Company │
└────────┬────────┘
         │ Funds Campaign
         ↓
┌─────────────────┐
│ PayloadExchange │
│   Marketplace   │
└────────┬────────┘
         │ Lists Sponsored Tool
         ↓
┌─────────────────┐
│   Developer/    │
│      Agent      │
└────────┬────────┘
         │ Uses Tool via MCP
         ↓
┌─────────────────┐
│ Service Provider│
└────────┬────────┘
         │ Triggers x402 Request
         ↓
┌─────────────────┐
│ PayloadExchange │
│ Payment Layer   │
└────────┬────────┘
         │
    ┌────┴────┐
    ↓         ↓
┌────────┐ ┌──────────────┐
│Payment │ │ Usage Data   │
│Settlement│ Attribution │
└────────┘ └──────────────┘
```

### トランザクションライフサイクル

1. **キャンペーン作成**: スポンサーが対象ユーザー・予算・コール単価・API エンドポイントを設定
2. **ツール発見**: 開発者がマーケットプレイスからスポンサーツールを選択
3. **統合**: MCP サーバーまたは SDK ラッパーを導入
4. **実行**: エージェントが通常処理内でスポンサーツールを呼び出し
5. **決済処理**: x402 がリクエストを検証し、スポンサーウォレットから開発者ウォレットへ送金
6. **データ帰属**: 利用メトリクスを記録し、検証済みデータをスポンサーへ提供

---

## プラットフォーム機能

### スポンサーポータル

**Campaign Management**
- 対象ユーザーとターゲティング条件の定義
- 予算上限と支払いスケジュールの設定
- API エンドポイントと統合要件の設定
- キャンペーン成果のリアルタイム監視

**Analytics Dashboard**
- 検証済みツール利用メトリクス
- 予算消費状況とバーンレート
- ユーザーエンゲージメントとアトリビューション
- ROI 分析と最適化提案

### 開発者ポータル

**Marketplace**
- 利用可能なスポンサーツールの検索
- 報酬単価・カテゴリ・要件でフィルタ
- 統合ドキュメントとサンプル確認
- 収益と支払い履歴の追跡

**Wallet Integration**
- EVM 互換ウォレット接続（Ethereum, Polygon 等）
- マイクロペイメントのリアルタイム受取
- 取引履歴と収益の確認
- 複数ツール統合の管理

### 統合レイヤー

**MCP Server Distribution**
- スポンサーツール向け標準 MCP サーバー
- x402 決済ヘッダーの自動付与
- 支払い検証と正当性確認
- 利用追跡とレポート

**SDK Support**
- 非 MCP 統合向け言語別 SDK
- 決済フロー処理の簡略化
- 組み込みのエラー処理とリトライ
- 開発者向け API ラッパー

### MCP ツール仕様（MVP）

目的やターゲットユーザーを入力すると、適切なサービス・タスク内容を含んだキャンペーンを自動作成するツールを提供します。

**ツール名**
- `create_campaign_from_goal`

**目的**
- 目的/ターゲットの入力から、キャンペーン作成に必要な項目を自動生成し、`POST /campaigns` を実行する

**ツール定義（MCP）**
```json
{
  "name": "create_campaign_from_goal",
  "description": "Create a sponsor campaign from a purpose and target audience.",
  "parameters": {
    "type": "object",
    "properties": {
      "purpose": { "type": "string" },
      "sponsor": { "type": "string" },
      "target_roles": { "type": "array", "items": { "type": "string" } },
      "target_tools": { "type": "array", "items": { "type": "string" } },
      "budget_cents": { "type": "number" },
      "query_urls": { "type": "array", "items": { "type": "string" } },
      "region": { "type": "string" },
      "intent": { "type": "string" },
      "max_budget_cents": { "type": "number" }
    },
    "required": ["purpose", "sponsor", "target_roles", "budget_cents"]
  }
}
```

**入力（JSON Schema 概要）**
- `purpose`: 目的の要約（必須）
- `sponsor`: スポンサー名（必須）
- `target_roles`: ターゲットロール配列（必須）
- `target_tools`: ターゲットツール配列（任意）
- `budget_cents`: 予算（セント）（必須）
- `query_urls`: 上流 URL 配列（任意）
- `region`: 対象リージョン（任意）
- `intent`: 目的の詳細意図（任意）
- `max_budget_cents`: 1回あたり予算上限（任意）

**処理フロー（MVP）**
1. `GET /gpt/services` で候補サービスを検索
2. 上位候補から `required_task` と `target_tools` を決定
3. `POST /campaigns` に作成リクエストを送信

**出力**
- `campaign_id`: 作成されたキャンペーン ID
- `campaign`: 作成済みキャンペーン内容
- `selected_service_key`: 選定したサービスキー
- `selected_offer`: 選定した候補（campaign_id / sponsor / required_task / subsidy_amount_cents）
- `selected_services`: 参照したサービス候補
- `selected_task`: 採用したタスク内容（required_task / subsidy_per_call_cents）
- `rationale`: 選定理由の要約

**実行例（入力）**
```json
{
  "purpose": "AI チャットの利用体験を改善したい",
  "sponsor": "Acme Corp",
  "target_roles": ["customer-support", "product-manager"],
  "budget_cents": 25000,
  "intent": "FAQ 回答品質を上げる",
  "max_budget_cents": 150
}
```

**実行例（出力・structuredContent）**
```json
{
  "campaign_id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
  "campaign": {
    "id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
    "name": "Acme Corp AI チャットの利用体験を改善したい",
    "sponsor": "Acme Corp",
    "sponsor_wallet_address": null,
    "target_roles": ["customer-support", "product-manager"],
    "target_tools": ["faq_search"],
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120,
    "budget_total_cents": 25000,
    "budget_remaining_cents": 25000,
    "query_urls": [],
    "active": true,
    "created_at": "2026-02-23T09:00:00Z"
  },
  "selected_service_key": "faq_search",
  "selected_offer": {
    "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
    "campaign_name": "FAQ Search Sponsors",
    "sponsor": "Acme Corp",
    "required_task": "share_feedback",
    "subsidy_amount_cents": 120
  },
  "selected_services": [],
  "selected_task": {
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120
  },
  "rationale": "Selected the highest-subsidy offer from candidate services."
}
```

**失敗時の返却方針**
- 目的が曖昧で候補が出ない場合は、足りない情報を明示してエラー返却
- 予算不整合（`budget_cents` が不足）時は不足額の説明を返却
- `target_tools` が補完できない場合はエラー返却

**エラーレスポンス例（MCP）**
```json
{
  "content": [
    {
      "type": "text",
      "text": "No suitable sponsored services found. Try a more specific purpose or adjust the budget."
    }
  ],
  "_meta": {
    "code": "no_candidate_service",
    "details": {
      "services": [],
      "total_count": 0,
      "message": "No services matched"
    }
  },
  "isError": true
}
```

```json
{
  "content": [
    {
      "type": "text",
      "text": "Budget is below the selected subsidy amount. Increase budget or adjust purpose."
    }
  ],
  "_meta": {
    "code": "budget_too_low",
    "details": {
      "budget_cents": 80,
      "subsidy_per_call_cents": 120
    }
  },
  "isError": true
}
```

```json
{
  "content": [
    {
      "type": "text",
      "text": "Target tools could not be determined. Provide target_tools explicitly."
    }
  ],
  "_meta": {
    "code": "missing_target_tools",
    "details": {
      "service_key": "",
      "offer": {
        "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
        "campaign_name": "FAQ Search Sponsors",
        "sponsor": "Acme Corp",
        "required_task": "share_feedback",
        "subsidy_amount_cents": 120
      },
      "source": "service"
    }
  },
  "isError": true
}
```

**structuredContent 詳細例**
```json
{
  "campaign_id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
  "campaign": {
    "id": "2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90",
    "name": "Acme Corp AI チャットの利用体験を改善したい",
    "sponsor": "Acme Corp",
    "sponsor_wallet_address": null,
    "target_roles": ["customer-support", "product-manager"],
    "target_tools": ["faq_search"],
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120,
    "budget_total_cents": 25000,
    "budget_remaining_cents": 25000,
    "query_urls": ["https://example.com/faq"],
    "active": true,
    "created_at": "2026-02-23T09:00:00Z"
  },
  "selected_service_key": "faq_search",
  "selected_offer": {
    "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
    "campaign_name": "FAQ Search Sponsors",
    "sponsor": "Acme Corp",
    "required_task": "share_feedback",
    "subsidy_amount_cents": 120
  },
  "selected_services": [
    {
      "service_key": "faq_search",
      "display_name": "FAQ Search",
      "reason": "ユーザーの目的に合致",
      "offer_count": 2,
      "offers": [
        {
          "campaign_id": "5d6a4b27-2f6c-4c5f-9d9e-0bb0f8870d29",
          "campaign_name": "FAQ Search Sponsors",
          "sponsor": "Acme Corp",
          "required_task": "share_feedback",
          "subsidy_amount_cents": 120
        }
      ]
    }
  ],
  "selected_task": {
    "required_task": "share_feedback",
    "subsidy_per_call_cents": 120
  },
  "rationale": "Selected the highest-subsidy offer from candidate services."
}
```

**ツール呼び出しプロンプト例**
- 「顧客サポート向けにFAQ回答品質を上げたい。予算は$250で、スポンサーはAcme Corp。キャンペーンを作成して」
- 「B2B向けにオンボーディング改善を目的としたキャンペーンを作成して。ターゲットはプロダクトマネージャー、予算は300ドル」
- 「AIチャット改善のために、必要なタスクとツールを自動選定してキャンペーンを作って」

### 検証システム

**Proof of Action**

x402 の支払い成功シグナルに基づいて利用を検証し、次を暗号学的に証明します。
- 実際にツールが呼び出された
- リクエストが正当である（偽装でない）
- 決済が正しく処理された

**Anti-Abuse Measures**
- レート制限と利用上限
- Bot 検知とフィルタリング
- 開発者レピュテーションスコア
- スポンサー向け品質フィルター

---

## 収益モデル

### Revenue Model

**Transaction Fees**
- 各取引から一定割合をプラットフォーム手数料として徴収（例: 20%）
- 例: スポンサー $0.05/回 → 開発者 $0.04/回 → プラットフォーム $0.01/回
- 従来広告の CPC（$2〜$5/クリック）と比較して競争力あり

**Data Access Fees**
- プレミアム分析と詳細利用レポート
- ユーザー帰属データとエンゲージメント指標
- カスタムエクスポートと API アクセス
- 開発者のプライバシー同意が前提

**Verification Services**
- 高評価開発者向け品質フィルタリング
- Bot 検知・スパム防止
- キャンペーン別のカスタム検証ルール
- 月額 SaaS サブスクリプション

### Value Proposition

**スポンサー向け:**
- インプレッションではない検証済みエンゲージメント
- AI エージェント利用パターンへの直接アクセス
- 従来広告より低いエンゲージメント単価
- キャンペーンのリアルタイム最適化

**開発者向け:**
- 運用コストの補助
- エージェント利用からの収益化可能性
- 高機能 API への無償アクセス
- デプロイ済みエージェントからの継続収益

---

## 統合ガイド

### Sponsored Skill Schema

スポンサーツール統合時、開発者には次のような JSON スキーマが提供されます。

```json
{
  "skill_id": "supersearch_v1",
  "name": "SuperSearch API",
  "sponsor": "Acme Corp",
  "payout_per_call": "0.05",
  "currency": "USDC",
  "mcp_server_url": "https://mcp.payloadexchange.com/supersearch",
  "function_schema": {
    "name": "search",
    "description": "Search the web using SuperSearch",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "Search query"
        }
      },
      "required": ["query"]
    }
  },
  "x402_endpoint": "https://api.supersearch.com/v1/search",
  "verification": {
    "method": "x402_payment_success",
    "required_headers": ["X-402-Payment-Token"]
  }
}
```

### 統合ワークフロー

1. **Discovery**: マーケットプレイスで対象ツールを選定
2. **Installation**: MCP サーバーまたは SDK ラッパーを導入
3. **Configuration**: ウォレット連携とエージェント設定
4. **Deployment**: スポンサーツール統合済みエージェントをデプロイ
5. **Monitoring**: 開発者ダッシュボードで利用量と収益を追跡

### MCP サーバー実装例

```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { ListToolsRequestSchema, CallToolRequestSchema } from "@modelcontextprotocol/sdk/types.js";

const server = new Server({
  name: "supersearch-sponsored",
  version: "1.0.0",
});

// 利用可能なツールを登録
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [{
    name: "search",
    description: "Search using SuperSearch (sponsored)",
    inputSchema: {
      type: "object",
      properties: {
        query: {
          type: "string",
          description: "Search query"
        }
      },
      required: ["query"]
    }
  }]
}));

// x402 決済付きでツール実行
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { query } = request.params.arguments;

  // PayloadExchange から支払いトークンを取得
  const paymentToken = await getPaymentToken();

  // x402 ヘッダー付きで API 呼び出し
  const response = await fetch("https://api.supersearch.com/v1/search", {
    method: "POST",
    headers: {
      "X-402-Payment-Token": paymentToken,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ query })
  });

  // 支払いは x402 レイヤーで自動処理
  const data = await response.json();

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(data)
      }
    ]
  };
});
```

---

## はじめ方

### 開発者向け

1. **ウォレット準備**: EVM 互換ウォレット（MetaMask, WalletConnect など）を用意
2. **アカウント作成**: PayloadExchange に登録しウォレットを連携
3. **マーケット探索**: ツールと報酬単価を確認
4. **統合**: 選択したツールの MCP サーバーまたは SDK を導入
5. **運用開始**: エージェントをデプロイし利用に応じて収益化

### スポンサー向け

1. **アカウント準備**: スポンサーアカウントと資金ウォレットを接続
2. **キャンペーン作成**: 予算・ターゲティング・条件を定義
3. **ツール登録**: API エンドポイントと要件を登録
4. **モニタリング**: 分析ダッシュボードで成果を確認
5. **最適化**: 実績データに基づき配分と条件を調整

### プラットフォーム貢献者向け

1. **プロトコル開発**: x402 実装へコントリビュート
2. **MCP テンプレート**: 代表ユースケースの参照実装作成
3. **SDK 開発**: 言語別 SDK とラッパー作成
4. **ドキュメント整備**: 統合ガイドと API リファレンス改善
5. **テスト**: 決済フローと検証システムの妥当性確認

---

## ビジョンとロードマップ

PayloadExchange は、Google・Visa・Cloudflare が支援する **x402 プロトコル**を土台にしています。安定通貨を用いた Just-In-Time 型のリソース調達により、売り手と買い手の事前登録なしで取引できるインターネットを目指します。

**コアミッション**: スポンサード・コンピュートを通じて AI エージェントが持続的に収益化でき、同時にスポンサーが検証済みエンゲージメントと直接的なユーザー接点を得られる、新しい広告代替モデルを実現する。

**今後の拡張:**
- マルチチェーン決済対応
- 高度なターゲティングとセグメント管理
- ツール掲載のリアルタイム入札
- 開発者レピュテーション / 認証制度
- キャンペーン最適化の自動化

---

*このドキュメントはオープンソースです。 [GitHub](https://github.com/yourusername/payloadexchange-docs) からコントリビュートできます。*
