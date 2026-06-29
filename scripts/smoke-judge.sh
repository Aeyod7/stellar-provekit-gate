#!/usr/bin/env bash
# Judge / CI smoke: local proof pipeline + contract tests (no testnet required).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="${HOME}/.cargo/bin:${HOME}/.local/bin:${PATH}"

echo "==> risc0-verifier tests"
cargo test --manifest-path "$ROOT/contracts/risc0-verifier/Cargo.toml" -q

echo "==> build risc0-verifier WASM (gate E2E)"
(cd "$ROOT/contracts/risc0-verifier" && stellar contract build -q)

echo "==> gate tests (9 tests incl. locked-artifact E2E + claim binding)"
cargo test --manifest-path "$ROOT/contracts/gate/Cargo.toml" -q

echo "==> verify locked Groth16 invoke JSON"
cargo test --manifest-path "$ROOT/contracts/risc0-verifier/Cargo.toml" verify_artifacts_groth16_invoke_json -q -- --nocapture

# Optional: reprove/reencode with the RISC Zero toolchain. Locked artifacts are
# already committed, so the core judge path works without Docker/rzup.
if command -v rzup >/dev/null 2>&1; then
    echo "==> host release + groth16 reencode (RISC Zero toolchain found)"
    cargo build --release --manifest-path "$ROOT/provekit/host/Cargo.toml" -q
    "$ROOT/provekit/target/release/provekit-groth16-reencode"
fi

echo ""
echo "OK — local judge smoke passed."
echo "On-chain demo: docs/SUBMISSION.md (testnet txs + explorer links)."