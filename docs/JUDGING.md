# Judging map — ProveKit Gate

Hackathon: [Stellar Hacks: Real-World ZK](https://dorahacks.io/hackathon/stellar-hacks-zk/detail). This doc maps repo features to typical criteria.

## Innovation

- **RISC Zero STARK receipt → Stellar-native Groth16 verify** on Soroban BN254 pairing (not an external oracle).
- **Gate contract** composes verifier + one-shot spend semantics for a real-world policy check.

## Technical depth

- Fixed RISC Zero BN254 wrapper VK with **5 public inputs** (`control_root`, `claim_digest`, `bn254_control_id`).
- Cross-contract `verify_and_spend_risc0` delegates to `contracts/risc0-verifier`.
- **Proof-bound nullifier**: SHA-256 over proof limbs + public inputs (no replay with arbitrary nullifier).
- **Policy commitment** on gate ties on-chain spend to guest journal binding (SHA-256 over score/threshold/met).

Evidence: `cargo test` in `contracts/risc0-verifier` + `contracts/gate` (**8 tests**, cross-contract E2E on locked artifacts); `./scripts/smoke-judge.sh`; `provekit-groth16-reencode` artifacts.

## Real-world ZK

- Private **score**; public **threshold_met** and **commitment** in guest journal.
- On-chain verifies Groth16 wrapper; gate enforces expected policy commitment before spend.
- Honest layering: see **`docs/DEMO_POLICY.md`** (execution proof vs gate policy version vs nullifier).

## UX / demo

| Action | Command |
|--------|---------|
| **Judge smoke (recommended)** | `./scripts/smoke-judge.sh` |
| Reencode artifacts | `provekit/target/release/provekit-groth16-reencode` |
| Local verify | `cargo test -p risc0-verifier verify_artifacts_groth16_invoke_json` |
| Sim gate spend | `STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py` |
| Dorahacks paste | `docs/DORAHACKS_PASTE.md` |

Explorer txs: `docs/SUBMISSION.md`.

## Completeness

- Deploy scripts, locked `artifacts/`, Python invoke helpers, CI (`.github/workflows/ci.yml`).
- Docs: `PATH_B.md`, `SECURITY.md`, `SUBMISSION.md`, `DEMO_POLICY.md`, `DORAHACKS_PASTE.md`, MIT `LICENSE`.

## Demo script (3 min, no video in repo)

1. Run `./scripts/smoke-judge.sh` → green.
2. Show `artifacts/soroban_groth16_invoke.json` + `docs/DEMO_POLICY.md`.
3. Open **on-chain** gate spend + replay txs in `docs/SUBMISSION.md`.

## Self-score (2026-06-26 refresh)

| Category | 1–5 | Notes |
|----------|-----|-------|
| Innovation | 4.5 | RISC0→Soroban Groth16 + composable gate |
| Technical | 4.5 | Proof-bound nullifier, 8 gate tests, WASM E2E |
| Real-world ZK | 4.5 | Guest journal + documented gate layering |
| Completeness | 5 | Smoke script, CI, LICENSE, submission pack |
| UX / clarity | 4.5 | README Path B, judge 5-min path |
| **Overall** | **~4.6** | Target **>4.5** met |

## Hackathon readiness

| Lens | Score |
|------|-------|
| **Submission readiness** | **9.5 / 10** |
| Blockers removed | LICENSE, `.gitignore`, `smoke-judge.sh`, E2E tests, paste pack |
| Remaining for 10/10 | Public GitHub push confirmed + optional pitch video |