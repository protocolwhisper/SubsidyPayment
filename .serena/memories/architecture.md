# Architecture — SubsidyPayment

## 全体構成
```
利用者 (Browser / GPT Apps / Claude / OpenClaw)
   ↓
Frontend (React + Vite)
   ↓ REST(JSON)
Backend (Rust + Axum)
   ├─ Core API (/health, /campaigns, /proxy, /sponsored-apis ...)
   ├─ GPT API (/gpt/services, /gpt/auth, /gpt/tasks/*, /gpt/preferences ...)
   ├─ Agent Discovery API
   │   (/agent/discovery/services, /claude/discovery/services, /openclaw/discovery/services)
   ├─ Middleware (CORS, GPT API key, Agent Discovery API key, レート制限)
   └─ Metrics (Prometheus)
   ↓
PostgreSQL (SQLx migrations 0001-0012)
   ↓
x402 Facilitator (verify / settle)
```

## ソース構造（現行）
```
src/
├── main.rs      # ルータ構築、主要ハンドラ、起動処理
├── gpt.rs       # GPT API + Discovery向けロジック、認証、レート制限
├── types.rs     # API型、AppState/AppConfig、DB行マッピング
├── error.rs     # ApiError定義とHTTP応答変換
├── onchain.rs   # x402 verify/settle 実行
├── utils.rs     # 共通レスポンス、支払い関連ユーティリティ
└── test.rs      # API統合テスト

frontend/src/
├── App.tsx
├── main.tsx
└── styles.css
```

## バックエンド設計の要点
- モノリシック構成: `build_app()` / `build_gpt_router()` / `build_agent_discovery_router()` でルートを集約
- 共有状態: `SharedState(Arc<RwLock<AppState>>)` に DB/HTTP/Config/Metrics を保持
- GPT専用ガード: `verify_gpt_api_key`（`GPT_ACTIONS_API_KEY` 設定時のみ検証）
- Agent Discovery ガード: `verify_agent_discovery_api_key`（`AGENT_DISCOVERY_API_KEY` 設定時のみ検証）
- レート制限: `RateLimiter`（GPTは 60 req/60s、Discoveryは `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN`）
- x402支払い: `verify_x402_payment` → facilitator verify/settle → 支払い記録
- 可観測性: `http_requests_total`, `payment_events_total`, `creator_events_total`, `sponsor_spend_cents_total`

## API群（現行）
- Core API: 19 routes（`/health`, `/campaigns`, `/proxy/{service}/run` など）
- GPT API: 7 routes（`/gpt/services`, `/gpt/auth`, `/gpt/tasks/{campaign_id}/complete`, `/gpt/preferences` など）
- Agent Discovery API: 1 route を 3 プレフィックスで公開（`/services`）
- 合計: `route(...)` 定義 27 件（`src/main.rs` ベース）

## DBスキーマ（主要テーブル）
- `users`
- `campaigns`（`task_schema`, `tags`, `sponsor_wallet_address`, `user_source` を含む）
- `task_completions`
- `payments`
- `creator_events`
- `sponsored_apis`
- `sponsored_api_calls`
- `consents`
- `gpt_sessions`
- `user_task_preferences`
