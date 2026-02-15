# Task Completion Checklist

## バックエンド（Rust）
```bash
cargo fmt
cargo clippy -- -D warnings
cargo check
cargo test
```

## フロントエンド（TypeScript / React）
```bash
cd frontend && npx tsc --noEmit
cd frontend && npm run build
```

## DB変更がある場合
```bash
sqlx migrate run
sqlx migrate info
```

## API変更がある場合
```bash
rg -n "route\\(" src/main.rs
rg -n "/gpt/|/agent/discovery|/claude/discovery|/openclaw/discovery" openapi.yaml
```

## 最終確認
- [ ] 秘密情報をハードコードしていない
- [ ] `ApiError` でエラー経路を統一している
- [ ] 追加APIのCORS/認証/レート制限要件を満たす
- [ ] `openapi.yaml` と実装ルートに齟齬がない
- [ ] 既存テストが通る
- [ ] コミットメッセージが Conventional Commits 準拠
