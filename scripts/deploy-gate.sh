#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"

if [[ -f "$ROOT_DIR/scripts/env.local" ]]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/scripts/env.local"
fi

NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-provekit-deployer}"

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

stellar contract alias show provekit-gate --network "$NETWORK"

echo "==> Next: ./scripts/initialize-gate-admin.sh (admin + policy commitment)"
echo "==> Then: ./scripts/init-gate-risc0-verifier.sh (admin auth required)"