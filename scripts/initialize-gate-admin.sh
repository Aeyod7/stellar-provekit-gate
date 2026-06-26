#!/usr/bin/env bash
# One-time after gate deploy: set admin + policy commitment from artifacts.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"

if [[ -f "$ROOT_DIR/scripts/env.local" ]]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/scripts/env.local"
fi

NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-provekit-deployer}"
POLICY_FILE="${POLICY_FILE:-$ROOT_DIR/artifacts/policy_commitment.hex}"
GATE_ID="${SOROBAN_GATE_ID:-}"
if [[ -z "$GATE_ID" ]]; then
  GATE_ID="$(stellar contract alias show provekit-gate --network "$NETWORK" 2>/dev/null || true)"
fi
# Prefer stellar alias when env.local still pins an old contract ID after redeploy.
ALIAS_ID="$(stellar contract alias show provekit-gate --network "$NETWORK" 2>/dev/null || true)"
if [[ -n "$ALIAS_ID" && -n "$GATE_ID" && "$GATE_ID" != "$ALIAS_ID" ]]; then
  echo "Note: SOROBAN_GATE_ID ($GATE_ID) != alias provekit-gate ($ALIAS_ID); using alias" >&2
  GATE_ID="$ALIAS_ID"
fi

if [[ -z "$GATE_ID" ]]; then
  echo "Need SOROBAN_GATE_ID or provekit-gate alias" >&2
  exit 1
fi

if [[ ! -f "$POLICY_FILE" ]]; then
  echo "Missing $POLICY_FILE — run provekit-groth16-reencode first" >&2
  exit 1
fi

POLICY_HEX="$(tr -d '[:space:]' < "$POLICY_FILE")"

echo "==> gate $GATE_ID initialize(admin=$SOURCE_ACCOUNT, policy_commitment)"
stellar contract invoke \
  --id "$GATE_ID" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK" \
  -- \
  initialize \
  --admin "$SOURCE_ACCOUNT" \
  --policy_commitment "$POLICY_HEX"