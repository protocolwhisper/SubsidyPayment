#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Cleanup sponsor/service demo campaigns directly in Postgres.

This script removes demo campaigns and dependent rows:
- gpt_service_runs
- payments
- task_completions
- consents
- campaigns

Usage:
  DATABASE_URL=<postgres_url> bash test_data/cleanup_sponsor_service_demo.sh [--run-tag <TAG>] [--all-demo]

Options:
  --run-tag <TAG>   Delete only campaigns with name containing [TAG]
  --all-demo        Delete all campaigns matching known demo sponsors/names
  --database-url    Optional override for DATABASE_URL

Notes:
  - Requires psql installed.
  - This does NOT delete users.
USAGE
}

if ! command -v psql >/dev/null 2>&1; then
  echo "Error: psql is required." >&2
  exit 1
fi

DATABASE_URL="${DATABASE_URL:-}"
RUN_TAG="${RUN_TAG:-}"
ALL_DEMO="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --database-url)
      DATABASE_URL="${2:-}"
      shift 2
      ;;
    --run-tag)
      RUN_TAG="${2:-}"
      shift 2
      ;;
    --all-demo)
      ALL_DEMO="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$DATABASE_URL" ]]; then
  echo "Error: DATABASE_URL (or --database-url) is required." >&2
  usage
  exit 1
fi

if [[ "$ALL_DEMO" != "true" && -z "$RUN_TAG" ]]; then
  echo "Error: pass either --run-tag <TAG> or --all-demo." >&2
  usage
  exit 1
fi

echo "== cleanup sponsor/service demo data =="
if [[ "$ALL_DEMO" == "true" ]]; then
  echo "Mode: all demo campaigns by known sponsors/names"
else
  echo "Mode: run tag [${RUN_TAG}]"
fi

if [[ "$ALL_DEMO" == "true" ]]; then
  WHERE_CLAUSE="
(
  c.sponsor in (
    'AgentCon',
    'QuickNode',
    'BuilderJobs',
    'Rain Cards',
    'Kraken',
    'Creator Alpha',
    'AI Market Lab',
    'DataLabel Network',
    'UGC Growth Studio',
    'GeoInfra Labs',
    'Base Ecosystem'
  )
  or c.name like 'AgentCon Hackathon Boost [%'
  or c.name like 'QuickNode DevRel Onboarding [%'
  or c.name like 'BuilderJobs Talent Funnel [%'
  or c.name like 'Rain Card Activation Growth [%'
  or c.name like 'Kraken New User Conversion [%'
  or c.name like 'Creator Video Amplifier [%'
  or c.name like 'AI Market Survey Program [%'
  or c.name like 'DataLabel Human Annotation Drive [%'
  or c.name like 'UGC Growth Sprint [%'
  or c.name like 'Infra Field Ops Program [%'
  or c.name like 'Base Experience Review Rewards [%'
)"
else
  if [[ ! "$RUN_TAG" =~ ^[0-9A-Za-z_-]+$ ]]; then
    echo "Error: --run-tag may only contain letters, numbers, _ and -." >&2
    exit 1
  fi
  WHERE_CLAUSE="c.name like '%[${RUN_TAG}]%'"
fi

read -r -d '' SQL <<SQL || true
begin;

create temp table _demo_target_campaigns as
select c.id
from campaigns c
where ${WHERE_CLAUSE};

select count(*) as target_campaigns from _demo_target_campaigns;

delete from gpt_service_runs where campaign_id in (select id from _demo_target_campaigns);
delete from payments where campaign_id in (select id from _demo_target_campaigns);
delete from task_completions where campaign_id in (select id from _demo_target_campaigns);
delete from consents where campaign_id in (select id from _demo_target_campaigns);
delete from campaigns where id in (select id from _demo_target_campaigns);

commit;
SQL

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -c "$SQL"

echo "Cleanup completed."
