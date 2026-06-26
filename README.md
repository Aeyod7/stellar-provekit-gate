# ProveKit Gate — RISC Zero on Stellar (Path B)

**Hackathon track:** Real-World ZK — RISC Zero Groth16 verified on **Soroban** with a one-shot **gate** (policy commitment + proof-bound nullifier).

| Judges start here | |
|-------------------|---|
| **5-min verify** | `./scripts/smoke-judge.sh` |
| **On-chain txs** | [docs/SUBMISSION.md](docs/SUBMISSION.md) |
| **Dorahacks copy-paste** | [docs/DORAHACKS_PASTE.md](docs/DORAHACKS_PASTE.md) |
| **Policy story** | [docs/DEMO_POLICY.md](docs/DEMO_POLICY.md) |

## Architecture (Path B only)

```
RISC Zero guest (score/threshold journal)
    → ProveKit host (Groth16 reencode)
    → artifacts/soroban_groth16_invoke.json
    → Soroban risc0-verifier.verify_proof
    → gate.verify_and_spend_risc0 (policy + nullifier + cross-invoke)
```

- **Guest:** `provekit/methods/guest` — SHA-256 policy commitment in journal  
- **Verifier:** `contracts/risc0-verifier` — BN254 Groth16, 5 public inputs  
- **Gate:** `contracts/gate` — admin-gated verifier wiring, replay protection  

Path A (Poseidon / native ProveKit verifier) is **out of scope** for this submission — see [docs/PATH_B.md](docs/PATH_B.md).

## Quick start (local)

```bash
chmod +x scripts/smoke-judge.sh
./scripts/smoke-judge.sh
```

Requires: Rust, Stellar CLI (`stellar contract build`). Locked artifacts are committed under `artifacts/`.

## Testnet demo (optional)

1. Copy `scripts/env.example` → `scripts/env.local` (never commit secrets).  
2. `scripts/deploy-gate.sh` → `scripts/init-gate-risc0-verifier.sh`  
3. `STELLAR_SEND=yes python3 scripts/invoke-gate-risc0-testnet.py`  

Gate ID on testnet: see **docs/SUBMISSION.md** (use `stellar contract alias show provekit-gate`, not stale env IDs).

## Repo map

| Path | Role |
|------|------|
| `contracts/gate` | Application gate + 8 tests (E2E on locked proofs) |
| `contracts/risc0-verifier` | On-chain Groth16 verify |
| `provekit/host` | `provekit-groth16-reencode`, RISC0 verify CLI |
| `artifacts/` | Locked invoke JSON + `policy_commitment.hex` |
| `docs/` | Submission, judging, security, integration |

## CI

GitHub Actions: risc0-verifier tests → build WASM → gate tests → host reencode → artifact verify.

## License

MIT — see [LICENSE](LICENSE).

## Links

- [INTEGRATION.md](docs/INTEGRATION.md)  
- [SECURITY.md](docs/SECURITY.md)  
- [JUDGING.md](docs/JUDGING.md) — self-score rubric  