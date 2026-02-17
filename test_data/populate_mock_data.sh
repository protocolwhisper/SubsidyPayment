#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Populate SubsidyPayment backend with realistic mock data.

Usage:
  bash test_data/populate_mock_data.sh --api-key <BEARER_KEY> [--base-url <URL>]

Or with env vars:
  API_KEY=<BEARER_KEY> BASE_URL=<URL> bash test_data/populate_mock_data.sh

Defaults:
  BASE_URL=https://subsidypayment-1k0h.onrender.com
USAGE
}

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required (brew install jq)." >&2
  exit 1
fi

BASE_URL="${BASE_URL:-https://subsidypayment-1k0h.onrender.com}"
API_KEY="${API_KEY:-}"

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
TS="$(date +%s)"

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

echo "== create users =="
U1="$(json_request POST /register "$(cat <<JSON
{
  \"email\":\"alex.growth+${TS}@acme.io\",
  \"region\":\"US\",
  \"roles\":[\"growth\",\"marketer\"],
  \"tools_used\":[\"uniswap\",\"dune\",\"notion\"],
  \"attributes\":{\"company\":\"Acme\",\"segment\":\"defi\"}
}
JSON
)" )"

echo "$U1" | jq .
U1_ID="$(echo "$U1" | jq -r '.id')"

U2="$(json_request POST /register "$(cat <<JSON
{
  \"email\":\"maria.ops+${TS}@finflow.ai\",
  \"region\":\"EU\",
  \"roles\":[\"founder\",\"operator\"],
  \"tools_used\":[\"quicknode\",\"alchemy\",\"supabase\"],
  \"attributes\":{\"company\":\"FinFlow\",\"segment\":\"infrastructure\"}
}
JSON
)" )"

echo "$U2" | jq .
U2_ID="$(echo "$U2" | jq -r '.id')"

echo "== create campaigns (free creation, no x402 payment) =="
C1="$(json_request POST /campaigns "$(cat <<JSON
{
  \"name\":\"Uniswap Power Users Q1\",
  \"sponsor\":\"Uniswap Labs\",
  \"sponsor_wallet_address\":\"0x1111111111111111111111111111111111111111\",
  \"target_roles\":[\"growth\",\"marketer\",\"trader\"],
  \"target_tools\":[\"uniswap\",\"defi\",\"web3\"],
  \"required_task\":\"enter email\",
  \"subsidy_per_call_cents\":25,
  \"budget_cents\":50000,
  \"query_urls\":[\"${BASE_URL}/campaigns/discovery\"]
}
JSON
)" )"

echo "$C1" | jq .
C1_ID="$(echo "$C1" | jq -r '.campaign.id')"

C2="$(json_request POST /campaigns "$(cat <<JSON
{
  \"name\":\"QuickNode Builder Boost\",
  \"sponsor\":\"QuickNode\",
  \"sponsor_wallet_address\":\"0x2222222222222222222222222222222222222222\",
  \"target_roles\":[\"founder\",\"operator\",\"developer\"],
  \"target_tools\":[\"quicknode\",\"rpc\",\"infra\"],
  \"required_task\":\"answer survey\",
  \"subsidy_per_call_cents\":35,
  \"budget_cents\":75000,
  \"query_urls\":[\"${BASE_URL}/campaigns/discovery\"]
}
JSON
)" )"

echo "$C2" | jq .
C2_ID="$(echo "$C2" | jq -r '.campaign.id')"

echo "== complete tasks (FREE path readiness) =="
json_request POST /tasks/complete "$(cat <<JSON
{
  \"campaign_id\":\"${C1_ID}\",
  \"user_id\":\"${U1_ID}\",
  \"task_name\":\"enter email\",
  \"details\":\"alex.growth+${TS}@acme.io\"
}
JSON
)" | jq .

json_request POST /tasks/complete "$(cat <<JSON
{
  \"campaign_id\":\"${C2_ID}\",
  \"user_id\":\"${U2_ID}\",
  \"task_name\":\"answer survey\",
  \"details\":\"q1=yes,q2=builder\"
}
JSON
)" | jq .

echo "== verify campaign discovery =="
json_request GET /campaigns/discovery | jq .

echo "== gpt auth for user1 =="
AUTH1="$(json_request POST /gpt/auth "$(cat <<JSON
{
  \"email\":\"alex.growth+${TS}@acme.io\",
  \"region\":\"US\",
  \"roles\":[\"growth\",\"marketer\"],
  \"tools_used\":[\"uniswap\",\"dune\",\"notion\"]
}
JSON
)" )"

echo "$AUTH1" | jq .
SESSION1="$(echo "$AUTH1" | jq -r '.session_token')"

echo "== run sponsored service (uniswap) =="
json_request POST /gpt/services/uniswap/run "$(cat <<JSON
{
  \"session_token\":\"${SESSION1}\",
  \"input\":\"{\\\"action\\\":\\\"quote\\\",\\\"pair\\\":\\\"ETH/USDC\\\"}\"
}
JSON
)" | jq .

echo "== non-sponsored check (figma should return x402 payment_required via /proxy) =="
X402_BODY_FILE="$(mktemp)"
X402_STATUS="$(curl -sS -o "$X402_BODY_FILE" -w "%{http_code}" -X POST "${BASE_URL}/proxy/figma/run" \
  -H "$AUTH_HEADER" \
  -H "Content-Type: application/json" \
  -d "$(cat <<JSON
{
  \"user_id\":\"${U1_ID}\",
  \"input\":\"create hero image concept\"
}
JSON
)")"

echo "HTTP status: ${X402_STATUS}"
cat "$X402_BODY_FILE" | jq .
rm -f "$X402_BODY_FILE"

echo
if [[ "$X402_STATUS" == "402" ]]; then
  echo "PASS: non-sponsored service returned x402 payment_required as expected."
else
  echo "WARN: expected 402 for non-sponsored service, got ${X402_STATUS}."
fi

echo
echo "== done =="
echo "User 1: ${U1_ID}"
echo "User 2: ${U2_ID}"
echo "Campaign 1 (uniswap): ${C1_ID}"
echo "Campaign 2 (quicknode): ${C2_ID}"
echo
cat <<NEXT
Next test prompts in ChatGPT:
1) Find sponsored services for uniswap.
2) Run service uniswap with input {\"action\":\"quote\",\"pair\":\"ETH/USDC\"}.
3) Run service figma with input \"create hero image concept\" (should route to PAY/x402 flow).
NEXT
