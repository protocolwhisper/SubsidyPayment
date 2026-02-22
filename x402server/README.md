# Sample x402 server & client code

## setup

```bash
cp .env.example .env
```

```bash
pnpm install
```

## start backend

```bash
pnpm run dev
```

## run demo client

```bash
pnpm run demo
```

## MCP Server test

```bash
# Start Rust backend server
cargo run

cd x402server
pnpm run dev

cd ../mcp-server
# start mcp server
npm run dev

# start MCP inspector
npx @modelcontextprotocol/inspector

# ngrok(test for GPT App)
ngrok http 3001
```

if you don't setup local db.  
Please run these commands

```bash
docker compose -f docker-compose.postgres.yml up -d

source .env
sqlx migrate run
# Check
sqlx migrate info
```