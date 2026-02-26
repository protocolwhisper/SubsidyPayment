# Tech Stack — SnapFuel

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
| **フレームワーク** | React | 18.3.1 |
| **ビルドツール** | Vite | 5.4.11 |
| **言語** | TypeScript | 5.6.3 |
| **3D** | three / @react-three/fiber | 0.176.0 / 8.17.14 |
| **画像処理** | Sharp (OG image 生成) | 0.34.5 |
| **デプロイ** | Vercel | — |
| **Node.js** | Node | 20.x |

## インフラ / デプロイ

| 項目 | 詳細 |
|---|---|
| **バックエンドデプロイ** | Render (render.yaml) |
| **フロントエンドデプロイ** | Vercel |
| **データベース** | PostgreSQL (ローカル: docker-compose.postgres.yml) |
| **マイグレーション** | SQLx マイグレーション (migrations/) |
| **ドキュメント** | Honkit (docs/) |

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
| `X402_FACILITATOR_BEARER_TOKEN` | Facilitator Bearer トークン |
| `X402_NETWORK` | ブロックチェーンネットワーク |
| `X402_PAY_TO` | 受取ウォレットアドレス |
| `X402_ASSET` | 決済アセットアドレス (USDC) |
| `PUBLIC_BASE_URL` | 公開 URL |
| `PORT` | サーバーポート (デフォルト: 3000) |
| `RUST_LOG` | ログレベル設定 |
| `SPONSORED_API_CREATE_PRICE_CENTS` | Sponsored API 作成価格 |
| `SPONSORED_API_TIMEOUT_SECS` | Sponsored API タイムアウト |
| `GPT_ACTIONS_API_KEY` | GPT Actions 用 API キー |
| `AGENT_DISCOVERY_API_KEY` | Agent Discovery 用 API キー |
| `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` | Agent Discovery のレート制限 |
| `CORS_ALLOW_ORIGINS` | CORS 許可オリジン (カンマ区切り) |
| `MCP_SERVER_URL` | MCP サーバー URL (CORS 用) |
| `ZKPASSPORT_VERIFIER_URL` | zkPassport 検証 API |
| `ZKPASSPORT_VERIFIER_API_KEY` | zkPassport 検証 API キー |
| `ZKPASSPORT_VERIFY_PAGE_URL` | zkPassport 検証ページ URL |
| `ZKPASSPORT_SCOPE` | zkPassport スコープ |
| `ZKPASSPORT_VERIFICATION_TTL_SECS` | 検証トークン TTL |
| `ZKPASSPORT_HASH_SALT` | zkPassport ハッシュソルト |

## MCP サーバー

| 項目 | 技術 | バージョン |
|---|---|---|
| **ランタイム** | Node.js | 20+ |
| **言語** | TypeScript | 5.6.0 |
| **フレームワーク** | Express | 4.21.0 |
| **MCP SDK** | @modelcontextprotocol/sdk | 1.26.0 |
| **拡張** | @modelcontextprotocol/ext-apps | 1.0.1 |
| **HTTP クライアント** | Axios | 1.13.5 |
| **EVM** | viem | 2.46.2 |
| **認証** | jsonwebtoken / jwks-rsa | 9.0.2 / 3.2.0 |
| **ログ** | pino | 9.0.0 |
| **バリデーション** | Zod | 3.25.0 |
| **ビルド** | esbuild + Vite (ウィジェット) | — |
| **テスト** | Vitest | 3.0.0 |

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
| `FRONTEND_URL` / `WEB_APP_URL` / `PUBLIC_FRONTEND_URL` | ウィジェット内リンク用フロント URL | `http://localhost:5173`（最終的に解決された値） |
| `X402_WEATHER_URL` | x402 サンプル `weather` エンドポイント URL | `http://localhost:4021/weather` |
| `X402_GITHUB_ISSUE_URL` | x402 サンプル `github-issue` エンドポイント URL | `http://localhost:4021/github-issue` |
| `X402_FACILITATOR_URL` | x402 Facilitator URL | `https://x402.org/facilitator` |
| `X402_NETWORK` | x402 ネットワーク識別子 | `eip155:84532` |
| `X402_PRIVATE_KEY` | x402 署名用秘密鍵 | — |
| `X402_REQUEST_TIMEOUT_MS` | x402 リクエストタイムアウト（ms） | `15000` |

## x402 サンプルサーバー (x402server)

| 項目 | 技術 | バージョン |
|---|---|---|
| **フレームワーク** | Hono | 4.7.1 |
| **x402** | @x402/core / @x402/hono / @x402/evm | latest / latest / latest |
| **HTTP クライアント** | Axios | 1.13.5 |
| **EVM** | viem | 2.46.2 |
| **実行** | tsx / pnpm | 4.7.0 / — |

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

# x402 サンプルサーバー起動
cd x402server && pnpm run dev

# x402 デモクライアント
cd x402server && pnpm run demo

# PostgreSQL 起動 (Docker)
docker compose -f docker-compose.postgres.yml up -d

# テスト（バックエンド）
cargo test

# テスト（MCP サーバー）
cd mcp-server && npm test

# ドキュメント
cd docs && npm run serve
```

## コーディング規約

- Rust: `rustfmt` + `clippy` に準拠
- TypeScript: Vite デフォルト設定
- コミット: Conventional Commits (feat:, fix:, docs:, test:, refactor:, chore:)
- エラー: `thiserror` で型安全なエラー定義、`ApiError` / `ApiResult` パターン

## Sync Notes（2026-02-25）

- MCP ウィジェット `task-form.html` は、`complete_task` 送信前にフロント側で必須バリデーションを実施する。
- `complete_task` の既存入力契約（`details: string`）は維持し、追加情報は JSON 文字列化して後方互換を保つ。
- タスクUI実装では「フォーム入力」「同意チェック」「検証（zkPassport）」「フィードバック入力」を同一画面で段階的に扱うパターンを標準とする。
