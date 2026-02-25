# Postgres Schema (Current)

This page reflects the current SQLx migration state as of 2026-02-25.

## Migration baseline

- Migration directory: `migrations/`
- Latest migration: `0014_zkpassport_verifications.sql`
- Migrations are auto-applied at backend startup (`sqlx::migrate!("./migrations")`)

## How to run migrations manually

```bash
sqlx migrate info
sqlx migrate run
```

Required env:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:55432/payloadexchange
```

## Current tables

| Table | Added in | Purpose |
|---|---|---|
| `users` | `0001` | End users / GPT users |
| `sponsored_apis` | `0001` | Sponsored API definitions |
| `sponsored_api_calls` | `0001` | Sponsored API call logs |
| `campaigns` | `0002` | Sponsor campaign master |
| `task_completions` | `0003` | Completed campaign tasks |
| `payments` | `0004` | Payment settlement records |
| `creator_events` | `0005` | Creator-side telemetry events |
| `consents` | `0007` | Consent records for campaign/task actions |
| `gpt_sessions` | `0009` | GPT session tokens |
| `user_task_preferences` | `0011` | Task preference profiles |
| `gpt_service_runs` | `0013` | GPT service execution history |
| `zkpassport_verifications` | `0014` | zkPassport verification lifecycle |

## Incremental schema changes (no new tables)

- `0006`: adds `campaigns.sponsor_wallet_address`
- `0008`: adds `users.source` (default: `web`)
- `0010`: adds `campaigns.task_schema` (`jsonb`)
- `0012`: adds `campaigns.tags` (`text[]`)

## Notes

- There is no separate `profiles` table; profile data is stored in `users`.
- Campaign tags are stored as an array column on `campaigns`, not a `campaign_tags` table.
- If you update schema, keep `src/types.rs`, API handlers, and tests in sync with migration changes.
