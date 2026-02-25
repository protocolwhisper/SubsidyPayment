# Local Environment Setup

This document reflects the current implementation as of 2026-02-25.

## Prerequisites
- Docker Desktop is installed and running
- Rust/Cargo installed
- Node.js 20+ installed
- `npm` and `pnpm` available

## 1. Start PostgreSQL

```bash
cd /path/to/SubsidyPayment
docker compose -f docker-compose.postgres.yml up -d
```

## 2. Start Rust backend

```bash
cd /path/to/SubsidyPayment
export DATABASE_URL=postgres://postgres:postgres@localhost:55432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000
export RUST_LOG=payloadexchange_mvp=info,tower_http=info
cargo run
```

Or:

```bash
./scripts/start-backend.sh
```

### Backend env vars (main ones)

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | Yes | PostgreSQL connection URL |
| `PUBLIC_BASE_URL` | Yes | Public backend base URL |
| `X402_FACILITATOR_URL` | Recommended | x402 facilitator URL |
| `X402_VERIFY_PATH` / `X402_SETTLE_PATH` | Recommended | x402 verify/settle paths |
| `GPT_ACTIONS_API_KEY` | Optional | Enables GPT API-key auth when set |
| `AGENT_DISCOVERY_API_KEY` | Optional | Enables Bearer auth for discovery aliases when set |
| `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` | Optional | Shared discovery rate limit |
| `ZKPASSPORT_VERIFIER_URL` | Recommended for zkPassport | Verifier endpoint (default points to MCP server) |
| `ZKPASSPORT_VERIFIER_API_KEY` | Optional | Verifier API key |
| `CORS_ALLOW_ORIGINS` | Recommended | Comma-separated allowed origins |
| `MCP_SERVER_URL` | Recommended | MCP server origin added to CORS allow list |

## 3. Start MCP server

```bash
cd /path/to/SubsidyPayment/mcp-server
npm install
npm run dev
```

For local MVP mode without Auth0:

```bash
AUTH_ENABLED=false npm run dev
```

### MCP env vars

| Variable | Required | Description |
|---|---|---|
| `RUST_BACKEND_URL` | Yes | Rust backend URL (`http://localhost:3000`) |
| `PUBLIC_URL` | Yes | MCP server public URL (`http://localhost:3001`) |
| `AUTH0_DOMAIN` | Optional | Auth0 domain |
| `AUTH0_AUDIENCE` | Optional | Auth0 audience |
| `AUTH_ENABLED` | Optional | Force OAuth on/off (`true`/`false`) |
| `MCP_INTERNAL_API_KEY` | Optional | Internal API key header sent to backend |
| `FRONTEND_URL` | Optional | Frontend URL for links from MCP tools |
| `X402_WEATHER_URL` | Optional | x402 sample weather endpoint |
| `X402_GITHUB_ISSUE_URL` | Optional | x402 sample github-issue endpoint |
| `X402_FACILITATOR_URL` | Optional | x402 facilitator URL |
| `X402_NETWORK` | Optional | x402 network ID (`eip155:84532`) |
| `X402_PRIVATE_KEY` | Optional | Private key used by x402 request flows |
| `X402_REQUEST_TIMEOUT_MS` | Optional | Timeout for x402 HTTP requests |
| `ZKPASSPORT_DOMAIN` | Optional | Domain passed to zkPassport SDK verifier |
| `ZKPASSPORT_VERIFIER_API_KEY` | Optional | API key for `/internal/zkpassport/verify` |

## 4. Start frontend

```bash
cd /path/to/SubsidyPayment/frontend
npm install
npm run dev
```

Frontend URL: `http://localhost:5173`

## 5. (Optional) Start x402 sample server

```bash
cd /path/to/SubsidyPayment/x402server
pnpm install
pnpm run dev
```

Sample server URL: `http://localhost:4021`

## Verification checklist

```bash
curl -s http://localhost:3000/health
curl -s "http://localhost:3000/gpt/services?q=design"
curl -s "http://localhost:3000/agent/discovery/services?q=design"
curl -s http://localhost:3001/health
```

Expected:
- Backend `/health` returns `{"message":"ok"}`
- MCP `/health` returns `{ "status": "ok", ... }`

## Troubleshooting

### Postgres connection errors
- Check `docker ps`
- Check `docker logs payloadexchange-postgres`
- Confirm `DATABASE_URL` port matches compose config (`55432` in `.env.example`)

### MCP OAuth errors in local
- Set `AUTH_ENABLED=false` for local MVP mode
- Or set valid `AUTH0_DOMAIN` and `AUTH0_AUDIENCE`

### zkPassport verification failures
- Ensure MCP server route `/internal/zkpassport/verify` is running
- Ensure Rust `ZKPASSPORT_VERIFIER_URL` points to MCP server URL
- Ensure verifier API keys match on both sides if enabled
