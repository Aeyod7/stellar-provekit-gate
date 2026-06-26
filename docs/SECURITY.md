# Security notes — ProveKit Gate (testnet)

This project is a **hackathon testnet demo**, not production-ready. Documented limits and fixes below.

## Testnet limits

- **Verifier VK is baked into WASM** — changing the guest circuit requires VK regen and redeploy.
- **No formal audit** — pairing crypto relies on Soroban host + RISC Zero reference tooling.
- **Secrets in `scripts/env.local`** — never commit; use `scripts/env.example` only.
- **Path A gate** (`verify_and_spend`) still uses placeholder Poseidon VK — use Path B for RISC Zero.

## Fixes in this repo (win-score handoff)

### Admin-gated verifier wiring

- `initialize(admin, policy_commitment)` runs once after deploy.
- `init_risc0_verifier` requires **admin auth** and is **one-time** (prevents arbitrary verifier swap).

### Proof-bound nullifier

- `nullifier` must equal `SHA-256(proof_a || proof_b || proof_c || public_inputs…)`.
- Prevents spending a valid proof under a fresh random nullifier (replay with new id).

### Policy commitment

- Gate stores expected guest **policy commitment** at init.
- `verify_and_spend_risc0` rejects mismatched `policy_commitment` arg.

### Guest commitment (stronger binding)

- Guest journal commitment is **SHA-256** over `(score, threshold, threshold_met)` little-endian limbs (not a toy tag).

## Operational checklist after gate changes

1. `./scripts/deploy-gate.sh`
2. `./scripts/initialize-gate-admin.sh`
3. `./scripts/init-gate-risc0-verifier.sh` (as admin)
4. `STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py`

## Negative testing

- Set `GATE_NULLIFIER_HEX` to a wrong 32-byte hex to confirm spend returns `false`.