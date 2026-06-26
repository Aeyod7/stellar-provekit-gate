#!/usr/bin/env bash
# Deploy RISC Zero Groth16 Soroban verifier (5 public inputs) to testnet.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
NETWORK="${NETWORK:-testnet}"
SOURCE_ACCOUNT="${SOURCE_ACCOUNT:-provekit-deployer}"
export STELLAR_NETWORK_PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
if [[ -f "$ROOT_DIR/scripts/env.local" ]]; then
  # shellcheck disable=SC1091
  source "$ROOT_DIR/scripts/env.local"
fi
GEN_VK="$ROOT_DIR/provekit/target/release/provekit-groth16-gen-vk"
if [[ ! -x "$GEN_VK" ]]; then
  echo "==> Building provekit-groth16-gen-vk"
  cargo build --release --manifest-path "$ROOT_DIR/provekit/host/Cargo.toml" --bin provekit-groth16-gen-vk
fi
"$GEN_VK"
echo "==> Building risc0-verifier WASM"
stellar contract build \
  --manifest-path "$ROOT_DIR/contracts/risc0-verifier/Cargo.toml" \
  --optimize
WASM="$ROOT_DIR/contracts/risc0-verifier/target/wasm32v1-none/release/soroban_risc0_verifier.wasm"
echo "==> Deploying to $NETWORK"
stellar contract deploy \
  --wasm "$WASM" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK" \
  --alias provekit-risc0-verifier \
  --cost