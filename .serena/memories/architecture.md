# Architecture — SubsidyPayment

## High-Level Architecture

```
User (ChatGPT / Claude / Browser)
        │
        ▼
┌─────────────────────┐
│  Frontend (React)    │  Vercel
│  - Paywall UI        │
│  - Campaign Builder  │
│  - Sponsor Dashboard │
│  - Service Discovery │
└────────┬────────────┘
         │ REST API (JSON)
         ▼
┌─────────────────────┐
│  Backend (Rust/Axum) │  Render
│  - Proxy Handler     │
│  - Campaign CRUD     │
│  - Task Completion   │
│  - Sponsored API     │
│  - x402 Payment      │
│  - Metrics           │
└────────┬────────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌──────────────┐
│PostgreSQL│ │x402 Facilitator│
│ (SQLx)  │ │ (verify/settle)│
└─────────┘ └──────────────┘
```

## Source Code Layout

```
src/
├── main.rs      # Entry point, router, ALL request handlers (~1500 lines)
├── types.rs     # AppConfig, AppState, SharedState, DB models, enums (~560 lines)
├── error.rs     # ApiError enum, IntoResponse impl (~150 lines)
├── onchain.rs   # x402 payment verification/settlement logic (~100 lines)
├── utils.rs     # Helper functions (env reading, payment utils) (~250 lines)
└── test.rs      # Integration tests (~160 lines)

frontend/src/
├── App.tsx      # Single-file React app with all components (~1800 lines)
├── main.tsx     # React DOM entry point
└── styles.css   # All styles

migrations/      # Sequential SQL migrations (0001-0006)
```

## Key Architectural Patterns

### Backend
- **Monolithic Axum server**: All handlers in `main.rs` with `build_app()` function
- **SharedState**: `Arc<RwLock<AppState>>` containing DB pool, HTTP client, config, metrics
- **x402 Proxy pattern**: Intercept 402 → show paywall → sponsor pays → return resource
- **Payment flow**: `verify_x402_payment()` → facilitator verify → settle → record payment
- **Metrics**: Prometheus counters for HTTP requests, payments, creator events, sponsor spend

### Frontend
- **Single-file React**: All UI in one `App.tsx` (campaign builder, dashboard, paywall, discovery)
- **No routing library**: Tab-based navigation via state
- **No state management library**: `useState` hooks only
- **Dark mode**: Toggle via state, CSS variables

### Database
- **PostgreSQL** via SQLx (compile-time checked queries)
- **Tables**: profiles, campaigns, task_completions, payments, creator_events, sponsored_apis
- **UUID v4** primary keys, `created_at` timestamps

## API Endpoints (20 total)

### Core
| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Health check |
| POST/GET | `/profiles` | User profile CRUD |
| POST | `/register` | User registration |

### Campaigns
| Method | Path | Purpose |
|--------|------|---------|
| POST/GET | `/campaigns` | Campaign CRUD |
| GET | `/campaigns/discovery` | Campaign search/browse |
| GET | `/campaigns/{id}` | Campaign details |

### Tasks & Proxy
| Method | Path | Purpose |
|--------|------|---------|
| POST | `/tasks/complete` | Mark task as completed |
| POST | `/tool/{service}/run` | Run a tool service |
| POST | `/proxy/{service}/run` | Proxy request to upstream |

### Sponsored APIs
| Method | Path | Purpose |
|--------|------|---------|
| POST/GET | `/sponsored-apis` | Sponsored API CRUD |
| GET | `/sponsored-apis/{id}` | Sponsored API details |
| POST | `/sponsored-apis/{id}/run` | Execute sponsored API |

### Webhooks & Metrics
| Method | Path | Purpose |
|--------|------|---------|
| POST | `/webhooks/x402scan/settlement` | x402 settlement webhook |
| GET | `/dashboard/sponsor/{id}` | Sponsor dashboard data |
| POST | `/creator/metrics/event` | Record creator metric |
| GET | `/creator/metrics` | Get creator metrics |
| GET | `/metrics` | Prometheus metrics |

## External Dependencies
- **x402 Facilitator** (`https://x402.org/facilitator`): Payment verify & settle
- **Base Sepolia**: Blockchain network for USDC payments
- **Vercel**: Frontend hosting
- **Render**: Backend hosting (render.yaml)
