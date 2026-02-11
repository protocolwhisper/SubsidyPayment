# Project Structure — SubsidyPayment

## ディレクトリツリー

```
SubsidyPayment/
├── .kiro/
│   ├── steering/          # AI Steering ファイル（product, tech, structure）
│   └── specs/             # 機能仕様（requirements → design → tasks）
├── .windsurf/
│   ├── mcp_config.json    # MCP サーバー設定
│   ├── skills/            # Windsurf Skills
│   └── workflows/         # Windsurf Workflows
├── src/                   # Rust バックエンド
│   ├── main.rs            # エントリポイント、ルーティング、全ハンドラ
│   ├── types.rs           # 型定義、AppState、AppConfig、DB モデル
│   ├── error.rs           # ApiError / ApiResult エラー型
│   ├── onchain.rs         # x402 オンチェーン決済ロジック
│   ├── utils.rs           # ユーティリティ関数
│   └── test.rs            # テスト
├── frontend/              # React + Vite フロントエンド
│   ├── src/
│   │   ├── App.tsx        # メインアプリケーション（単一ファイル構成）
│   │   ├── main.tsx       # エントリポイント
│   │   └── styles.css     # スタイルシート
│   ├── index.html         # HTML テンプレート
│   ├── vite.config.ts     # Vite 設定
│   ├── vercel.json        # Vercel デプロイ設定
│   └── package.json       # npm 依存関係
├── migrations/            # SQLx PostgreSQL マイグレーション
│   ├── 0001_init.sql              # 初期スキーマ（profiles, resources, offers）
│   ├── 0002_campaigns.sql         # キャンペーンテーブル
│   ├── 0003_task_completions.sql  # タスク完了テーブル
│   ├── 0004_payments.sql          # 支払いテーブル
│   ├── 0005_creator_events.sql    # クリエイターイベントテーブル
│   └── 0006_add_sponsor_wallet.sql # スポンサーウォレット追加
├── docs/                  # GitBook ドキュメント
│   ├── README.md          # メインドキュメント
│   ├── SUMMARY.md         # 目次
│   ├── getting-started/   # セットアップガイド
│   └── protocol/          # プロトコル仕様
├── scripts/
│   └── x402.sh            # x402 テストスクリプト
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
| POST | `/profiles` | プロフィール作成 |
| GET | `/profiles` | プロフィール一覧 |
| POST | `/register` | ユーザー登録 |
| POST | `/campaigns` | キャンペーン作成 |
| GET | `/campaigns` | キャンペーン一覧 |
| GET | `/campaigns/discovery` | キャンペーン検索 |
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

## DB スキーマ概要

マイグレーションファイルから推測される主要テーブル:

| テーブル | 用途 |
|---|---|
| `profiles` | ユーザープロフィール |
| `resources` | x402 保護リソース |
| `offers` | スポンサーオファー |
| `campaigns` | スポンサーキャンペーン |
| `task_completions` | タスク完了記録 |
| `payments` | 支払い記録 |
| `creator_events` | クリエイターイベント |

## アーキテクチャパターン

- **モノリシック Axum サーバー**: 全ハンドラが `main.rs` に集約
- **SharedState パターン**: `Arc<RwLock<AppState>>` で状態共有
- **型安全エラー**: `thiserror` ベースの `ApiError` → HTTP ステータスコード変換
- **x402 プロキシパターン**: 402 レスポンスをインターセプト → Paywall 表示 → スポンサー決済 → リソース返却
- **フロントエンド**: 単一 `App.tsx` ファイル構成（将来的にコンポーネント分割推奨）
