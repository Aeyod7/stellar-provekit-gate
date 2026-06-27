# ProveKit Gate

**RISC Zero Groth16 verification on Stellar (Soroban)** — prove a private policy off-chain, verify natively on-chain, and settle through a gate contract with policy binding and proof-bound nullifiers.

**Repository:** https://github.com/Aeyod7/stellar-provekit-gate  
**Track:** Real-World ZK / Stellar (Dorahacks) — **Path B** only  
**Status:** Testnet-complete with locked artifacts, CI, and public explorer transactions.

---

## The problem

Public blockchains are good at **transparent settlement**, but many real policies depend on **data you cannot publish**:

- Credit or risk scores  
- Eligibility thresholds  
- Compliance checks on private inputs  

Teams are stuck between bad options:

| Approach | Failure mode |
|----------|----------------|
| Put raw data on-chain | No privacy |
| “Trust our API” | Not verifiable on the ledger |
| ZK only off-chain | No Stellar-native enforcement |

**What builders need:** run a defined program on private inputs, produce a cryptographic attestation, and let **anyone** verify on Stellar that (1) the proof is valid, (2) the deployment accepts this **policy version**, and (3) the same proof cannot **spend twice**.

---

## Why this project exists

Stellar’s **Soroban** host exposes **BN254 pairing** precompiles suitable for **Groth16** verification. **RISC Zero** provides a practical path from arbitrary Rust guest logic to Groth16 receipts. **ProveKit** (this repo’s host tooling) bridges RISC Zero seals into the exact proof layout Soroban expects.

**ProveKit Gate** demonstrates the full stack for a concrete “real-world ZK” story:

- **Private:** numeric score and threshold (demo inputs).  
- **Public in the proof journal:** whether the threshold was met, plus a **policy commitment** (hash binding score, threshold, and outcome).  
- **On-chain:** verify Groth16, match policy commitment, consume a **nullifier** derived from the proof itself.

The same pattern applies to access control, attestations, and settlement hooks — not only a toy `if score > threshold`.

---

## Our solution 

End-to-end pipeline:

```
RISC Zero guest (private score / threshold → journal + commitment)
        ↓
ProveKit host (Groth16 re-encode for Soroban BN254)
        ↓
artifacts/soroban_groth16_invoke.json
        ↓
Soroban risc0-verifier.verify_proof (5 public inputs, fixed VK)
        ↓
gate.verify_and_spend_risc0 (policy + proof-bound nullifier + cross-invoke)
```

### Components

| Layer | Location | Role |
|-------|----------|------|
| **Guest** | `provekit/methods/guest` | RISC Zero program; SHA-256 policy commitment in journal |
| **Host** | `provekit/host` | STARK→SNARK seal, Soroban invoke JSON, `policy_commitment.hex` |
| **Verifier** | `contracts/risc0-verifier` | On-chain Groth16 verify (RISC Zero BN254 wrapper) |
| **Gate** | `contracts/gate` | `verify_and_spend_risc0`: verifier + policy match + one-shot spend |

**Path A** (`contracts/verifier` + legacy gate path) is a Poseidon stub for interim experiments — **judges should evaluate Path B only.** See [`docs/PATH_B.md`](docs/PATH_B.md).

### What the gate enforces (three layers)

Documented honestly in [`docs/DEMO_POLICY.md`](docs/DEMO_POLICY.md):

1. **Cryptography** — Groth16 proves a valid RISC Zero execution (claim digest, control root, etc. in public inputs).  
2. **Policy version** — caller’s `policy_commitment` must match the value set at gate `initialize`.  
3. **Anti-replay** — `nullifier` = `SHA-256(proof_a ‖ proof_b ‖ proof_c ‖ public_inputs)`; same proof cannot spend twice.

These are **separate concerns** by design so judges are not misled into thinking the verifier alone encodes business policy.

### Demo policy (locked in repo)

| Field | Demo value |
|-------|------------|
| Private score | `42` |
| Threshold | `100` |
| `threshold_met` | `0` (fails threshold — still proves honest execution) |
| Policy commitment | `artifacts/policy_commitment.hex` |

---

## On-chain evidence (Stellar testnet)

| Event | Explorer |
|-------|----------|
| Gate spend (first use → **true**) | [2a9cb388…](https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8) |
| Replay (same proof → **false**) | [4248be28…](https://stellar.expert/explorer/testnet/tx/4248be2854e11870dfae394852f95b9664779b8db229ebfb6ec9b517a7ad09f8) |

**Contract IDs (current):**

| Contract | ID |
|----------|-----|
| Gate | `CBVJSXUQEWFGDTBPKVXANKSR2P6HZ7KZDIUNPXOD6ARMARNHEHPHRNMM` |
| RISC0 verifier | `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3K2H2CT3IQ3BY5RAG7D6L6KD` |

Full deploy/init/spend history: [`docs/SUBMISSION.md`](docs/SUBMISSION.md).

---

## Judges: verify in ~5 minutes
```bash
git clone https://github.com/Aeyod7/stellar-provekit-gate.git
cd stellar-provekit-gate
chmod +x scripts/smoke-judge.sh
./scripts/smoke-judge.sh
```

Expected final line: **`OK — local judge smoke passed.`**

Then read:

| Doc | Purpose |
|-----|---------|
| [`docs/SUBMISSION.md`](docs/SUBMISSION.md) | Commands, txs, self-score |
| [`docs/DEMO_POLICY.md`](docs/DEMO_POLICY.md) | Guest vs gate semantics |
| [`docs/PATH_B.md`](docs/PATH_B.md) | Pipeline and redeploy |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Testnet limits and fixes |
| [`docs/DORAHACKS_PASTE.md`](docs/DORAHACKS_PASTE.md) | Form copy-paste pack |

**CI:** GitHub Actions runs verifier tests, builds verifier WASM, runs gate tests (including cross-contract E2E on locked `artifacts/soroban_groth16_invoke.json`).

---

## Manual reproduction (without smoke script)

```bash
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
cd stellar-provekit-gate

# Host + locked artifacts
cargo build --release --manifest-path provekit/host/Cargo.toml
provekit/target/release/provekit-groth16-reencode
provekit/target/release/provekit-groth16-risc0-verify

# Contracts
cd contracts/risc0-verifier && stellar contract build && cargo test
cd ../gate && cargo test
```

Fresh Groth16 prove (new seal / VK) requires Docker — see [`docs/PATH_B.md`](docs/PATH_B.md).

---

## Repository layout

```
provekit/
  methods/guest/     # RISC Zero guest (policy program)
  host/              # Prove + Groth16 reencode binaries
contracts/
  risc0-verifier/    # Soroban Groth16 verifier (Path B)
  gate/              # Policy + nullifier gate
artifacts/           # Locked invoke JSON, policy hex, seals (committed)
docs/                # Submission, judging, security, Path B
scripts/
  smoke-judge.sh     # One-command local judge path
  env.example        # Template only — never commit secrets
```

---

## Tech stack

- **Rust**, Soroban SDK 25, Stellar testnet  
- **RISC Zero** / ProveKit host tooling  
- **BN254** Groth16 on Soroban (RISC Zero wrapper circuit, 5 public inputs)  
- **GitHub Actions** CI  

---

## Security and scope

This is a **hackathon testnet demonstration**, not a mainnet security audit. VK is embedded in verifier WASM; guest circuit changes require regen and redeploy. See [`docs/SECURITY.md`](docs/SECURITY.md).

---

## License

MIT — see [`LICENSE`](LICENSE). Copyright (c) 2026 Ayomide Apeh (Aeyo).
