# Devin handoff — Hackathon win score + security + hygiene

**Repo (local):** `/home/aeyo/stellar-provekit-gate`  
**Model:** Use **GLM 5.2** (free tier) in Devin if selectable.  
**Do not touch:** `scripts/env.local` (secrets). Do not commit keys.  
**Video:** Out of scope for this task.

---

## Mission

Raise submission quality across **four rubric pillars** (align with Dorahacks / Stellar Hacks “Real-World ZK”):

| Pillar | Target |
|--------|--------|
| **Technical depth** | Path B E2E stays green; add tests + CI smoke |
| **Security / design** | Admin-gated verifier init; proof-bound nullifier |
| **Correctness / product** | On-chain policy commitment check; guest commitment upgrade |
| **Engineering hygiene** | Docs sync, `JUDGING.md`, GitHub Actions |

---

## Success criteria (must all pass)

```bash
export PATH="/home/aeyo/.local/bin:/home/aeyo/.cargo/bin:$PATH"
cd /home/aeyo/stellar-provekit-gate

# Host / artifacts
cargo build --release --manifest-path provekit/host/Cargo.toml
provekit/target/release/provekit-groth16-reencode
provekit/target/release/provekit-groth16-risc0-verify

# Contracts
cd contracts/risc0-verifier && cargo test && cd ../..
cd contracts/gate && cargo test && cd ../..

# Optional if host tests exist
cargo test --manifest-path provekit/host/Cargo.toml 2>/dev/null || true
```

- [ ] `docs/INTEGRATION.md` reflects Path B complete (not “Day 2–3 pending”).
- [ ] `docs/SUBMISSION.md` guest path = `provekit/methods/guest` (not `guests/policy-threshold`).
- [ ] New `docs/JUDGING.md` maps features → typical hackathon criteria (innovation, technical, UX/demo script, completeness).
- [ ] New `docs/SECURITY.md` — testnet limits + what was fixed (admin, nullifier binding).
- [ ] `.github/workflows/ci.yml` runs contract tests + `provekit-groth16-reencode` (no Docker).
- [ ] Gate contract changes documented with **redeploy required** section in `PATH_B.md`.

**Testnet re-verify (after redeploy):** user runs with `scripts/env.local`; you may sim-only in docs.

---

## 1. Security / design (gate + scripts)

### 1a. Admin + one-time `init_risc0_verifier`

**File:** `contracts/gate/src/lib.rs`

- Add `DataKey::Admin` (Address).
- Add `initialize(admin: Address)` — sets admin once; panic if already set.
- Change `init_risc0_verifier(verifier_id: Address)` → require `admin.require_auth()` and caller == stored admin.
- Change `init_risc0_verifier` to panic if `Risc0VerifierId` already set (one-time).
- Update `scripts/deploy-gate.sh` / new `scripts/initialize-gate-admin.sh` to call `initialize` with source account after deploy.
- Update `scripts/init-gate-risc0-verifier.sh` to pass `--source-account` auth for admin.

### 1b. Proof-bound nullifier (no replay with new nullifier)

**File:** `contracts/gate/src/lib.rs` — `verify_and_spend_risc0`

- Compute `proof_id: BytesN<32>` = SHA-256 over concatenation of `proof_a`, `proof_b`, `proof_c`, and all `public_inputs` (use Soroban `env.crypto().sha256` on assembled bytes in contract).
- Require `nullifier == proof_id` (or rename arg to `proof_id` and document). Panic otherwise.
- Update `scripts/invoke-gate-risc0-testnet.py` to derive nullifier from same hash as contract (document algorithm in script header).
- Remove `GATE_NULLIFIER_HEX` random override for real spends; optional env only for negative tests.

---

## 2. Correctness / product

### 2a. Policy commitment on gate

- Add `DataKey::PolicyCommitment` (BytesN<32>), set in `initialize` or `set_policy_commitment` (admin-only, once).
- Add host helper or script step: read journal commitment from prove output / `artifacts/` (e.g. hash of `ProveGateOutput` commitment field from guest).
- `verify_and_spend_risc0` adds arg `policy_commitment: BytesN<32>`; must equal stored value (proves gate expects this policy circuit output).
- Document in `SUBMISSION.md`: Soroban verifies RISC0 Groth16 wrapper; **policy commitment** ties spend to guest journal binding.

### 2b. Guest commitment (demo → stronger)

**File:** `provekit/methods/guest/src/main.rs`

- Replace toy `commitment_for` with **SHA-256** over canonical bytes `(score, threshold, threshold_met)` using `sha2` in guest (add dep in guest `Cargo.toml` if needed — RISC0 guest allows sha2).
- Re-run note in docs: new ELF → regen VK → redeploy `risc0-verifier` (may be out of scope if Docker missing; still update guest code + document).

### 2c. Host artifact for policy commitment

- After reencode, write `artifacts/policy_commitment.hex` from known demo prove inputs (`secret_score=42`, `threshold=100` → commitment from guest logic).
- Invoke scripts read this file for gate call.

---

## 3. Engineering hygiene

### 3a. Gate tests

**File:** `contracts/gate/src/test.rs` or `#[cfg(test)]` module

- Test `proof_id` / nullifier binding logic (unit test pure fn if extracted).
- Test admin gate on `init_risc0_verifier` using `soroban-sdk` testutils addresses.

### 3b. Docs

- Rewrite `docs/INTEGRATION.md` — Path B complete, pointer to `PATH_B.md` + `SUBMISSION.md`.
- Patch `README.md` — short “Judging highlights” bullet list + link `JUDGING.md`.

### 3c. CI

**File:** `.github/workflows/ci.yml`

```yaml
# on push/pr: rust toolchain, cargo test risc0-verifier + gate, build host release, run reencode
```

No secrets. No testnet in CI.

---

## 4. Submission score copy (no video)

**File:** `docs/SUBMISSION.md` — add section **“Judge checklist (self-score)”** with honest 1–5 per category and evidence links (existing explorer txs).

**File:** `docs/JUDGING.md` — map:

- Innovation: RISC0 receipt → Stellar native Groth16
- Technical: pairing + 5 public inputs + cross-contract gate
- Real-world ZK: private score, public threshold_met + commitment
- Completeness: scripts, locked artifacts, explorer proofs

---

## Files likely touched

| File | Change |
|------|--------|
| `contracts/gate/src/lib.rs` | admin, policy commitment, proof-bound nullifier |
| `contracts/gate/src/test.rs` | new tests |
| `provekit/methods/guest/src/main.rs` | sha256 commitment |
| `scripts/invoke-gate-risc0-testnet.py` | nullifier + policy commitment |
| `scripts/initialize-gate-admin.sh` | new |
| `scripts/deploy-gate.sh` | note initialize |
| `docs/*` | INTEGRATION, SUBMISSION, JUDGING, SECURITY, PATH_B |
| `.github/workflows/ci.yml` | new |

---

## Out of scope

- Demo video
- Docker fresh Groth16 prove (unless you have Docker)
- Mainnet deploy
- Boundless prover integration (README one-liner only)

---

## Devin paste prompt (one paragraph)

Implement `docs/DEVIN_HANDOFF_WIN_SCORE.md` in `/home/aeyo/stellar-provekit-gate`: admin-gated `init_risc0_verifier`, proof-bound nullifier, policy commitment on gate, stronger guest SHA-256 commitment, gate tests, CI workflow, and doc updates (`JUDGING.md`, `SECURITY.md`, fix INTEGRATION/SUBMISSION paths). Run all Success criteria commands; leave testnet redeploy instructions in `PATH_B.md` if contract IDs change. Do not commit `scripts/env.local`.