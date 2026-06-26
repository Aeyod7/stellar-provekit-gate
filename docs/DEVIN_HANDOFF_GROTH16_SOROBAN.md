# Devin handoff — RISC0 Groth16 → Stellar Soroban `verify_proof`

**Repo:** `/home/aeyo/stellar-provekit-gate`  
**Hermes request:** Fix end-to-end; use **GLM 5.2 free** (or equivalent) for this session if Devin model is selectable.

---

## Success criteria (all required)

1. `provekit-groth16-risc0-verify` → **`risc0_groth16::Verifier::verify OK`** (already passing).
2. `cd contracts/risc0-verifier && cargo test verify_artifacts_groth16_invoke_json -- --nocapture` → **pass**.
3. Testnet `scripts/invoke-risc0-groth16-testnet.py` → **`verify_proof` succeeds** (not `Contract #1` / pairing failure).
4. Update `artifacts/soroban_groth16_invoke.json` if proof or public inputs change.

**Do not** submit raw RISC0 `Seal::to_vec()` to Soroban. Re-encode via `provekit/host` (`host_stellar.rs`, `soroban_groth16.rs`).

---

## Current state (2026-06-26)

| Check | Status |
|-------|--------|
| Off-chain RISC0 Groth16 | **OK** (`provekit-groth16-risc0-verify`) |
| Local Soroban `verify_artifacts` | **OK** (`verify_artifacts_groth16_invoke_json`) |
| Testnet invoke | **OK** (`scripts/invoke-risc0-groth16-testnet.py` → `true`; tx on explorer) |

**Testnet risc0-verifier:** `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3KGPU6KVSJ7T3VKH7JDDLFV`

**Artifacts (prove `42`, `100`, seal from `groth16_seal.bin`):**
- `artifacts/soroban_groth16_invoke.json`
- `artifacts/groth16_seal.bin`
- `artifacts/groth16_claim_digest.hex`

**Contract test:** `verify_artifacts_groth16_invoke_json` reads `proof_a_hex`, `proof_b_hex`, `proof_c_hex`, `public_inputs_hex` from that JSON.

---

## Fixes already landed (verify, don’t redo blindly)

### 1. Public inputs (was wrong — **fixed in code**)

`provekit/host/src/soroban_groth16.rs` — `risc0_five_public_inputs` now matches **`risc0_groth16::Verifier::new`**:

- `split_digest(control_root)` → two Fr limbs (not duplicating full control_root twice).
- `split_digest(claim_digest)` → two Fr limbs.
- 5th input: **`bn254_control_id` bytes reversed**, single Fr (not split).

Re-run: `provekit-groth16-reencode` — `public_inputs_hex` should show **five distinct** 32-byte hex strings.

### 2. VK G2 limb order (Solidity / contract)

`provekit/host/src/risc0_vk_decode.rs` — G2 from `verifier.rs` constants uses **`[X1, X2], [Y1, Y2]`** (`risc0_g2_from_sol_const`), not snarkjs-swapped order.

### 3. Crate layout

- `provekit/host/src/lib.rs` exports `host_stellar`, `soroban_groth16`, `risc0_vk_decode`.
- Bins: `provekit-groth16-reencode`, `provekit-groth16-gen-vk`, `provekit-groth16-risc0-verify`.

### 4. Encoding path

- Proofs: RISC0 seal (ark 0.5) → uncompressed → ark **0.4** → `host_stellar` `stellar_g1` / `stellar_g2` (matches `soroban-env-host` BN254; G2 uses **64-byte Fp2 chunk reversal** per `bn254.rs`).
- VK: decimals from `contracts/risc0-verifier/src/verifier.rs` → `risc0_vk_decode` → same `host_stellar` encodings → `vk_constants.rs`.

---

## Open blockers (your focus)

### A. `provekit-groth16-gen-vk` still fails

Last error: **`invalid data`** during **alpha G1** decode in `risc0_g1_from_limbs` (backtrace pointed at `g2_from_elem` / deserialize — likely stale binary or `from_u256` padding mismatch).

**Action:** Finish aligning `from_u256_decimal` in `risc0_vk_decode.rs` with RISC0’s `from_u256` in `risc0-groth16-3.0.4/src/verifier.rs`. Rebuild:

```bash
export PATH="/home/aeyo/.cargo/bin:$PATH"
cd /home/aeyo/stellar-provekit-gate/provekit/host
cargo build --release --bin provekit-groth16-gen-vk
./target/release/provekit-groth16-gen-vk
```

Output must overwrite `contracts/risc0-verifier/src/vk_constants.rs`.

**Alternative:** If limb decode stays painful, derive Stellar VK bytes from `risc0_groth16::verifying_key()` inner ark VK (serialize G1/G2 points) → `host_stellar::stellar_g1/g2` — same bytes contract needs.

### B. `vk_constants.rs` may be stale

May still reflect Python `scripts/gen_risc0_soroban_vk.py` (snarkjs-style) partial mismatch. **Authoritative on-chain VK** = output of **successful** `provekit-groth16-gen-vk`.

### C. Local test still fails after public-input fix

Implies **VK constants** and/or **proof Stellar encoding** mismatch while RISC0 native verify passes.

**Debug order:**
1. Regenerate VK (`gen-vk` green).
2. `provekit-groth16-reencode` → refresh JSON.
3. `cargo test verify_artifacts_groth16_invoke_json` in `contracts/risc0-verifier`.
4. If fail: compare `Fr::from_bytes` in contract vs `fr_repr_to_soroban_bytes` in `soroban_groth16.rs`; compare VK `VK_BETA_G2` etc. to host `G2::from_bytes` in a small Rust bin.

---

## Commands (copy-paste)

```bash
export PATH="/home/aeyo/.cargo/bin:$PATH"
ROOT=/home/aeyo/stellar-provekit-gate

# Off-chain sanity
$ROOT/provekit/target/release/provekit-groth16-risc0-verify

# Regenerate invoke JSON (after claim hex + seal exist)
$ROOT/provekit/target/release/provekit-groth16-reencode

# Regenerate VK (must succeed)
$ROOT/provekit/target/release/provekit-groth16-gen-vk

# Local Soroban test
cd $ROOT/contracts/risc0-verifier
cargo test verify_artifacts_groth16_invoke_json -- --nocapture

# Deploy + testnet (only after local green; secrets in scripts/env.local)
# source scripts/env.local && ./scripts/deploy-risc0-verifier.sh
# python3 scripts/invoke-risc0-groth16-testnet.py
```

**Full reprove (slow, Docker):** `provekit-groth16-soroban 42 100` from `provekit/host`.

---

## Key files

| Path | Role |
|------|------|
| `provekit/host/src/soroban_groth16.rs` | Public inputs + `build_soroban_invoke` |
| `provekit/host/src/host_stellar.rs` | Stellar BN254 bytes (ark 0.4) |
| `provekit/host/src/risc0_vk_decode.rs` | VK limb decode |
| `contracts/risc0-verifier/src/vk_constants.rs` | On-chain VK |
| `contracts/risc0-verifier/src/lib.rs` | `verify_proof`, `verify_artifacts` test |
| `contracts/risc0-verifier/src/verifier.rs` | Decimal VK source |
| `soroban-env-host-25.0.1/src/crypto/bn254.rs` | Host G2 Fp2 reversal reference |

---

## Env / safety

- Stellar secrets: `scripts/env.local` (do not commit).
- **Testnet only** unless user explicitly asks mainnet.
- Cargo: `/home/aeyo/.cargo/bin/cargo`.

---

## Report back to Aeyo

When done: short summary + **test command output** (local test green + testnet tx hash / success). No “should work” without logs.