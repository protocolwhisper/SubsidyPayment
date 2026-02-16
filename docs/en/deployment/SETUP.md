# Local Environment Setup

## Prerequisites
- Docker Desktop is installed and running
- Rust and Cargo are installed

## Setup Steps

### 1. Start Docker Desktop

Launch the Docker Desktop application.

### 2. Start Postgres container

```bash
cd /path/to/SubsidyPayment
docker compose -f docker-compose.postgres.yml up -d
```

### 3. Start backend server

Run in a new terminal:

```bash
cd /path/to/SubsidyPayment
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000
RUST_LOG=info cargo run
```

Or use script:

```bash
./scripts/start-backend.sh
```

### 4. Start MCP server

Run in a new terminal:

```bash
cd /path/to/SubsidyPayment/mcp-server
npm install
npm run dev
```

#### Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `RUST_BACKEND_URL` | Yes | Rust backend URL (default: `http://localhost:3000`) |
| `MCP_INTERNAL_API_KEY` | Yes | API key for backend communication |
| `AUTH0_DOMAIN` | No | Auth0 tenant domain (e.g. `your-tenant.auth0.com`) |
| `AUTH0_AUDIENCE` | No | Auth0 API audience |
| `PUBLIC_URL` | Yes | MCP server public URL (default: `http://localhost:3001`) |
| `AUTH_ENABLED` | No | Toggle OAuth authentication (`true`/`false`). When omitted, auto-detected from Auth0 settings |

#### AUTH_ENABLED behavior

| `AUTH_ENABLED` | `AUTH0_DOMAIN` + `AUTH0_AUDIENCE` | Result |
|---|---|---|
| `true` | any | OAuth **enabled** |
| `false` / `0` / `no` | any | OAuth **disabled** |
| not set | both present | OAuth **enabled** |
| not set | either missing | OAuth **disabled** |

For local development without Auth0, start with:

```bash
AUTH_ENABLED=false npm run dev
```

The startup log will show `OAuth authentication is DISABLED (MVP mode)`.

### 5. Frontend server

Frontend should be available at `http://localhost:5173`.

## Verification

1. Open `http://localhost:5173` in browser
2. Sign in and click `Create Campaign`
3. Fill form and click `Create Campaign`
4. If no error occurs, setup is working

## Troubleshooting

### Cannot connect to Postgres

- Confirm container is running: `docker ps`
- Check container logs: `docker logs payloadexchange-postgres`

### Backend server does not start

- Confirm `DATABASE_URL` is correct
- Confirm Postgres container is running
- Confirm port `3000` is not already used
