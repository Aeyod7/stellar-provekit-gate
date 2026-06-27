# Dorahacks paste pack

Copy sections into the submission form. **Repository URL** is public: push `main` before submit if you have unpushed commits.

---

## Project title

**ProveKit Gate — RISC Zero Groth16 verification on Stellar (Soroban)**

---

## One-liner (elevator)

We prove **private policy inputs** off-chain with **RISC Zero**, verify **Groth16 on Soroban**, and settle via a **gate contract** (policy commitment + proof-bound nullifier). **Live testnet txs:** first gate spend returns **true**, replay of the same proof returns **false**.

---

## Short description (form field)

**Problem:** Public chains need policy enforcement on **sensitive data** (scores, eligibility, compliance) without publishing secrets or trusting a central server.

**Solution:** Path B pipeline — RISC Zero guest → ProveKit Groth16 re-encode → Soroban `risc0-verifier` → `gate.verify_and_spend_risc0`. The guest commits a **policy hash** and **threshold_met** in its journal; the gate checks **policy version**, delegates crypto to the verifier, and blocks **double spend** via a nullifier derived from the proof.

**Evidence:** Locked artifacts in-repo, 8 gate tests (cross-contract E2E), CI, `./scripts/smoke-judge.sh`, and Stellar testnet explorer transactions (spend + replay).

**Honesty:** Groth16 proves RISC Zero execution; gate policy commitment is a separate layer — see `docs/DEMO_POLICY.md`.

---

## Longer description (if the form allows)

ProveKit Gate is infrastructure for **real-world ZK on Stellar**, not a consumer web app. Judges verify by cloning the repo and running one smoke script, then opening on-chain transactions.

**Architecture**

1. **Guest** (`provekit/methods/guest`) — private `(score, threshold)`; public journal fields include `threshold_met` and SHA-256 `policy_commitment`.
2. **Host** — `provekit-groth16-reencode` produces `artifacts/soroban_groth16_invoke.json` and `artifacts/policy_commitment.hex`.
3. **Verifier contract** — BN254 pairing verify, 5 Fr public inputs (RISC Zero wrapper VK).
4. **Gate contract** — admin initializes expected policy commitment; `verify_and_spend_risc0` cross-invokes verifier, enforces commitment match and proof-bound nullifier.

**Demo constants:** score 42, threshold 100, `threshold_met = 0` (intentional — proves execution while failing threshold).

**On-chain (testnet)**

| Step | Link |
|------|------|
| First spend → `true` | https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8 |
| Replay → `false` | https://stellar.expert/explorer/testnet/tx/4248be2854e11870dfae394852f95b9664779b8db229ebfb6ec9b517a7ad09f8 |

Gate: `CBVJSXUQEWFGDTBPKVXANKSR2P6HZ7KZDIUNPXOD6ARMARNHEHPHRNMM`  
Verifier: `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3K2H2CT3IQ3BY5RAG7D6L6KD`

**Verify locally (5 min)**

```bash
chmod +x scripts/smoke-judge.sh && ./scripts/smoke-judge.sh
```

Docs: `README.md`, `docs/SUBMISSION.md`, `docs/PATH_B.md`, `docs/SECURITY.md`.

**Video demo:** Not required — reproducible CLI + explorer evidence.

---

## Repository URL

https://github.com/Aeyod7/stellar-provekit-gate

---

## How to verify (judges)

```bash
git clone https://github.com/Aeyod7/stellar-provekit-gate.git
cd stellar-provekit-gate
./scripts/smoke-judge.sh
```

Open spend and replay txs above on Stellar Expert.

---

## Tech stack

Rust, Soroban SDK 25, RISC Zero / ProveKit, Stellar testnet, GitHub Actions.

---

## Team

Ayomide Apeh (Aeyo) — geospatial systems & analysis; ZK + Stellar integration for Real-World ZK hackathon.

---

## Optional tags / keywords

`zero-knowledge`, `RISC Zero`, `Groth16`, `Soroban`, `Stellar`, `smart contracts`, `policy`, `nullifier`, `real-world-zk`