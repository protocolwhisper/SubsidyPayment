# Agent Discovery Contract

Schema version: `2026-02-14`
Endpoint: `GET /agent/discovery/services`
Aliases: `GET /claude/discovery/services`, `GET /openclaw/discovery/services`

## Required metadata fields
- `capabilities`
- `price_cents`
- `sla`
- `required_task`
- `sponsor`

## Capability normalization
The backend canonicalizes capability names before ranking and returning metadata:
- `scrape`, `web-scrape`, `web-scraping` -> `scraping`
- `ui-design`, `designing` -> `design`
- `data-tool`, `data-tools` -> `data-tooling`
- underscores/spaces are converted to hyphens

## SLA tier values
- `best_effort`
- `standard`

## Ranking signals
`ranking_score` is derived from:
- `subsidy_score`
- `budget_health_score`
- `relevance_score`

## Auth and rate limit
- Optional bearer auth via `AGENT_DISCOVERY_API_KEY`
- Shared in-memory rate limit via `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN`

## Stability notes
1. Treat `schema_version` as the compatibility key.
2. If `schema_version` changes, adapters should validate field compatibility before rollout.
3. Unknown capabilities should be handled as non-fatal in agents.
