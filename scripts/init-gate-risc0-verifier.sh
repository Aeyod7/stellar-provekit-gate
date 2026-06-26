#!/usr/bin/env bash
# Point deployed gate at deployed risc0-verifier (admin auth, one-time per gate deploy).
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"

if [[ -f "$ROOT_DIR/scripts/env.local" ]]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/scripts/env.local"
fi

NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-provekit-deployer}"
GATE_ID="${SOROBAN_GATE_ID:-}"
VERIFIER_ID="${SOROBAN_RISC0_VERIFIER_ID:-}"

if [[ -z "$GATE_ID" ]]; then
  GATE_ID="$(stellar contract alias show provekit-gate --network "$NETWORK" 2>/dev/null || true)"
fi
ALIAS_GATE="$(stellar contract alias show provekit-gate --network "$NETWORK" 2>/dev/null || true)"
if [[ -n "$ALIAS_GATE" && -n "$GATE_ID" && "$GATE_ID" != "$ALIAS_GATE" ]]; then
  echo "Note: SOROBAN_GATE_ID ($GATE_ID) != alias provekit-gate ($ALIAS_GATE); using alias" >&2
  GATE_ID="$ALIAS_GATE"
fi
if [[ -z "$VERIFIER_ID" ]]; then
  VERIFIER_ID="$(stellar contract alias show provekit-risc0-verifier --network "$NETWORK" 2>/dev/null || true)"
fi

if [[ -z "$GATE_ID" || -z "$VERIFIER_ID" ]]; then
  echo "Need SOROBAN_GATE_ID and SOROBAN_RISC0_VERIFIER_ID (or stellar aliases)" >&2
  exit 1
fi

echo "==> gate $GATE_ID init_risc0_verifier -> $VERIFIER_ID"
stellar contract invoke \
  --id "$GATE_ID" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK" \
  -- \
  init_risc0_verifier \
  --risc0_verifier "$VERIFIER_ID"