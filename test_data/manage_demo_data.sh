#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'USAGE'
Manage demo data in one command: cleanup + seed.

Examples:
  # sponsor/service demo only
  bash test_data/manage_demo_data.sh --api-key <KEY> --base-url <URL>

  # cleanup this run tag first, then seed sponsor demo
  DATABASE_URL=<postgres_url> bash test_data/manage_demo_data.sh \
    --api-key <KEY> --base-url <URL> --clean-first --run-tag 20260218133034

  # cleanup all known demo campaigns, then seed both scripts
  DATABASE_URL=<postgres_url> bash test_data/manage_demo_data.sh \
    --api-key <KEY> --base-url <URL> --clean-first --all-demo --with-mock

Options:
  --api-key <KEY>      Backend bearer key (required for populate)
  --base-url <URL>     Backend URL (default: https://subsidypayment-1k0h.onrender.com)
  --run-tag <TAG>      Tag for sponsor demo seed and/or cleanup targeting
  --clean-first        Run cleanup before populating
  --all-demo           With --clean-first, remove all known demo campaigns
  --with-mock          Also run populate_mock_data.sh after sponsor demo seed
  --database-url <URL> Postgres URL for cleanup (or use DATABASE_URL env)
USAGE
}

API_KEY="${API_KEY:-}"
BASE_URL="${BASE_URL:-https://subsidypayment-1k0h.onrender.com}"
RUN_TAG="${RUN_TAG:-}"
CLEAN_FIRST="false"
ALL_DEMO="false"
WITH_MOCK="false"
DATABASE_URL="${DATABASE_URL:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --api-key)
      API_KEY="${2:-}"
      shift 2
      ;;
    --base-url)
      BASE_URL="${2:-}"
      shift 2
      ;;
    --run-tag)
      RUN_TAG="${2:-}"
      shift 2
      ;;
    --clean-first)
      CLEAN_FIRST="true"
      shift
      ;;
    --all-demo)
      ALL_DEMO="true"
      shift
      ;;
    --with-mock)
      WITH_MOCK="true"
      shift
      ;;
    --database-url)
      DATABASE_URL="${2:-}"
      shift 2
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

if [[ -z "$API_KEY" ]]; then
  echo "Error: --api-key is required." >&2
  usage
  exit 1
fi

if [[ "$CLEAN_FIRST" == "true" ]]; then
  if [[ -z "$DATABASE_URL" ]]; then
    echo "Error: cleanup requested but DATABASE_URL/--database-url is missing." >&2
    exit 1
  fi

  if [[ "$ALL_DEMO" == "true" ]]; then
    bash "${SCRIPT_DIR}/cleanup_sponsor_service_demo.sh" \
      --database-url "$DATABASE_URL" \
      --all-demo
  else
    if [[ -z "$RUN_TAG" && -f "${SCRIPT_DIR}/.last_demo_run_tag" ]]; then
      RUN_TAG="$(cat "${SCRIPT_DIR}/.last_demo_run_tag")"
      echo "Using last run tag from file: ${RUN_TAG}"
    fi
    if [[ -z "$RUN_TAG" ]]; then
      echo "Error: cleanup requested without --all-demo and no --run-tag available." >&2
      exit 1
    fi

    bash "${SCRIPT_DIR}/cleanup_sponsor_service_demo.sh" \
      --database-url "$DATABASE_URL" \
      --run-tag "$RUN_TAG"
  fi
fi

SPONSOR_ARGS=(--api-key "$API_KEY" --base-url "$BASE_URL")
if [[ -n "$RUN_TAG" ]]; then
  SPONSOR_ARGS+=(--run-tag "$RUN_TAG")
fi

bash "${SCRIPT_DIR}/populate_sponsor_service_demo.sh" "${SPONSOR_ARGS[@]}"

if [[ "$WITH_MOCK" == "true" ]]; then
  bash "${SCRIPT_DIR}/populate_mock_data.sh" \
    --api-key "$API_KEY" \
    --base-url "$BASE_URL"
fi

echo
echo "Done."
