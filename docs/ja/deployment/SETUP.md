# ローカル環境セットアップ

このドキュメントは 2026-02-25 時点の実装に合わせています。

## 前提条件
- Docker Desktop がインストール済みで起動している
- Rust/Cargo がインストール済み
- Node.js 20+ がインストール済み
- `npm` / `pnpm` が利用可能

## 1. PostgreSQL を起動

```bash
cd /path/to/SubsidyPayment
docker compose -f docker-compose.postgres.yml up -d
```

## 2. Rust バックエンドを起動

```bash
cd /path/to/SubsidyPayment
export DATABASE_URL=postgres://postgres:postgres@localhost:55432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000
export RUST_LOG=payloadexchange_mvp=info,tower_http=info
cargo run
```

または:

```bash
./scripts/start-backend.sh
```

### バックエンド環境変数（主要）

| 変数名 | 必須 | 説明 |
|---|---|---|
| `DATABASE_URL` | はい | PostgreSQL 接続 URL |
| `PUBLIC_BASE_URL` | はい | 公開バックエンド URL |
| `X402_FACILITATOR_URL` | 推奨 | x402 facilitator URL |
| `X402_VERIFY_PATH` / `X402_SETTLE_PATH` | 推奨 | x402 verify/settle パス |
| `GPT_ACTIONS_API_KEY` | 任意 | 設定時のみ GPT API キー認証を有効化 |
| `AGENT_DISCOVERY_API_KEY` | 任意 | 設定時のみ Discovery API の Bearer 認証を有効化 |
| `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` | 任意 | Discovery API 共通レート制限 |
| `ZKPASSPORT_VERIFIER_URL` | zkPassport 利用時推奨 | 検証エンドポイント（既定は MCP サーバー） |
| `ZKPASSPORT_VERIFIER_API_KEY` | 任意 | 検証 API キー |
| `CORS_ALLOW_ORIGINS` | 推奨 | カンマ区切り許可オリジン |
| `MCP_SERVER_URL` | 推奨 | CORS 許可に追加する MCP サーバー origin |

## 3. MCP サーバーを起動

```bash
cd /path/to/SubsidyPayment/mcp-server
npm install
npm run dev
```

Auth0 なしのローカル MVP モード:

```bash
AUTH_ENABLED=false npm run dev
```

### MCP 環境変数

| 変数名 | 必須 | 説明 |
|---|---|---|
| `RUST_BACKEND_URL` | はい | Rust バックエンド URL（`http://localhost:3000`） |
| `PUBLIC_URL` | はい | MCP サーバー公開 URL（`http://localhost:3001`） |
| `AUTH0_DOMAIN` | 任意 | Auth0 ドメイン |
| `AUTH0_AUDIENCE` | 任意 | Auth0 audience |
| `AUTH_ENABLED` | 任意 | OAuth を強制 ON/OFF（`true`/`false`） |
| `MCP_INTERNAL_API_KEY` | 任意 | バックエンドに送る内部 API キーヘッダー |
| `FRONTEND_URL` | 任意 | MCP ツールが返すリンク先フロント URL |
| `X402_WEATHER_URL` | 任意 | x402 サンプル weather エンドポイント |
| `X402_GITHUB_ISSUE_URL` | 任意 | x402 サンプル github-issue エンドポイント |
| `X402_FACILITATOR_URL` | 任意 | x402 facilitator URL |
| `X402_NETWORK` | 任意 | x402 ネットワーク ID（`eip155:84532`） |
| `X402_PRIVATE_KEY` | 任意 | x402 リクエスト系で利用する秘密鍵 |
| `X402_REQUEST_TIMEOUT_MS` | 任意 | x402 HTTP リクエストタイムアウト |
| `ZKPASSPORT_DOMAIN` | 任意 | zkPassport SDK 検証時に使うドメイン |
| `ZKPASSPORT_VERIFIER_API_KEY` | 任意 | `/internal/zkpassport/verify` 用 API キー |

## 4. フロントエンドを起動

```bash
cd /path/to/SubsidyPayment/frontend
npm install
npm run dev
```

フロント URL: `http://localhost:5173`

## 5. （任意）x402 サンプルサーバーを起動

```bash
cd /path/to/SubsidyPayment/x402server
pnpm install
pnpm run dev
```

サンプルサーバー URL: `http://localhost:4021`

## 動作確認チェック

```bash
curl -s http://localhost:3000/health
curl -s "http://localhost:3000/gpt/services?q=design"
curl -s "http://localhost:3000/agent/discovery/services?q=design"
curl -s http://localhost:3001/health
```

期待値:
- バックエンド `/health` は `{"message":"ok"}` を返す
- MCP `/health` は `{ "status": "ok", ... }` を返す

## トラブルシューティング

### Postgres 接続エラー
- `docker ps` でコンテナ状態を確認
- `docker logs payloadexchange-postgres` を確認
- `DATABASE_URL` のポートが compose 設定（`.env.example` では `55432`）と一致しているか確認

### MCP OAuth エラー（ローカル）
- `AUTH_ENABLED=false` を設定して MVP モードで起動
- または `AUTH0_DOMAIN` / `AUTH0_AUDIENCE` を正しく設定

### zkPassport 検証エラー
- MCP サーバーの `/internal/zkpassport/verify` が起動しているか確認
- Rust 側 `ZKPASSPORT_VERIFIER_URL` が MCP サーバーを向いているか確認
- 両サーバーの verifier API キーが一致しているか確認（有効化時）
