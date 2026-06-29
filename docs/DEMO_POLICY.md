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
2. **Guest-claim binding** — the proof's `claim_digest` limbs (public inputs index 2,3) must equal the value baked into the gate at build time. `claim_digest = SHA-256(image_id, journal_digest, …)`, so this binds acceptance to **exactly this guest program and this journal** — it is not satisfiable by an arbitrary valid RISC Zero proof.
3. **`nullifier`** must equal `SHA-256(proof_a ‖ proof_b ‖ proof_c ‖ public_inputs)` (proof-bound, one-shot spend).
4. **Cross-invoke** `risc0-verifier.verify_proof` must return true (BN254 Groth16 pairing).

This is intentional separation: **cryptographic execution** (RISC0/Groth16) vs **which program + policy this gate accepts** (claim-digest + stored commitment) vs **double-spend prevention** (nullifier map).

## Locked demo parameters

| Parameter | Value |
|-----------|--------|
| Demo score | `42` |
| Demo threshold | `100` |
| `threshold_met` in guest | `0` (fails threshold — still a valid proof of execution) |
| Policy commitment (hex) | `cf4e821a0332e6aafb89c29bf15a63971d3724459bdd8bd17b735854cdb8cd0d` |

Host writes `artifacts/policy_commitment.hex` when running `provekit-groth16-reencode` with the same demo constants.

## Production path (not required for hackathon demo)

To require **passing** policy in the product sense, two equivalent routes:

1. **Pin a passing claim.** The guest already sets `threshold_met` from private inputs; the `claim_digest` binding already pins the *whole journal*. Reprove with passing inputs (`threshold_met == 1`) and bake that `claim_digest` into the gate — only proofs of a passing execution then spend.
2. **Decode in-circuit / add a public input.** Add a dedicated `threshold_met` public input (or decode the journal on-chain) and require `== 1`, so a single deployment accepts any passing proof. Needs Docker reprove + redeploy if the guest/ELF changes.

## Honest judge FAQ

**Q: Is the proof bound to this specific guest program?**  
A: Yes. The gate pins the expected RISC Zero `claim_digest` (image_id + journal), so an arbitrary valid RISC Zero proof of a different program is rejected before the pairing check. See `contracts/gate/src/lib.rs` and `docs/SECURITY.md`.

**Q: Does the gate branch on `threshold_met`?**  
A: It does not read the field as a variable, but pinning `claim_digest` cryptographically binds acceptance to the exact journal (which contains `threshold_met`). The demo intentionally pins the `threshold_met = 0` execution to show end-to-end verification + replay without claiming the threshold passed; pinning a passing claim (route 1 above) flips it to a true policy gate.

**Q: Why does the testnet spend succeed with `threshold_met = 0`?**  
A: The demo proves **end-to-end verification + nullifier replay** on Stellar testnet; the policy commitment and claim digest match the initialized demo execution.