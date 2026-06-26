# Demo policy (Path B) — for judges

## What the guest proves

The RISC Zero guest (`provekit/methods/guest/src/main.rs`) reads **private** `(score, threshold)` and commits a **public journal**:

| Field | Meaning |
|--------|---------|
| `policy_commitment` | `SHA-256(score ‖ threshold ‖ threshold_met)` |
| `threshold_met` | `1` if `score >= threshold`, else `0` |

The Groth16 proof (via ProveKit / RISC Zero STARK→SNARK) attests to a **valid RISC Zero claim** for that execution. The on-chain verifier checks the **RISC Zero wrapper** public inputs (claim digest, control root, etc.) — see `contracts/risc0-verifier`.

## What the gate enforces (application layer)

`verify_and_spend_risc0` adds **settlement rules** on top of verification:

1. **`policy_commitment` argument** must match the value stored at gate `initialize` (demo: `artifacts/policy_commitment.hex`).
2. **`nullifier`** must equal `SHA-256(proof_a ‖ proof_b ‖ proof_c ‖ public_inputs)` (proof-bound, one-shot spend).
3. **Cross-invoke** `risc0-verifier.verify_proof` must return true.

This is intentional separation: **cryptographic execution** (RISC0/Groth16) vs **which policy version this gate accepts** (stored commitment) vs **double-spend prevention** (nullifier map).

## Locked demo parameters

| Parameter | Value |
|-----------|--------|
| Demo score | `42` |
| Demo threshold | `100` |
| `threshold_met` in guest | `0` (fails threshold — still a valid proof of execution) |
| Policy commitment (hex) | `cf4e821a0332e6aafb89c29bf15a63971d3724459bdd8bd17b735854cdb8cd0d` |

Host writes `artifacts/policy_commitment.hex` when running `provekit-groth16-reencode` with the same demo constants.

## Production path (not required for hackathon demo)

To require **passing** policy in the product sense:

1. Guest already sets `threshold_met` from private inputs.
2. Extend gate or verifier to require `threshold_met == 1` (e.g. decode journal from claim, or add a dedicated public input after reprove).
3. Re-run Docker reprove + redeploy verifier VK if guest/ELF changes.

## Honest judge FAQ

**Q: Does the gate read `threshold_met` from the proof?**  
A: Not in this demo build. The proof proves the guest ran; the gate checks policy **commitment** + nullifier + Groth16. See `docs/SECURITY.md`.

**Q: Why does testnet spend succeed with `threshold_met = 0`?**  
A: Demo proves **end-to-end verification + nullifier replay** on Stellar testnet; policy commitment matches initialized demo policy.