# Task Completion Checklist（更新: 2026-02-20）

## Rust 変更時
```bash
cargo fmt
cargo clippy -- -D warnings
cargo check
cargo test
```

## Frontend 変更時
```bash
cd frontend && npm run build
```

## MCP サーバー変更時
```bash
cd mcp-server && npm run typecheck
cd mcp-server && npm test
cd mcp-server && npm run build
```

## DB 変更時
```bash
sqlx migrate info
sqlx migrate run
```

## API/契約変更時
```bash
rg -n "route\\(" src/main.rs
rg -n "/gpt/|/agent/discovery|/claude/discovery|/openclaw/discovery" openapi.yaml
```

## .kiro/specs 運用確認
- 変更対象 feature の `requirements -> design -> tasks` 承認状態を確認
- `spec.json.language`（現状は ja）に合わせてMarkdownを記述
- 完了タスクのチェック状態と実装内容が一致しているか確認

## 最終確認
- [ ] 秘密情報をハードコードしていない
- [ ] 追加エンドポイントに認証/CORS/レート制限を適用した
- [ ] `openapi.yaml` と実装ルートに齟齬がない
- [ ] Migrationと型定義・テストが同期している
- [ ] 既存E2E/ユニットテストを壊していない
- [ ] Conventional Commitsに沿ったコミットメッセージ