# Suggested Commands（更新: 2026-02-20）

## バックエンド（Rust）
```bash
cargo check
cargo build
cargo run
cargo test
cargo fmt
cargo clippy -- -D warnings
```

## フロントエンド（frontend）
```bash
cd frontend && npm install
cd frontend && npm run dev
cd frontend && npm run build
cd frontend && npm run preview
```

## MCP サーバー（mcp-server）
```bash
cd mcp-server && npm install
cd mcp-server && npm run dev
cd mcp-server && npm run build
cd mcp-server && npm test
cd mcp-server && npm run typecheck
```

## DB（PostgreSQL + migration）
```bash
docker compose -f docker-compose.postgres.yml up -d
docker compose -f docker-compose.postgres.yml down
sqlx migrate info
sqlx migrate run
```

## API疎通確認
```bash
curl -s http://localhost:3000/health
curl -s "http://localhost:3000/gpt/services?q=design"
curl -s "http://localhost:3000/agent/discovery/services?q=design"
curl -s http://localhost:3000/.well-known/openapi.yaml | head
curl -s http://localhost:3001/health
```

## リポジトリ探索
```bash
git status --short
rg --files
rg -n "route\\(" src/main.rs
rg -n "registerAllTools|registerAppTool" mcp-server/src
rg -n "000[0-9]+_" migrations
```

## サンプル x402 サーバー
```bash
cd x402server && pnpm install
cd x402server && pnpm run dev
cd x402server && pnpm run demo
```