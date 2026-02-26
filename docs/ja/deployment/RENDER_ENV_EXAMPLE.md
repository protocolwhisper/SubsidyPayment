# Render 環境変数設定例（現行）

このページは 2026-02-25 時点の実装に基づいています。

## 1. Rust バックエンドサービス（`payloadexchange-backend`）

### 必須

| Key | 設定例 | 補足 |
|---|---|---|
| `DATABASE_URL` | `postgres://...` | Render Postgres の **Internal Database URL** を使用 |
| `PUBLIC_BASE_URL` | `https://payloadexchange-backend.onrender.com` | 公開バックエンド URL |

### 推奨

| Key | 設定例 | 補足 |
|---|---|---|
| `RUST_LOG` | `payloadexchange_mvp=info,tower_http=info` | ログ設定 |
| `CORS_ALLOW_ORIGINS` | `https://your-frontend.vercel.app,https://chatgpt.com,https://chat.openai.com` | カンマ区切り |
| `MCP_SERVER_URL` | `https://your-mcp-server.onrender.com` | CORS 許可に追加 |
| `X402_FACILITATOR_URL` | `https://x402.org/facilitator` | x402 設定 |
| `X402_VERIFY_PATH` | `/verify` | x402 設定 |
| `X402_SETTLE_PATH` | `/settle` | x402 設定 |
| `X402_NETWORK` | `base-sepolia` | x402 設定 |
| `SPONSORED_API_CREATE_PRICE_CENTS` | `25` | Sponsored API 作成価格 |
| `SPONSORED_API_TIMEOUT_SECS` | `12` | 上流タイムアウト |
| `GPT_ACTIONS_API_KEY` | `your-secret-api-key` | 設定時のみ GPT API キー認証を有効化 |
| `AGENT_DISCOVERY_API_KEY` | `your-agent-api-key` | Discovery API 任意認証 |
| `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` | `120` | Discovery API レート制限 |
| `ZKPASSPORT_VERIFIER_URL` | `https://your-mcp-server.onrender.com/internal/zkpassport/verify` | 検証エンドポイント |
| `ZKPASSPORT_VERIFIER_API_KEY` | `your-zkpassport-key` | 有効化時は MCP 側キーと一致させる |
| `ZKPASSPORT_VERIFY_PAGE_URL` | `https://payloadexchange-backend.onrender.com/verify/zkpassport` | 公開検証ページ |
| `ZKPASSPORT_SCOPE` | `snapfuel-gpt-age-country-v1` | スコープ名 |
| `ZKPASSPORT_VERIFICATION_TTL_SECS` | `900` | トークン TTL |
| `ZKPASSPORT_HASH_SALT` | `replace-in-production` | 識別子ハッシュ用ソルト |

## 2. MCP サーバーサービス（`snapfuel-mcp-server`）

### 必須

| Key | 設定例 | 補足 |
|---|---|---|
| `RUST_BACKEND_URL` | `https://payloadexchange-backend.onrender.com` | バックエンド URL |
| `PUBLIC_URL` | `https://your-mcp-server.onrender.com` | MCP 公開 URL |

### 任意 / 機能依存

| Key | 設定例 | 補足 |
|---|---|---|
| `AUTH0_DOMAIN` | `your-tenant.auth0.com` | OAuth |
| `AUTH0_AUDIENCE` | `https://api.your-domain` | OAuth |
| `AUTH_ENABLED` | `true` / `false` | 自動判定を上書き |
| `MCP_INTERNAL_API_KEY` | `your-mcp-internal-api-key` | バックエンドへ Bearer 送信 |
| `FRONTEND_URL` | `https://your-frontend.vercel.app` | ツールが返すリンク先 |
| `LOG_LEVEL` | `info` | ログレベル |
| `X402_WEATHER_URL` | `https://your-x402-sample/weather` | Weather ツール用 |
| `X402_GITHUB_ISSUE_URL` | `https://your-x402-sample/github-issue` | Github Issue ツール用 |
| `X402_FACILITATOR_URL` | `https://x402.org/facilitator` | x402 設定 |
| `X402_NETWORK` | `eip155:84532` | x402 ネットワーク ID |
| `X402_PRIVATE_KEY` | `0x...` | x402 署名付きリクエストで利用 |
| `X402_REQUEST_TIMEOUT_MS` | `15000` | タイムアウト |
| `ZKPASSPORT_DOMAIN` | `your-mcp-server.onrender.com` | zkPassport verifier のドメイン |
| `ZKPASSPORT_VERIFIER_API_KEY` | `your-zkpassport-key` | 有効化時は backend 側と一致させる |

## デプロイ後の確認

```bash
curl -s https://your-backend.onrender.com/health
curl -s https://your-mcp-server.onrender.com/health
curl -s https://your-backend.onrender.com/.well-known/openapi.yaml | head
```

期待値:
- backend health: `{"message":"ok"}`
- MCP health: `{ "status": "ok", ... }`
