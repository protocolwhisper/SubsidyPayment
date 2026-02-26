# GPT Apps Integration — 調査ログ

## サマリー

GPT Apps（Custom GPTs + Custom Actions）をSnapFuelバックエンドに統合するための技術調査を実施。既存のRust/Axumバックエンド、PostgreSQLスキーマ、x402決済フローとの統合ポイントを特定し、GPT Actions固有の制約と認証方式を調査した。

### 調査スコープ

- GPT Actions認証方式（None / APIキー / OAuth）
- OpenAPI 3.1.0スキーマ設計のGPT固有制約
- 既存Axumアーキテクチャへの統合パターン
- Axumミドルウェアによる認証・レート制限の実装方式
- 同意管理のDBスキーマ設計

---

## 調査ログ

### トピック1: GPT Actions認証方式

**ソース**: OpenAI公式ドキュメント（developers.openai.com/api/docs/actions/authentication）

**調査結果**:
- GPT Actionsは3つの認証方式をサポート: **None**, **API Key**, **OAuth**
- **API Key認証**: GPT Builder UIでAPIキーを設定。OpenAIがキーを暗号化して保存。リクエスト時に`Authorization`ヘッダーで送信される
- **OAuth認証**: ユーザー個別のトークンを使用。`authorization_url`, `token_url`, `client_id`, `client_secret` を設定。リダイレクトURLは `https://chatgpt.com/aip/{g-YOUR-GPT-ID-HERE}/oauth/callback`
- **推奨**: 初期フェーズではAPIキー認証を採用。理由: 実装が単純、ユーザーのサインインフローが不要（ドロップオフ防止）、SnapFuelのユーザー識別はAPI内で行う

**影響**:
- バックエンド側で `Authorization: Bearer <api_key>` ヘッダーを検証するミドルウェアが必要
- APIキーは環境変数 `GPT_ACTIONS_API_KEY` で管理
- GPT Builder UIでこのキーを設定する運用手順が必要

### トピック2: OpenAPI 3.1.0スキーマのGPT固有制約

**ソース**: OpenAI公式ドキュメント、chatgpt-apps-developer Skill

**調査結果**:
- OpenAPI 3.1.0形式が必須
- GPTは `description` フィールドを読んで呼び出しタイミングを判断 → 各エンドポイント・パラメータに詳細な `description` が必須
- `operationId` は明確な動詞句（例: `searchServices`, `completeTask`）
- レスポンスはフラットなJSON構造が推奨（3階層以上のネストはGPTが処理しにくい）
- エンドポイント数は30以下が推奨（パフォーマンス劣化防止）
- `servers` フィールドに本番HTTPS URLを記載
- レスポンスサイズ上限は約100KB

**影響**:
- GPT向けエンドポイントは5-8個に絞る（既存18エンドポイントの全公開は不要）
- 既存レスポンス型（`ServiceRunResponse`, `CampaignDiscoveryItem`等）はフラットで再利用可能
- `description` の品質がGPTの呼び出し精度に直結

### トピック3: Axumミドルウェアによる認証実装

**ソース**: 既存コードベース分析 + Axum 0.8ドキュメント

**調査結果**:
- 現在のAxumアプリには認証レイヤーが一切存在しない
- CORSレイヤーは `tower_http::cors::CorsLayer` で実装済み（`src/main.rs` L56-84）
- `Authorization` ヘッダーは既にCORS許可ヘッダーに含まれている（`src/main.rs` L62）
- Axum 0.8では `middleware::from_fn` またはカスタム `Extractor` で認証を実装可能
- GPT専用ルートのみに認証を適用するには、`Router::nest` でサブルーターを作成し、そこにレイヤーを適用

**実装パターン**:
```
// GPT専用ルートに認証ミドルウェアを適用
let gpt_routes = Router::new()
    .route("/services", get(gpt_search_services))
    .route("/auth", post(gpt_auth))
    // ...
    .layer(middleware::from_fn(verify_gpt_api_key));

let app = Router::new()
    .nest("/gpt", gpt_routes)  // 認証付き
    .route("/health", get(health))  // 認証なし
    // 既存ルート...
```

### トピック4: 既存コードベースのモジュール構成

**ソース**: 既存コードベース分析

**調査結果**:
- `src/main.rs` が1583行のモノリシック構成（全ハンドラが集約）
- 型定義は `src/types.rs`（561行）に分離済み
- エラー処理は `src/error.rs`（174行）に分離済み
- ユーティリティは `src/utils.rs`（300行）に分離済み
- オンチェーン処理は `src/onchain.rs`（135行）に分離済み

**影響**:
- GPTエンドポイント群を `src/main.rs` に追加するとさらに肥大化
- **推奨**: `src/gpt.rs` モジュールを新設し、GPT関連ハンドラを分離
- GPT用の型定義は `src/types.rs` に追加（既存パターンに従う）

### トピック5: 同意管理のDB設計

**ソース**: 既存マイグレーション分析 + 要件6

**調査結果**:
- 現在のDBスキーマに同意管理テーブルは存在しない
- `task_completions` テーブルのパターン（user_id + campaign_id + 詳細）を参考にできる
- 同意は「ユーザー × キャンペーン × 同意種別」の粒度で管理が必要

**設計案**:
```sql
CREATE TABLE consents (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id),
  campaign_id UUID NOT NULL REFERENCES campaigns(id),
  consent_type TEXT NOT NULL,  -- 'data_sharing', 'contact', 'retention'
  granted BOOLEAN NOT NULL,
  purpose TEXT,
  retention_days INTEGER,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### トピック6: レート制限の実装方式

**ソース**: tower クレート調査

**調査結果**:
- `tower::limit::RateLimitLayer` はシンプルなトークンバケット方式を提供
- GPT Actionsからのリクエストは通常のAPI利用より頻度が低い（ユーザー会話ベース）
- 初期フェーズでは控えめなレート制限（例: 60 req/min）で十分
- `Retry-After` ヘッダーはカスタムレスポンスで返す必要あり

**影響**:
- `tower::limit::RateLimitLayer` をGPTルートに適用
- 429レスポンス時のカスタムエラーハンドリングが必要

---

## アーキテクチャパターン評価

### パターンA: 既存エンドポイント拡張

- **説明**: 既存APIにクエリパラメータ・レスポンスフィールドを追加
- **メリット**: 変更量最小
- **デメリット**: 認証スコープが不明確、GPT最適化が困難
- **評価**: ❌ 不採用

### パターンB: GPT専用サブルーター（推奨）

- **説明**: `/gpt/*` プレフィックスで専用ルートを新設、`Router::nest` で分離
- **メリット**: 認証スコープ明確、既存API影響ゼロ、GPT向け最適化容易
- **デメリット**: エンドポイント数増加（ただし30以下維持可能）
- **評価**: ✅ 採用

### パターンC: 別サービス分離

- **説明**: GPT Actions用の独立したマイクロサービスを新設
- **メリット**: 完全な分離
- **デメリット**: インフラコスト増、DB共有の複雑さ、MVP段階では過剰
- **評価**: ❌ 不採用（将来的な選択肢として保持）

---

## リスクと未解決事項

| リスク | 影響度 | 緩和策 |
|---|---|---|
| GPTがOpenAPIスキーマを正しく解釈しない | 中 | GPT Builder上での反復テスト、description品質向上 |
| APIキー漏洩 | 高 | 環境変数管理、キーローテーション手順の文書化 |
| main.rsの肥大化 | 中 | gpt.rsモジュール分離で対応 |
| 同意フローのUX | 中 | GPTシステムプロンプトで自然な会話フローを設計 |
