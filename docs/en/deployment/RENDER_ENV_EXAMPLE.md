# Render Environment Variable Examples (Current)

This page reflects the current implementation as of 2026-02-25.

## 1. Rust backend service (`payloadexchange-backend`)

### Required

| Key | Example value | Notes |
|---|---|---|
| `DATABASE_URL` | `postgres://...` | Use Render Postgres **Internal Database URL** |
| `PUBLIC_BASE_URL` | `https://payloadexchange-backend.onrender.com` | Public backend URL |

### Recommended

| Key | Example value | Notes |
|---|---|---|
| `RUST_LOG` | `payloadexchange_mvp=info,tower_http=info` | Logging |
| `CORS_ALLOW_ORIGINS` | `https://your-frontend.vercel.app,https://chatgpt.com,https://chat.openai.com` | Comma-separated |
| `MCP_SERVER_URL` | `https://your-mcp-server.onrender.com` | Added to CORS allow list |
| `X402_FACILITATOR_URL` | `https://x402.org/facilitator` | x402 config |
| `X402_VERIFY_PATH` | `/verify` | x402 config |
| `X402_SETTLE_PATH` | `/settle` | x402 config |
| `X402_NETWORK` | `base-sepolia` | x402 config |
| `SPONSORED_API_CREATE_PRICE_CENTS` | `25` | Sponsored API defaults |
| `SPONSORED_API_TIMEOUT_SECS` | `12` | Upstream timeout |
| `GPT_ACTIONS_API_KEY` | `your-secret-api-key` | Enables GPT API key auth when set |
| `AGENT_DISCOVERY_API_KEY` | `your-agent-api-key` | Optional discovery auth |
| `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` | `120` | Discovery rate limit |
| `ZKPASSPORT_VERIFIER_URL` | `https://your-mcp-server.onrender.com/internal/zkpassport/verify` | Verifier endpoint |
| `ZKPASSPORT_VERIFIER_API_KEY` | `your-zkpassport-key` | Must match MCP verifier key if enabled |
| `ZKPASSPORT_VERIFY_PAGE_URL` | `https://payloadexchange-backend.onrender.com/verify/zkpassport` | Public verify page |
| `ZKPASSPORT_SCOPE` | `snapfuel-gpt-age-country-v1` | Scope name |
| `ZKPASSPORT_VERIFICATION_TTL_SECS` | `900` | Token TTL |
| `ZKPASSPORT_HASH_SALT` | `replace-in-production` | Hash salt for unique identifier |

## 2. MCP server service (`subsidypayment-mcp-server`)

### Required

| Key | Example value | Notes |
|---|---|---|
| `RUST_BACKEND_URL` | `https://payloadexchange-backend.onrender.com` | Backend base URL |
| `PUBLIC_URL` | `https://your-mcp-server.onrender.com` | MCP public URL |

### Optional / feature-dependent

| Key | Example value | Notes |
|---|---|---|
| `AUTH0_DOMAIN` | `your-tenant.auth0.com` | OAuth |
| `AUTH0_AUDIENCE` | `https://api.your-domain` | OAuth |
| `AUTH_ENABLED` | `true` or `false` | Overrides auto-detection |
| `MCP_INTERNAL_API_KEY` | `your-mcp-internal-api-key` | Sent as Bearer to backend |
| `FRONTEND_URL` | `https://your-frontend.vercel.app` | Used in tool-generated links |
| `LOG_LEVEL` | `info` | Logger level |
| `X402_WEATHER_URL` | `https://your-x402-sample/weather` | Weather tool |
| `X402_GITHUB_ISSUE_URL` | `https://your-x402-sample/github-issue` | Github issue tool |
| `X402_FACILITATOR_URL` | `https://x402.org/facilitator` | x402 config |
| `X402_NETWORK` | `eip155:84532` | x402 network ID |
| `X402_PRIVATE_KEY` | `0x...` | Required for x402 request signing flows |
| `X402_REQUEST_TIMEOUT_MS` | `15000` | Timeout |
| `ZKPASSPORT_DOMAIN` | `your-mcp-server.onrender.com` | zkPassport verifier domain |
| `ZKPASSPORT_VERIFIER_API_KEY` | `your-zkpassport-key` | Must match backend verifier key if enabled |

## Quick validation after deploy

```bash
curl -s https://your-backend.onrender.com/health
curl -s https://your-mcp-server.onrender.com/health
curl -s https://your-backend.onrender.com/.well-known/openapi.yaml | head
```

Expected:
- Backend health: `{"message":"ok"}`
- MCP health: `{ "status": "ok", ... }`
