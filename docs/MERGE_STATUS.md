# Merge status — win-score pass

**Date:** 2026-06-26  
**Sources merged:** GLM/subagent handoff (`DEVIN_HANDOFF_WIN_SCORE.md`) + operator redeploy + doc polish.

## Shipped in repo

| Area | Status |
|------|--------|
| Admin + one-time `init_risc0_verifier` | ✅ `contracts/gate` |
| Policy commitment + proof-bound nullifier | ✅ |
| Guest SHA-256 commitment | ✅ `provekit/methods/guest` |
| `artifacts/policy_commitment.hex` | ✅ via `provekit-groth16-reencode` |
| Gate unit tests (4) | ✅ `cargo test -p provekit-gate` |
| RISC0 artifact test | ✅ |
| CI workflow | ✅ `.github/workflows/ci.yml` |
| `JUDGING.md`, `SECURITY.md`, `SUBMISSION.md` | ✅ updated |

## Testnet (post-merge)

| Step | Tx |
|------|-----|
| Gate deploy | [fbdee8dc…](https://stellar.expert/explorer/testnet/tx/fbdee8dce0857fb8462e8385348993b1ec3c1d3c88919a4acd41c95974c628c3) |
| `initialize` | [9e93fe24…](https://stellar.expert/explorer/testnet/tx/9e93fe244a238e207f64f5a86b492e6243c1882f77192685cf3c5b012003ca2a) |
| `init_risc0_verifier` | [6f160086…](https://stellar.expert/explorer/testnet/tx/6f1600860235ec377a499ad1fe294c3df0140f99904b95a406f32a5544c6a609) |
| `verify_and_spend_risc0` (first) | [2a9cb388…](https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8) → `true` |
| Replay (same proof) | [4248be28…](https://stellar.expert/explorer/testnet/tx/4248be2854e11870dfae394852f95b9664779b8db229ebfb6ec9b517a7ad09f8) → `false` |

**Gate ID:** `CBVJSXUQEWFGDTBPKVXANKSR2P6HZ7KZDIUNPXOD6ARMARNHEHPHRNMM`

## Dorahacks paste checklist

1. One-liner from `docs/SUBMISSION.md`
2. Repo link (push to GitHub if not public yet)
3. Verifier tx + **new gate spend tx** (proof-bound nullifier)
4. Say: Path B only; Path A is stub
5. Optional: Boundless as future prover — same Soroban API

## Still optional (time permitting)

- Screen recording (you skipped video)
- Push repo + run Devin pass for typo sweep
- Second gate invoke → expect `false` (replay) for judge demo