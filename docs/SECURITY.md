# Security notes — ProveKit Gate (testnet)

This project is a **hackathon testnet demo**, not production-ready. Documented limits and fixes below.

## Testnet limits

- **VK + expected `claim_digest` are baked into the contracts** — changing the guest circuit requires regen and redeploy.
- **No formal audit** — pairing crypto relies on Soroban host + RISC Zero reference tooling.
- **Secrets in `scripts/env.local`** — never commit; use `scripts/env.example` only.
- **Nullifier set is a single `Map` entry** — fine for the demo; production should key each nullifier under its own persistent entry (avoids unbounded re-serialization) and bump TTL.
- **Live deployment vs repo** — the linked testnet txs were produced by the prior gate build; repo `main` adds the guest-claim binding and removes the legacy path. Redeploy with `scripts/deploy-gate.sh`; demo behavior (`spend → true`, `replay → false`) is unchanged.

## Security properties enforced in this repo

### Guest-claim binding (proof ↔ program)

- `verify_and_spend_risc0` requires the proof's `claim_digest` limbs (public inputs 2,3) to equal a value baked into the gate at build time.
- `claim_digest = SHA-256(image_id, journal_digest, …)`, so an arbitrary valid RISC Zero proof of a *different* program/output is rejected — closing the "any valid proof passes" gap.

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

Gate tests (`contracts/gate/src/test.rs`) assert `false` for each failure mode:

- wrong `policy_commitment`
- wrong `claim_digest` (guest-claim binding)
- wrong `nullifier`
- replay of an already-spent proof

On testnet, set `GATE_NULLIFIER_HEX` to a wrong 32-byte hex to confirm a spend returns `false`.