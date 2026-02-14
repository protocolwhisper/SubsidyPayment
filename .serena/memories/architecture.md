# Architecture — SubsidyPayment

## 全体構成
```
利用者 (Browser / GPT Apps)
   ↓
Frontend (React + Vite)
   ↓ REST(JSON)
Backend (Rust + Axum)
   ├─ Core API (/health, /campaigns, /proxy, /sponsored-apis ...)
   ├─ GPT API (/gpt/services, /gpt/auth, /gpt/tasks, /gpt/preferences ...)
   ├─ Middleware (CORS, GPT API key, レート制限)
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
├── gpt.rs       # GPT Actions用ハンドラ、認証、レート制限、嗜好反映
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
- モノリシック構成: `build_app()` と `build_gpt_router()` でルートを集約
- 共有状態: `SharedState(Arc<RwLock<AppState>>)` に DB/HTTP/Config/Metrics を保持
- GPT専用ガード: `verify_gpt_api_key`（`GPT_ACTIONS_API_KEY` 設定時のみ検証）
- レート制限: `RateLimiter`（デフォルト 60 req / 60s）
- x402支払い: `verify_x402_payment` → facilitator verify/settle → 支払い記録
- 可観測性: `http_requests_total`, `payment_events_total`, `creator_events_total`, `sponsor_spend_cents_total`

## API群（現行）
- Core API: 19 routes（`/health`, `/campaigns`, `/proxy/{service}/run` など）
- GPT API: 7 routes（`/gpt/services`, `/gpt/auth`, `/gpt/tasks/{campaign_id}`, `/gpt/preferences` など）
- 合計: 26 routes（`src/main.rs` の `route(...)` 定義ベース）

## DBスキーマ（主要テーブル）
- `users`
- `campaigns`（`task_schema`, `tags`, `sponsor_wallet_address` を含む）
- `task_completions`
- `payments`
- `creator_events`
- `sponsored_apis`
- `sponsored_api_calls`
- `consents`
- `gpt_sessions`
- `user_task_preferences`
