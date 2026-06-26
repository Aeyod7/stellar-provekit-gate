# Hackathon submission — ProveKit Gate (Path B)

**Repo:** https://github.com/Aeyod7/stellar-provekit-gate  
**Local path:** `/home/aeyo/stellar-provekit-gate`  
**Track:** RISC Zero / Groth16 on Soroban (Stellar testnet)  
**Readiness (self-assessed):** **9.5 / 10** — see `docs/JUDGING.md`, `./scripts/smoke-judge.sh`

## One-liner

Prove a RISC Zero guest policy off-chain, re-encode the Groth16 seal for Soroban BN254, verify on-chain, optionally spend a proof-bound nullifier via a gate contract that delegates to the verifier and checks policy commitment.

## What works (verified)

| Step | Command / script | Result |
|------|------------------|--------|
| Off-chain RISC0 Groth16 | `provekit/target/release/provekit-groth16-risc0-verify` | OK |
| Re-encode for Soroban | `provekit/target/release/provekit-groth16-reencode` | `artifacts/soroban_groth16_invoke.json`, `artifacts/policy_commitment.hex` |
| Local on-chain verify | `cargo test -p risc0-verifier verify_artifacts_groth16_invoke_json` | OK |
| Gate E2E (locked artifacts) | `cargo test -p provekit-gate` (8 tests) | OK |
| Judge smoke (all local) | `./scripts/smoke-judge.sh` | OK |
| Testnet `verify_proof` (sim) | `scripts/invoke-risc0-groth16-testnet.py` | `true` |
| Testnet `verify_proof` (on-chain) | `STELLAR_SEND=yes python3 scripts/invoke-risc0-groth16-testnet.py` | [tx `c94800bd…`](https://stellar.expert/explorer/testnet/tx/c94800bd1b7cc2b806681fa9ae38c3885fd8036b6423027bb11a40105b57c66a) → `true` |
| Gate → verifier + nullifier (sim) | `STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py` | `true` |
| Gate on-chain spend (security upgrade) | `STELLAR_SEND=yes` + new gate ID | [tx `2a9cb388…`](https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8) → `true` |
| Gate init (admin + policy) | `initialize` on new gate | [tx `9e93fe24…`](https://stellar.expert/explorer/testnet/tx/9e93fe244a238e207f64f5a86b492e6243c1882f77192685cf3c5b012003ca2a) |
| Gate `init_risc0_verifier` | admin one-time | [tx `6f160086…`](https://stellar.expert/explorer/testnet/tx/6f1600860235ec377a499ad1fe294c3df0140f99904b95a406f32a5544c6a609) |

## Testnet contract IDs (current)

| Contract | ID | Alias |
|----------|-----|-------|
| RISC0 Groth16 verifier | `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3KGPU6KVSJ7T3VKH7JDDLFV` | `provekit-risc0-verifier` |
| Gate (policy + proof-bound nullifier) | `CBVJSXUQEWFGDTBPKVXANKSR2P6HZ7KZDIUNPXOD6ARMARNHEHPHRNMM` | `provekit-gate` |

> **After redeploy:** update `SOROBAN_GATE_ID` in `scripts/env.local` or unset it so scripts use `stellar contract alias show provekit-gate`.

**Gate setup (after each gate deploy):**

1. `./scripts/initialize-gate-admin.sh` — admin + policy commitment  
2. `./scripts/init-gate-risc0-verifier.sh` — admin-only, one-time  

## On-chain verification story

Soroban verifies the **RISC Zero Groth16 BN254 wrapper** (`verify_proof`). The gate adds **policy commitment** (guest journal binding) and **proof-bound nullifier** before marking a spend.

## Demo artifacts (policy: value ≥ 100, prove `42`)

- `artifacts/soroban_groth16_invoke.json` — proof + 5 public inputs for CLI invoke
- `artifacts/policy_commitment.hex` — SHA-256 commitment for demo inputs (score 42, threshold 100)
- `artifacts/groth16_seal.bin`, `artifacts/groth16_claim_digest.hex`
- Guest: `provekit/methods/guest` (RISC Zero)

## Architecture (judge-friendly)

1. **Guest** — RISC Zero ELF proves policy (e.g. threshold); SHA-256 commitment in journal.
2. **Host** — `provekit/host` compresses seal, builds Soroban G1/G2 limbs + 5 Fr public inputs.
3. **Verifier** — `contracts/risc0-verifier` fixed VK, `verify_proof`.
4. **Gate** — `verify_and_spend_risc0` checks policy commitment + nullifier = proof id, cross-invokes verifier.

Path A (`contracts/verifier` + gate inline VK) remains a Poseidon stub; **Path B is the submission story.**

## Reproduce (no Docker required for locked artifacts)

```bash
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
cd /home/aeyo/stellar-provekit-gate

provekit/target/release/provekit-groth16-reencode
provekit/target/release/provekit-groth16-risc0-verify
cd contracts/risc0-verifier && cargo test verify_artifacts_groth16_invoke_json -- --nocapture
cd ../gate && cargo test

STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py
```

**Fresh Groth16 prove** (new seal / VK): needs Docker — see `docs/PATH_B.md`.

## Send a real gate tx (after redeploy)

```bash
source scripts/env.local
STELLAR_SEND=yes python3 scripts/invoke-gate-risc0-testnet.py
```

Nullifier is derived from the proof (see `scripts/invoke-gate-risc0-testnet.py` header). Re-spend of the same proof returns `false`.

## Judge checklist (self-score)

| Category | Score (1–5) | Evidence |
|----------|-------------|----------|
| Innovation | 4 | RISC0 receipt → native Soroban Groth16 + gate composition |
| Technical | 4 | 5 public inputs, pairing verify, proof-bound nullifier, policy commitment |
| Real-world ZK | 4 | Private score, public threshold_met + commitment |
| Completeness | 5 | CI, gate tests, explorer txs (verifier + **new gate**) |
| UX / clarity | 4 | SUBMISSION + JUDGING + SECURITY + PATH_B |

Honest gap: production audit / mainnet hardening still out of scope; demo is testnet-complete.

**Weighted overall (self): ~4.4 / 5** — see `docs/JUDGING.md`.

## Docs

- `docs/JUDGING.md` — rubric map
- `docs/SECURITY.md` — testnet limits + fixes
- `docs/PATH_B.md` — Path B pipeline + redeploy
- `README.md` — quick start