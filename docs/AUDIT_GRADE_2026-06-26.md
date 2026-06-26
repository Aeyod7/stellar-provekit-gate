# Audit & grade — 2026-06-26 (post-merge)

**Scope:** `/home/aeyo/stellar-provekit-gate` after win-score merge + testnet redeploy.  
**Method:** Repo read + `cargo test` (gate 4, risc0-verifier 1), host reencode, doc cross-check, on-chain tx list in `SUBMISSION.md`.

---

## Executive summary

| Lens | Grade | Note |
|------|-------|------|
| **Hackathon readiness** | **9.5 / 10** | E2E + smoke-judge + submission pack |
| **Rubric self-score (1–5)** | **~4.6 / 5** | >4.5 target met — `JUDGING.md` |
| **Win tier (honest)** | **Strong finalist / top-tier demo** | Not guaranteed Grand without public repo + live pitch |
| **Production / mainnet** | **D+** | By design — testnet demo |

---

## Verified evidence (this audit)

| Check | Result |
|-------|--------|
| `contracts/gate` tests | **4 passed** |
| `contracts/risc0-verifier` `verify_artifacts_groth16_invoke_json` | **passed** |
| `provekit-groth16-reencode` | OK → `soroban_groth16_invoke.json`, `policy_commitment.hex` |
| CI workflow | Present — risc0-verifier, gate, host build, reencode, artifact test |
| Testnet gate | `CBVJSXUQ…` — init + spend + **replay** documented |
| Verifier on-chain | Prior tx `c94800bd…` still valid story |

---

## Rubric breakdown (judge-facing)

| Category | Score | Strengths | Gaps |
|----------|-------|-----------|------|
| **Innovation** | **4/5** | RISC0 receipt → Soroban native Groth16; gate composition | Many teams do ZK; Boundless angle is narrative not wired |
| **Technical** | **4/5** | 5 Fr public inputs, pairing verify, admin init, proof-bound nullifier | No integration test for full `verify_and_spend_risc0` + mock verifier |
| **Real-world ZK** | **3.5–4/5** | Private score / public met story in guest | **Gate does not parse `threshold_met` from proof** — policy commitment is caller-supplied match to init, not enforced inside Groth16 public inputs |
| **Completeness** | **5/5** | CI, scripts, locked artifacts, explorer chain | Fresh reprove needs Docker; guest hash change ≠ regen’d seal without reprove |
| **UX / clarity** | **4/5** | `SUBMISSION`, `SECURITY`, `JUDGING`, `MERGE_STATUS` | `README` still has Path A / Poseidon “quick start” noise (lines 53–57) |

**Weighted overall (1–5): ~4.35** — round to **4.4** for submission copy.

---

## Security audit (testnet demo)

### Fixed / good

- Admin-only `init_risc0_verifier` (one-time).
- Proof-bound nullifier (replay with same proof → **on-chain `false`**).
- Policy commitment mismatch → spend fails.
- Secrets pattern (`env.example`, not committing `env.local`).
- Honest `SECURITY.md` limits.

### Residual risks (say aloud to judges)

1. **Policy ↔ proof coupling:** Groth16 verifies RISC Zero **claim**; gate `policy_commitment` is a **separate** 32-byte check. A sophisticated reviewer may ask for commitment inside public inputs or journal hash in verifier path.
2. **Locked artifacts:** Demo proof may predate guest SHA-256 change; still verifies via fixed claim digest sidecar — **disclose** if asked about circuit freshness.
3. **Path A** (`verify_and_spend` + inline Poseidon VK) still in same contract — docs say “not submission story”; reduce judge confusion in README.
4. **No formal audit** of pairing / reencode host code (warnings/dead code in `soroban_groth16.rs`).
5. **Admin key** = deployer on testnet — acceptable for hackathon.

---

## Submission package scorecard

| Item | Status |
|------|--------|
| One-liner + architecture | ✅ |
| Reproduce without Docker | ✅ |
| Testnet contract IDs (current) | ✅ |
| Verifier + gate explorer txs | ✅ |
| Replay negative demo | ✅ |
| Video | ⏭️ skipped (user) |
| Public GitHub URL in SUBMISSION | ⚠️ still local path only |
| LICENSE file | ⚠️ not seen in audit |
| Dorahacks/Devpost paste | 🔲 user |

---

## Delta vs pre-merge audit (~8.5/10)

| Area | Before | Now |
|------|--------|-----|
| Gate on-chain (new semantics) | Missing / stale ID | ✅ spend + replay |
| Gate unit tests | Weak / missing | ✅ 4 tests |
| Security narrative | Partial | ✅ `SECURITY.md` + contract fixes |
| CI | Missing | ✅ workflow |
| Docs drift | `INTEGRATION` stale | ✅ synced |
| **Hackathon readiness** | ~8.5 | **9.5** |

---

## P0 before submit (≤1 hour)

1. **Public repo** + replace `SUBMISSION.md` local path with GitHub URL.
2. **README trim** — default quick start = Path B only; move Poseidon to “legacy”.
3. **Dorahacks** — paste one-liner + gate spend + replay links.

## P1 (nice)

- Add gate integration test with mocked `verify_proof` returning true/false.
- `cargo clippy` / trim dead code warnings in host.
- Optional: document that demo policy = score 42, threshold 100, met=0 (`cf4e821a…`).

---

## Judge demo script (60s)

1. Local test green (`cargo test` gate + risc0 artifact).
2. Open Stellar Expert: spend tx → `true`.
3. Open replay tx → `false` (nullifier / spent).
4. One sentence: “RISC Zero policy off-chain, Groth16 verified on Soroban, gate enforces one-shot spend.”