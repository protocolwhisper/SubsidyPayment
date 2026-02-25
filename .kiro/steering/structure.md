# Project Structure — SubsidyPayment

## ディレクトリツリー

```
SubsidyPayment/
├── .claude/              # Kiro コマンド / Skills
├── .kiro/
│   ├── steering/          # AI Steering ファイル（product, tech, structure）
│   └── specs/             # 機能仕様（requirements → design → tasks）
├── .serena/               # Serena メモリー
├── .trae/                 # Trae ルール / Skills
├── .windsurf/             # Windsurf ルール / Skills
├── agents/                # エージェント向けプロンプト
├── design/                # UI デザインデータ
├── src/                   # Rust バックエンド
│   ├── main.rs            # エントリポイント、ルーティング、全ハンドラ
│   ├── gpt.rs             # GPT Apps 向けルート / ロジック
│   ├── types.rs           # 型定義、AppState、AppConfig、DB モデル
│   ├── error.rs           # ApiError / ApiResult エラー型
│   ├── onchain.rs         # x402 オンチェーン決済ロジック
│   ├── utils.rs           # ユーティリティ関数
│   ├── test.rs            # テスト
│   └── zkpassport_verify.html # zkPassport 検証ページ
├── frontend/              # React + Vite フロントエンド
│   ├── src/
│   │   ├── App.tsx        # ランディングページ本体
│   │   ├── GetStartedButton3D.tsx # 3D CTA ボタン
│   │   ├── LandingBackground3D.tsx # 3D 背景
│   │   ├── main.tsx       # エントリポイント
│   │   └── styles.css     # スタイルシート
│   ├── public/            # 静的アセット（logo / OG画像）
│   ├── scripts/           # OG画像生成スクリプト
│   ├── index.html         # HTML テンプレート
│   ├── vite.config.ts     # Vite 設定
│   ├── vercel.json        # Vercel デプロイ設定
│   └── package.json       # npm 依存関係
├── mcp-server/            # MCP サーバー (Node/Express)
│   ├── src/               # サーバー本体（tools/auth/widgets）
│   ├── src/widgets/       # GPT Apps 向けウィジェット実装
│   ├── __tests__/         # ユニットテスト
│   ├── tests/             # タスク単位の検証テスト
│   └── app-metadata.json  # MCP アプリメタデータ
├── x402server/            # x402 サンプルサーバー / クライアント
├── migrations/            # SQLx PostgreSQL マイグレーション
│   ├── 0001_init.sql              # 初期スキーマ
│   ├── 0002_campaigns.sql         # キャンペーンテーブル
│   ├── 0003_task_completions.sql  # タスク完了テーブル
│   ├── 0004_payments.sql          # 支払いテーブル
│   ├── 0005_creator_events.sql    # クリエイターイベントテーブル
│   ├── 0006_add_sponsor_wallet.sql # スポンサーウォレット追加
│   ├── 0007_consents.sql          # 同意レコード
│   ├── 0008_add_user_source.sql   # ユーザーソース追加
│   ├── 0009_gpt_sessions.sql      # GPT セッション
│   ├── 0010_add_task_schema.sql   # タスクスキーマ
│   ├── 0011_user_task_preferences.sql # 嗜好管理
│   ├── 0012_campaign_tags.sql     # キャンペーンタグ
│   ├── 0013_gpt_service_runs.sql  # GPT サービス実行履歴
│   └── 0014_zkpassport_verifications.sql # zkPassport 検証
├── docs/                  # Honkit ドキュメント
│   ├── en/                # 英語版
│   └── ja/                # 日本語版
├── scripts/               # 運用補助スクリプト
├── test_data/             # デモデータ管理
├── skills/                # カスタム Skills
├── Cargo.toml             # Rust 依存関係
├── Cargo.lock             # Rust ロックファイル
├── render.yaml            # Render デプロイ設定
├── docker-compose.postgres.yml  # ローカル PostgreSQL
├── AGENTS.md              # AI-DLC 開発ガイドライン
├── README.md              # プロジェクト概要・要件
└── .env.example           # 環境変数テンプレート
```

## API エンドポイント一覧

| メソッド | パス | 機能 |
|---|---|---|
| GET | `/health` | ヘルスチェック |
| GET | `/verify/zkpassport` | zkPassport 検証ページ |
| GET | `/zkpassport/session/{verification_token}` | zkPassport セッション取得 |
| POST | `/zkpassport/session/{verification_token}/submit` | zkPassport 証明送信 |
| POST | `/profiles` | プロフィール作成 |
| GET | `/profiles` | プロフィール一覧 |
| POST | `/register` | ユーザー登録 |
| POST | `/campaigns` | キャンペーン作成 |
| GET | `/campaigns` | キャンペーン一覧 |
| GET | `/campaigns/discovery` | キャンペーン検索 |
| GET | `/agent/discovery/services` | Agent Discovery サービス一覧 |
| GET | `/claude/discovery/services` | Claude Discovery サービス一覧 |
| GET | `/openclaw/discovery/services` | OpenClaw Discovery サービス一覧 |
| GET | `/campaigns/{campaign_id}` | キャンペーン詳細 |
| POST | `/tasks/complete` | タスク完了 |
| POST | `/tool/{service}/run` | ツール実行 |
| POST | `/proxy/{service}/run` | プロキシ実行 |
| POST | `/sponsored-apis` | Sponsored API 作成 |
| GET | `/sponsored-apis` | Sponsored API 一覧 |
| GET | `/sponsored-apis/{api_id}` | Sponsored API 詳細 |
| POST | `/sponsored-apis/{api_id}/run` | Sponsored API 実行 |
| POST | `/webhooks/x402scan/settlement` | x402 決済 Webhook |
| GET | `/dashboard/sponsor/{campaign_id}` | スポンサーダッシュボード |
| POST | `/creator/metrics/event` | クリエイターメトリクスイベント記録 |
| GET | `/creator/metrics` | クリエイターメトリクス取得 |
| GET | `/metrics` | Prometheus メトリクス |
| GET | `/.well-known/openapi.yaml` | OpenAPI 定義 |
| GET | `/privacy` | プライバシー文書 |
| GET | `/gpt/services` | GPT Apps サービス検索 |
| POST | `/gpt/auth` | GPT Apps 認証 |
| GET | `/gpt/tasks/{campaign_id}` | GPT Apps タスク取得 |
| POST | `/gpt/tasks/{campaign_id}/complete` | GPT Apps タスク完了 |
| POST | `/gpt/tasks/{campaign_id}/zkpassport/init` | GPT Apps zkPassport 初期化 |
| POST | `/gpt/services/{service}/run` | GPT Apps サービス実行 |
| GET | `/gpt/user/status` | GPT Apps ユーザーステータス |
| GET | `/gpt/user/record` | GPT Apps 利用履歴 |
| GET/POST | `/gpt/preferences` | GPT Apps 嗜好取得/更新 |

## DB スキーマ概要

マイグレーションファイルから確認できる主要テーブル:

| テーブル | 用途 |
|---|---|
| `users` | GPT Apps ユーザー |
| `resources` | x402 保護リソース |
| `offers` | スポンサーオファー |
| `campaigns` | スポンサーキャンペーン |
| `task_completions` | タスク完了記録 |
| `payments` | 支払い記録 |
| `creator_events` | クリエイターイベント |
| `consents` | 同意レコード |
| `gpt_sessions` | GPT セッション |
| `user_task_preferences` | 嗜好設定 |
| `campaign_tags` | キャンペーンタグ |
| `gpt_service_runs` | GPT サービス実行履歴 |
| `zkpassport_verifications` | zkPassport 検証 |

## アーキテクチャパターン

- **モノリシック Axum サーバー**: 全ハンドラが `main.rs` に集約
- **SharedState パターン**: `Arc<RwLock<AppState>>` で状態共有
- **型安全エラー**: `thiserror` ベースの `ApiError` → HTTP ステータスコード変換
- **x402 プロキシパターン**: 402 レスポンスをインターセプト → Paywall 表示 → スポンサー決済 → リソース返却
- **GPT Apps 連携**: `/gpt` ルート配下に GPT Apps 向け API を集約
- **Agent Discovery 連携**: `/agent/discovery` `/claude/discovery` `/openclaw/discovery` を共通ロジックで提供
- **zkPassport 連携**: 検証ページ + セッション API を提供
- **MCP App 構成**: OAuth + MCP Tools + Vite singlefile ウィジェットを同一リポジトリで運用
- **フロントエンド**: `App.tsx` + 3D コンポーネント分割構成
