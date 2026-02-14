# Suggested Commands

## バックエンド（Rust / Axum）
```bash
cargo check
cargo build
cargo run
cargo test
cargo fmt
cargo clippy
```

## フロントエンド（React / Vite）
```bash
cd frontend && npm install
cd frontend && npm run dev
cd frontend && npm run build
cd frontend && npm run preview
```

## DB（PostgreSQL）
```bash
docker compose -f docker-compose.postgres.yml up -d
docker compose -f docker-compose.postgres.yml down
sqlx migrate run
sqlx migrate info
```

## API確認
```bash
curl -s http://localhost:3000/health
curl -s http://localhost:3000/.well-known/openapi.yaml | head
curl -s "http://localhost:3000/gpt/services?q=design"
```

## リポジトリ探索
```bash
git status --short
git log -n 10 --oneline
git diff
rg --files
rg -n "route\\(" src/main.rs
rg -n "gpt_" src/gpt.rs src/types.rs
```
