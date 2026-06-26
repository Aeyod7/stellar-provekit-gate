# RISC Zero → Soroban integration

## Path B (complete)

Path B is **wired end-to-end**: RISC Zero Groth16 receipt → Soroban BN254 `verify_proof` → gate nullifier spend.

| Step | Location |
|------|----------|
| Guest policy | `provekit/methods/guest` |
| Host prove / reencode | `provekit/host` (`provekit-groth16-reencode`, `provekit-groth16-risc0-verify`) |
| On-chain verifier | `contracts/risc0-verifier` (5 public inputs) |
| App gate | `contracts/gate` (`verify_and_spend_risc0`) |

See **`docs/PATH_B.md`** for deploy/invoke and **`docs/SUBMISSION.md`** for testnet evidence.

## Pipeline

```text
STARK receipt  →  risc0-groth16 compress  →  Groth16 proof + public inputs  →  Soroban pairing_check
```

1. Guest proves `secret_score >= threshold`; journal commits `threshold_met` + SHA-256 **policy commitment**.
2. Host compresses seal and writes `artifacts/soroban_groth16_invoke.json` + `artifacts/policy_commitment.hex`.
3. Gate stores expected policy commitment at `initialize`; spend requires matching commitment + proof-bound nullifier.

## Fresh circuit / VK

New guest ELF → regen VK (`provekit-groth16-gen-vk`) → redeploy `risc0-verifier` and gate. Docker required for fresh Groth16 prove (`scripts/provekit-groth16-soroban.sh`).

## Path A (legacy interim)

`contracts/verifier/` + gate inline VK remain a **Poseidon preimage** stub for early demos; not the submission story.

## Boundless narrative

- **Today:** local RISC Zero prove + Soroban Groth16 verify on testnet.
- **Ship story:** same guest logic; prover network (Boundless) replaces local `default_prover()`; Stellar remains the trust-minimized verify layer.