# Architecture — SubsidyPayment（更新: 2026-02-20）

## システム全体
```
Client (Web / GPT Apps / Claude系)
  ↓
Frontend (React + Vite)
  ↓ REST(JSON)
Rust Backend (Axum)
  ├─ Core API
  ├─ GPT API (/gpt/*)
  ├─ Discovery API (/agent|/claude|/openclaw/discovery/*)
  ├─ OpenAPI / Privacy / Metrics
  └─ x402 verify/settle 連携
  ↓
PostgreSQL (SQLx migrations 0001-0013)

MCP Server (Node + Express + MCP SDK)
  ├─ /mcp (Streamable HTTP)
  ├─ OAuth metadata endpoints
  ├─ Tools: search/auth/task/run/user/preferences
  └─ Widgets: services-list, service-tasks, task-form, service-access, user-dashboard
  ↓
Rust /gpt/* API へHTTP委譲
```

## Rust バックエンド設計
- ルーティングは `src/main.rs` に集約（モノリシック）
- 共有状態: `SharedState(Arc<RwLock<AppState>>)`
- エラー統一: `ApiError` / `ApiResult<T>`
- x402 支払い: verify/settle 実行 + DB記録
- メトリクス: Prometheus（HTTP、決済、クリエイター指標）

## MCP サーバー設計
- `mcp-server/src/main.ts` で Express アプリと CORS を構成
- `/mcp` は GET/POST/DELETE をハンドリング
- `createServer()` で全ツール・全ウィジェットを登録
- Auth0ベースOAuthは `AUTH_ENABLED`/環境変数で有効化切替
- `search_services` は noauth、他ツールは OAuth/noauth 切替可能設計

## DB スキーマ状況
- マイグレーションは `0013_gpt_service_runs.sql` まで存在
- 主要テーブル:
  - `users`, `campaigns`, `task_completions`, `payments`, `creator_events`
  - `sponsored_apis`, `sponsored_api_calls`
  - `consents`, `gpt_sessions`, `user_task_preferences`, `gpt_service_runs`

## 重要な実装ポイント
- `/gpt/preferences`（GET/POST）で嗜好管理
- Discovery API は3プレフィックスで同一ロジック提供
- `.well-known/openapi.yaml` と実装ルートの整合が重要