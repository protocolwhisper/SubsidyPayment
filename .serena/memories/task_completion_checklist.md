# Task Completion Checklist

When a coding task is completed, run through this checklist before reporting done.

## Backend (Rust)

```bash
# 1. Format
cargo fmt

# 2. Lint
cargo clippy -- -D warnings

# 3. Build check
cargo check

# 4. Tests
cargo test
```

## Frontend (TypeScript / React)

```bash
# 1. Type check
cd frontend && npx tsc --noEmit

# 2. Build
cd frontend && npm run build
```

## Database Changes

If migrations were added or modified:
```bash
# 1. Verify migration applies cleanly
sqlx migrate run

# 2. Verify migration info
sqlx migrate info
```

## General

- [ ] No hardcoded secrets or API keys (use env vars)
- [ ] Error cases handled with descriptive messages
- [ ] CORS headers correct if new endpoints added
- [ ] Existing tests still pass (`cargo test`)
- [ ] New functionality has tests where appropriate
- [ ] Commit message follows Conventional Commits format
- [ ] Documentation updated if public API changed
- [ ] No unused imports or dead code left behind
