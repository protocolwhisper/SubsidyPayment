# Suggested Commands

## Backend (Rust / Axum)

```bash
# Build
cargo build

# Run backend server (default port 3000)
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Check (fast compile check without producing binary)
cargo check
```

## Frontend (React / Vite / TypeScript)

```bash
# Install dependencies
cd frontend && npm install

# Dev server
cd frontend && npm run dev

# Production build
cd frontend && npm run build

# Preview production build
cd frontend && npm run preview
```

## Database (PostgreSQL)

```bash
# Start local PostgreSQL via Docker
docker compose -f docker-compose.postgres.yml up -d

# Stop PostgreSQL
docker compose -f docker-compose.postgres.yml down

# Run migrations (via sqlx-cli)
sqlx migrate run

# Check migration status
sqlx migrate info
```

## Documentation (GitBook)

```bash
cd docs && npx gitbook serve
```

## Deployment

```bash
# Frontend → Vercel
cd frontend && vercel --prod

# Backend → Render (configured via render.yaml)
# Push to main branch triggers auto-deploy
```

## Environment Setup

```bash
# Copy env template
cp .env.example .env
# Edit .env with your values
```

## System Utilities (macOS / Darwin)

```bash
# Git
git status
git log -n 10 --oneline
git diff
git add -A && git commit -m "feat: description"

# File search
find . -name "*.rs" -not -path "./target/*"
grep -r "pattern" src/

# Process management
lsof -i :3000   # Check what's using port 3000
```

## x402 Testing

```bash
# Run x402 test script
bash scripts/x402.sh
```
