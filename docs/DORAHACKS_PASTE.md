# Dorahacks paste pack

Copy sections into the submission form. Update **Repository URL** after `git push`.

---

## Project title

**ProveKit Gate — RISC Zero Groth16 verification on Stellar (Soroban)**

---

## One-liner (elevator)

We verify **RISC Zero** Groth16 proofs **on-chain** on Stellar testnet: a Soroban gate checks policy commitment + proof-bound nullifier, delegates to a Groth16 verifier, and **one-shot spends** — with **live explorer txs** (first spend `true`, replay `false`).

---

## Description (short)

Path B hackathon track: **RISC Zero → ProveKit Groth16 → Soroban**.

- **Guest:** private credit-style `(score, threshold)` → public `policy_commitment` + `threshold_met` in journal (`provekit/methods/guest`).
- **Host:** reprove / reencode to `artifacts/soroban_groth16_invoke.json`.
- **Contracts:** `risc0-verifier` (BN254 pairing, 5 public inputs) + `gate` (`verify_and_spend_risc0`).
- **Tests:** 8 gate tests including cross-contract E2E on locked artifacts; CI + `scripts/smoke-judge.sh`.

**On-chain evidence (testnet):**

| Step | Explorer |
|------|----------|
| Gate deploy | [SUBMISSION.md](./SUBMISSION.md) |
| First spend → `true` | https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8 |
| Replay → `false` | https://stellar.expert/explorer/testnet/tx/4248be2854e11870dfae394852f95b9664779b8db229ebfb6ec9b517a7ad09f8 |

Gate: `CBVJSXUQEWFGDTBPKVXANKSR2P6HZ7KZDIUNPXOD6ARMARNHEHPHRNMM`  
Verifier: `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3K2H2CT3IQ3BY5RAG7D6L6KD`

---

## Repository URL

`https://github.com/Aeyod7/stellar-provekit-gate` *(push before submit)*

---

## How to verify (judges, 5 min)

```bash
chmod +x scripts/smoke-judge.sh && ./scripts/smoke-judge.sh
```

Read: `docs/SUBMISSION.md`, `docs/DEMO_POLICY.md`, `docs/PATH_B.md`.

---

## Tech stack

Rust, Soroban SDK 25, RISC Zero / ProveKit, Stellar testnet, GitHub Actions CI.

---

## Team

Ayomide Apeh (Aeyo) — geospatial / systems builder; ZK + Stellar integration for Real-World ZK hackathon.