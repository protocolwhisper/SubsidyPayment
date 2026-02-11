# Style and Conventions

## General
- Think in English, write project files (docs, specs, requirements) in Japanese (per AGENTS.md)
- Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`
- Boy Scout Rule: Leave the code better than you found it

## Rust Backend

### Code Style
- Follow `rustfmt` defaults
- Follow `clippy` recommendations
- Edition 2024

### Patterns
- **Error handling**: `thiserror`-based `ApiError` enum with `IntoResponse` impl for HTTP status mapping
- **Type alias**: `ApiResult<T> = Result<T, ApiError>` for all handler return types
- **State**: `SharedState` wrapping `Arc<RwLock<AppState>>` — shared across all handlers
- **Config**: `AppConfig::from_env()` reads all config from environment variables with sensible defaults
- **Metrics**: Prometheus counters/gauges registered in `Metrics` struct
- **DB**: SQLx with raw SQL queries (not an ORM), using `query_as!` and `query_scalar!` macros
- **Monolithic structure**: All handlers in `main.rs`, types in `types.rs`, errors in `error.rs`

### Naming
- snake_case for functions, variables, modules
- PascalCase for types, enums, structs
- SCREAMING_SNAKE_CASE for constants
- HTTP headers as lowercase constants: `PAYMENT_SIGNATURE_HEADER`, `X402_VERSION_HEADER`

### Error Responses
- Always return structured JSON errors with descriptive messages
- Use `ApiError` variants: `NotFound`, `BadRequest`, `Unauthorized`, `Internal`, `PaymentRequired`, etc.

## TypeScript Frontend

### Code Style
- React 18 with functional components and hooks
- Single-file architecture: main app logic in `App.tsx`
- Vite for build tooling
- No CSS framework — custom CSS in `styles.css`
- TypeScript strict mode

### Patterns
- Type definitions at top of `App.tsx` (Campaign, Profile, SponsoredApi, etc.)
- Constants for configuration (SERVICE_CATEGORIES, TASK_CATEGORIES, KPI_OPTIONS)
- `fetchJson` utility for API calls
- State management via `useState` hooks (no external state library)

## Database
- Raw SQL migrations in `migrations/` directory (sequential numbering: 0001, 0002, ...)
- PostgreSQL with SQLx
- UUID v4 for primary keys
- `created_at` timestamps with `chrono::DateTime<Utc>`
- JSON columns via `sqlx::types::Json<T>`

## API Design
- RESTful endpoints
- JSON request/response bodies
- CORS configured via `CORS_ALLOW_ORIGINS` env var (defaults to `*`)
- x402 payment headers: `payment-signature`, `payment-required`, `payment-response`, `x402-version`
