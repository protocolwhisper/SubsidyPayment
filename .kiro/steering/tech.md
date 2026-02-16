# Tech Stack — SubsidyPayment

## バックエンド

| 項目 | 技術 | バージョン |
|---|---|---|
| **言語** | Rust | Edition 2024 |
| **Web フレームワーク** | Axum | 0.8 |
| **非同期ランタイム** | Tokio | 1.49 |
| **データベース** | PostgreSQL (SQLx) | SQLx 0.8 |
| **HTTP クライアント** | Reqwest | 0.13 (rustls) |
| **シリアライズ** | Serde / serde_json | 1.x |
| **メトリクス** | Prometheus | 0.14 |
| **ログ** | tracing + tracing-subscriber | 0.1 / 0.3 |
| **CORS** | tower-http | 0.6 |
| **UUID** | uuid (v4) | 1.x |
| **日時** | chrono | 0.4 |
| **エラー** | thiserror | 2.x |

## フロントエンド

| 項目 | 技術 | バージョン |
|---|---|---|
| **フレームワーク** | React | 18.3 |
| **ビルドツール** | Vite | 5.4 |
| **言語** | TypeScript | 5.6 |
| **画像処理** | Sharp (OG image 生成) | 0.34 |
| **デプロイ** | Vercel | — |

## インフラ / デプロイ

| 項目 | 詳細 |
|---|---|
| **バックエンドデプロイ** | Render (render.yaml) |
| **フロントエンドデプロイ** | Vercel |
| **データベース** | PostgreSQL (ローカル: docker-compose.postgres.yml) |
| **マイグレーション** | SQLx マイグレーション (migrations/) |
| **ドキュメント** | GitBook (docs/) |

## 外部プロトコル / API

| 項目 | 詳細 |
|---|---|
| **x402 プロトコル** | HTTP 402 ベースのペイメントプロトコル |
| **x402 Facilitator** | https://x402.org/facilitator (verify / settle) |
| **ネットワーク** | Base Sepolia (テストネット) |
| **決済** | USDC (テストネット) |

## 環境変数

| 変数名 | 用途 |
|---|---|
| `DATABASE_URL` | PostgreSQL 接続文字列 |
| `X402_FACILITATOR_URL` | x402 Facilitator エンドポイント |
| `X402_VERIFY_PATH` / `X402_SETTLE_PATH` | Facilitator のパス |
| `X402_NETWORK` | ブロックチェーンネットワーク |
| `X402_PAY_TO` | 受取ウォレットアドレス |
| `X402_ASSET` | 決済アセットアドレス (USDC) |
| `PUBLIC_BASE_URL` | 公開 URL |
| `PORT` | サーバーポート (デフォルト: 3000) |
| `RUST_LOG` | ログレベル設定 |
| `SPONSORED_API_CREATE_PRICE_CENTS` | Sponsored API 作成価格 |
| `SPONSORED_API_TIMEOUT_SECS` | Sponsored API タイムアウト |

## MCP サーバー

| 項目 | 技術 | バージョン |
|---|---|---|
| **ランタイム** | Node.js | 22+ |
| **言語** | TypeScript | 5.x |
| **フレームワーク** | Express | 4.x |
| **MCP SDK** | @modelcontextprotocol/sdk | — |
| **認証** | Auth0 JWT (jwks-rsa) | — |
| **ログ** | pino | — |
| **バリデーション** | Zod | — |
| **ビルド** | esbuild + Vite (ウィジェット) | — |
| **テスト** | Vitest | 3.x |

### MCP サーバー環境変数

| 変数名 | 用途 | デフォルト |
|---|---|---|
| `RUST_BACKEND_URL` | Rust バックエンド URL | `http://localhost:3000` |
| `MCP_INTERNAL_API_KEY` | バックエンド通信用 API キー | — |
| `AUTH0_DOMAIN` | Auth0 テナントドメイン | — |
| `AUTH0_AUDIENCE` | Auth0 API オーディエンス | — |
| `PUBLIC_URL` | MCP サーバー公開 URL | `http://localhost:3001` |
| `PORT` | MCP サーバーポート | `3001` |
| `LOG_LEVEL` | ログレベル | `info` |
| `AUTH_ENABLED` | OAuth 認証トグル (`true`/`false`)。未設定時は AUTH0_DOMAIN と AUTH0_AUDIENCE が両方あれば有効、なければ無効 | 自動判定 |

## 開発コマンド

```bash
# バックエンド起動
cargo run

# フロントエンド起動
cd frontend && npm run dev

# MCP サーバー起動
cd mcp-server && npm run dev

# MCP サーバー起動（認証OFF / MVPモード）
cd mcp-server && AUTH_ENABLED=false npm run dev

# PostgreSQL 起動 (Docker)
docker compose -f docker-compose.postgres.yml up -d

# テスト（バックエンド）
cargo test

# テスト（MCP サーバー）
cd mcp-server && npm test

# ドキュメント
cd docs && npx gitbook serve
```

## コーディング規約

- Rust: `rustfmt` + `clippy` に準拠
- TypeScript: Vite デフォルト設定
- コミット: Conventional Commits (feat:, fix:, docs:, test:, refactor:, chore:)
- エラー: `thiserror` で型安全なエラー定義、`ApiError` / `ApiResult` パターン
