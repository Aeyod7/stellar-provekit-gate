#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.local/bin:${HOME}/.risc0/bin:${HOME}/.cargo/bin:${PATH}"

if ! command -v stellar >/dev/null 2>&1; then
  echo "stellar CLI required — install to ~/.local/bin (see README)" >&2
  exit 1
fi

NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-provekit-deployer}"
DEPLOY_GATE="${DEPLOY_GATE:-true}"

if [[ -f "$ROOT_DIR/scripts/env.local" ]]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/scripts/env.local"
fi

echo "==> Building verifier WASM"
stellar contract build \
  --manifest-path "$ROOT_DIR/contracts/verifier/Cargo.toml" \
  --optimize

VERIFIER_WASM="${VERIFIER_WASM:-$ROOT_DIR/contracts/verifier/target/wasm32v1-none/release/soroban_zk_verifier.wasm}"

echo "==> Deploying verifier to $NETWORK (source: $SOURCE_ACCOUNT)"
stellar contract deploy \
  --wasm "$VERIFIER_WASM" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK" \
  --alias provekit-verifier \
  --cost

if [[ "$DEPLOY_GATE" == "true" ]]; then
  echo "==> Building gate WASM"
  stellar contract build \
    --manifest-path "$ROOT_DIR/contracts/gate/Cargo.toml" \
    --optimize

  GATE_WASM="${GATE_WASM:-$ROOT_DIR/contracts/gate/target/wasm32v1-none/release/provekit_gate.wasm}"

  echo "==> Deploying gate to $NETWORK"
  stellar contract deploy \
    --wasm "$GATE_WASM" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    --alias provekit-gate \
    --cost
fi

echo "==> Done. Contract IDs (see docs/TESTNET.md):"
stellar contract alias show provekit-verifier --network "$NETWORK" 2>/dev/null || true
stellar contract alias show provekit-gate --network "$NETWORK" 2>/dev/null || true