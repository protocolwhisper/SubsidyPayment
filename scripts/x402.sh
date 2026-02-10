#!/usr/bin/env bash
set -euo pipefail

required_env=(
  "X402_PAY_TO"
  "X402_ASSET"
  "TESTNET_PAYMENT_SIGNATURE_DESIGN"
)

missing=()
for key in "${required_env[@]}"; do
  if [[ -z "${!key:-}" ]]; then
    missing+=("${key}")
  fi
done

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "Missing required env vars:"
  for key in "${missing[@]}"; do
    echo "  - ${key}"
  done
  echo
  echo "Optional env vars:"
  echo "  X402_FACILITATOR_URL (default: https://x402.org/facilitator)"
  echo "  X402_VERIFY_PATH (default: /verify)"
  echo "  X402_SETTLE_PATH (default: /settle)"
  echo "  X402_NETWORK (default: base-sepolia)"
  echo "  PUBLIC_BASE_URL (default: http://localhost:3000)"
  exit 1
fi

echo "Running live x402 testnet tests..."
cargo test testnet_payment_signature_unlocks_tool -- --nocapture
cargo test testnet_payment_signature_service_mismatch_is_rejected -- --nocapture
echo "Live x402 testnet tests finished."
