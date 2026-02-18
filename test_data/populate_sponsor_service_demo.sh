#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Populate SubsidyPayment backend with sponsor-vs-service demo data.

This script creates campaigns for distinct sponsor archetypes:
- Hackathon/conference providers
- Development companies
- Hiring platforms
- Wallet/card providers
- Centralized exchanges
- Individual creators
- Research companies
- AI data providers
- Marketing agencies
- Infrastructure companies
- Blockchain/chain companies

Usage:
  bash test_data/populate_sponsor_service_demo.sh --api-key <BEARER_KEY> [--base-url <URL>] [--run-tag <TAG>]

Or with env vars:
  API_KEY=<BEARER_KEY> BASE_URL=<URL> bash test_data/populate_sponsor_service_demo.sh

Defaults:
  BASE_URL=https://subsidypayment-1k0h.onrender.com
  RUN_TAG=current timestamp (YYYYMMDDHHMMSS)
USAGE
}

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required (brew install jq)." >&2
  exit 1
fi

BASE_URL="${BASE_URL:-https://subsidypayment-1k0h.onrender.com}"
API_KEY="${API_KEY:-}"
RUN_TAG="${RUN_TAG:-}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

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
  echo "Error: API key is required." >&2
  usage
  exit 1
fi

BASE_URL="${BASE_URL%/}"
AUTH_HEADER="Authorization: Bearer ${API_KEY}"
if [[ -z "$RUN_TAG" ]]; then
  RUN_TAG="$(date +%Y%m%d%H%M%S)"
fi

json_request() {
  local method="$1"
  local path="$2"
  local body="${3:-}"

  if [[ -n "$body" ]]; then
    curl -sS -X "$method" "${BASE_URL}${path}" \
      -H "$AUTH_HEADER" \
      -H "Content-Type: application/json" \
      -d "$body"
  else
    curl -sS -X "$method" "${BASE_URL}${path}" \
      -H "$AUTH_HEADER"
  fi
}

echo "== health =="
curl -sS "${BASE_URL}/health" | jq .
echo "== run tag: ${RUN_TAG} =="

CAMPAIGNS_JSON='[
  {
    "archetype": "hackathon_conference_provider",
    "name": "AgentCon Hackathon Boost",
    "sponsor": "AgentCon",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000001",
    "target_roles": ["builder", "developer", "founder"],
    "target_tools": ["claude-code", "vercel", "supabase"],
    "required_task": "Sign up for AgentCon hackathon and confirm registration email",
    "subsidy_per_call_cents": 40,
    "budget_cents": 250000
  },
  {
    "archetype": "development_company",
    "name": "QuickNode DevRel Onboarding",
    "sponsor": "QuickNode",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000002",
    "target_roles": ["developer", "engineer", "builder"],
    "target_tools": ["quicknode", "alchemy", "the-graph", "infura"],
    "required_task": "Generate API key and run SDK or CLI quickstart sample",
    "subsidy_per_call_cents": 35,
    "budget_cents": 220000
  },
  {
    "archetype": "hiring_platform",
    "name": "BuilderJobs Talent Funnel",
    "sponsor": "BuilderJobs",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000003",
    "target_roles": ["builder", "hacker", "developer"],
    "target_tools": ["claude-code", "nansen", "coinmarketcap"],
    "required_task": "Create hiring profile for agent builders",
    "subsidy_per_call_cents": 30,
    "budget_cents": 180000
  },
  {
    "archetype": "wallet_card_provider",
    "name": "Rain Card Activation Growth",
    "sponsor": "Rain Cards",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000004",
    "target_roles": ["creator", "founder", "operator"],
    "target_tools": ["x-api", "render", "neon"],
    "required_task": "Create and activate crypto card or wallet account",
    "subsidy_per_call_cents": 28,
    "budget_cents": 160000
  },
  {
    "archetype": "centralized_exchange",
    "name": "Kraken New User Conversion",
    "sponsor": "Kraken",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000005",
    "target_roles": ["trader", "analyst", "builder"],
    "target_tools": ["coingecko", "coinmarketcap", "nansen"],
    "required_task": "Create exchange account and complete onboarding",
    "subsidy_per_call_cents": 32,
    "budget_cents": 175000
  },
  {
    "archetype": "individual_creator",
    "name": "Creator Video Amplifier",
    "sponsor": "Creator Alpha",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000006",
    "target_roles": ["creator", "marketer", "community-manager"],
    "target_tools": ["x-api", "canva", "figma"],
    "required_task": "Post tweet or SNS share for sponsor video with required hashtag",
    "subsidy_per_call_cents": 24,
    "budget_cents": 140000
  },
  {
    "archetype": "research_company",
    "name": "AI Market Survey Program",
    "sponsor": "AI Market Lab",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000007",
    "target_roles": ["researcher", "operator", "founder"],
    "target_tools": ["firecrawl", "browserbase", "hugging-face"],
    "required_task": "Answer AI usage and market research survey",
    "subsidy_per_call_cents": 26,
    "budget_cents": 145000
  },
  {
    "archetype": "ai_data_provider",
    "name": "DataLabel Human Annotation Drive",
    "sponsor": "DataLabel Network",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000008",
    "target_roles": ["annotator", "builder", "researcher"],
    "target_tools": ["hugging-face", "moralis", "browserbase"],
    "required_task": "Complete human annotation or data labeling task",
    "subsidy_per_call_cents": 29,
    "budget_cents": 155000
  },
  {
    "archetype": "marketing_agency",
    "name": "UGC Growth Sprint",
    "sponsor": "UGC Growth Studio",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000009",
    "target_roles": ["marketer", "creator", "growth"],
    "target_tools": ["canva", "figma", "x-api", "midjourney"],
    "required_task": "Create SNS account and publish UGC post",
    "subsidy_per_call_cents": 27,
    "budget_cents": 150000
  },
  {
    "archetype": "infrastructure_company",
    "name": "Infra Field Ops Program",
    "sponsor": "GeoInfra Labs",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000010",
    "target_roles": ["operator", "researcher", "founder"],
    "target_tools": ["render", "railway", "neon", "supabase"],
    "required_task": "Conduct local research and upload on-site photos",
    "subsidy_per_call_cents": 31,
    "budget_cents": 165000
  },
  {
    "archetype": "blockchain_chain_company",
    "name": "Base Experience Review Rewards",
    "sponsor": "Base Ecosystem",
    "sponsor_wallet_address": "0x1000000000000000000000000000000000000011",
    "target_roles": ["consumer", "creator", "builder"],
    "target_tools": ["thirdweb", "pinata", "moralis", "alchemy", "neynar"],
    "required_task": "Post review about dining or customer service experience",
    "subsidy_per_call_cents": 33,
    "budget_cents": 170000
  }
]'

echo "== create campaigns =="
CREATED_ROWS="[]"

while IFS= read -r campaign_row; do
  payload="$(echo "$campaign_row" | jq \
    --arg discovery "${BASE_URL}/campaigns/discovery" \
    --arg tag "$RUN_TAG" \
    '{
      name: (.name + " [" + $tag + "]"),
      sponsor: .sponsor,
      sponsor_wallet_address: .sponsor_wallet_address,
      target_roles: .target_roles,
      target_tools: .target_tools,
      required_task: .required_task,
      subsidy_per_call_cents: .subsidy_per_call_cents,
      budget_cents: .budget_cents,
      query_urls: [$discovery]
    }'
  )"

  response="$(json_request POST /campaigns "$payload")"
  campaign_id="$(echo "$response" | jq -r '.campaign.id')"
  campaign_name="$(echo "$response" | jq -r '.campaign.name')"
  sponsor="$(echo "$response" | jq -r '.campaign.sponsor')"
  archetype="$(echo "$campaign_row" | jq -r '.archetype')"
  target_tools="$(echo "$response" | jq -r '.campaign.target_tools | join(",")')"

  echo "Created: ${campaign_name} | sponsor=${sponsor} | archetype=${archetype}"

  CREATED_ROWS="$(echo "$CREATED_ROWS" | jq \
    --arg id "$campaign_id" \
    --arg name "$campaign_name" \
    --arg sponsor "$sponsor" \
    --arg archetype "$archetype" \
    --arg target_tools "$target_tools" \
    '. + [{
      campaign_id: $id,
      campaign_name: $name,
      sponsor: $sponsor,
      sponsor_archetype: $archetype,
      target_tools: $target_tools
    }]'
  )"
done < <(echo "$CAMPAIGNS_JSON" | jq -c '.[]')

echo
echo "== created campaigns summary =="
echo "$CREATED_ROWS" | jq .

echo
echo "== verification: discovery snapshot =="
json_request GET "/agent/discovery/services?limit=100" | jq '{
  schema_version,
  total_count,
  services_sample: (.services[0:5] // []),
  service_catalog,
  sponsor_catalog
}'

echo
cat <<NEXT
Demo queries:
1) curl -sS "${BASE_URL}/agent/discovery/services?limit=50" -H "${AUTH_HEADER}" | jq '.service_catalog, .sponsor_catalog'
2) curl -sS "${BASE_URL}/gpt/services?q=quicknode" -H "${AUTH_HEADER}" | jq '.services, .service_catalog, .sponsor_catalog'
3) curl -sS "${BASE_URL}/gpt/services?q=figma" -H "${AUTH_HEADER}" | jq '.services, .service_catalog, .sponsor_catalog'
NEXT

echo "${RUN_TAG}" > "${SCRIPT_DIR}/.last_demo_run_tag"
echo
echo "Saved run tag to: ${SCRIPT_DIR}/.last_demo_run_tag"
