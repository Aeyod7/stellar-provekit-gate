# Pitch video script — ProveKit Gate (Path B)

**Target length:** 2:30–3:00  
**Tone:** Direct, technical, honest (hackathon demo — not mainnet audit)  
**Record:** Screen + voiceover (OBS / Loom). Optional face cam in intro/outro.

---

## Pre-record checklist

| Shot | What to capture |
|------|-----------------|
| A | Terminal: `./scripts/smoke-judge.sh` → `OK` |
| B | GitHub README + repo URL on screen |
| C | Stellar Expert: spend tx → return `true` |
| D | Stellar Expert: **same proof again** → return `false` (replay) |
| E | Simple architecture slide (or README diagram) |

**Links to have open:**

- Repo: https://github.com/Aeyod7/stellar-provekit-gate  
- Spend: https://stellar.expert/explorer/testnet/tx/2a9cb388fff608f66f1f3a303ad669025be607b017fc26677d52e310459295d8  
- Replay: https://stellar.expert/explorer/testnet/tx/4248be2854e11870dfae394852f95b9664779b8db229ebfb6ec9b517a7ad09f8  

---

## SCENE 1 — Hook (0:00–0:20)

**VISUAL:** Stellar testnet tx returning `true`, then cut to code / terminal.

**VO:**

> Real-world apps need to prove something happened in private — without dumping secrets on-chain.  
> We built **ProveKit Gate**: take a **RISC Zero** program, compress it to **Groth16**, and **verify it natively on Stellar Soroban** — then enforce one-shot settlement with a proof-bound nullifier.  
> This isn’t a mock verifier. We have **live testnet transactions** you can click.

---

## SCENE 2 — Problem (0:20–0:45)

**VISUAL:** Simple graphic: “Private score” → “Public proof” → “On-chain action”.

**VO:**

> Picture underwriting or access control: you want to prove a policy ran on sensitive inputs — like a score against a threshold — without publishing the score.  
> ZK gives you that proof. The hard part is **where** you verify: many chains can’t run a full RISC Zero prover inside a smart contract.  
> Our approach: prove off-chain with RISC Zero, ship a **Groth16 seal** Soroban can check with **BN254 pairings**, and layer **application rules** in a small gate contract.

---

## SCENE 3 — Architecture (0:45–1:15)

**VISUAL:** README architecture block or 4-box slide:

`Guest → Host reencode → risc0-verifier → gate`

**VO:**

> Four pieces, all in the repo.  
> **One:** a RISC Zero **guest** reads private score and threshold, and commits a public policy hash in the journal.  
> **Two:** our **host** re-encodes the Groth16 proof into Soroban-friendly limbs and five public inputs for the RISC Zero wrapper circuit.  
> **Three:** the **risc0-verifier** contract does on-chain Groth16 verification.  
> **Four:** the **gate** cross-invokes that verifier, checks the policy commitment you initialized at deploy time, and only then marks a spend — with a nullifier derived from the proof itself, so you can’t replay the same proof twice.

---

## SCENE 4 — Demo: local proof (1:15–1:45)

**VISUAL:** Terminal recording — `./scripts/smoke-judge.sh` scrolling (speed up 2× if long).

**VO:**

> Judges shouldn’t need our laptop. Run one script: **smoke-judge**.  
> It runs verifier tests, builds the verifier WASM, runs **eight gate tests** including end-to-end verification on **locked artifacts** in the repo, then re-encodes and checks the Groth16 invoke JSON.  
> If this passes, the cryptography and contracts match what we deployed on testnet.

**ON-SCREEN TEXT (caption):**  
`./scripts/smoke-judge.sh` — 5-minute judge path

---

## SCENE 5 — Demo: on-chain (1:45–2:25)

**VISUAL:** Stellar Expert — first tx success; second tx failure / `false`.

**VO:**

> On Stellar testnet we deployed the verifier and gate.  
> First invocation: **verify and spend** returns **true** — Groth16 checks out, policy commitment matches, nullifier recorded.  
> Submit the **same proof again**: the gate returns **false** — replay blocked.  
> I’ll leave these explorer links in the README and Dorahacks submission so you can verify without trusting the video.

**ON-SCREEN TEXT:**  
Gate: `CBVJSXUQ…` · Verifier: `CB6BHX3K…`  
Spend ✓ · Replay ✗

---

## SCENE 6 — Honest caveat (2:25–2:45)

**VISUAL:** `docs/DEMO_POLICY.md` or SECURITY.md bullet list.

**VO:**

> One honest line for judges: the Groth16 proof attests the **RISC Zero execution**; the gate separately checks **which policy version** this deployment accepts and enforces **one-shot spend**.  
> We document that layering in **DEMO_POLICY** and **SECURITY** — this is a **testnet-complete hackathon demo**, not a mainnet audit.  
> Production would bind threshold outcomes tighter in the guest and gate — the path is clear in the repo.

---

## SCENE 7 — Close (2:45–3:00)

**VISUAL:** GitHub repo + hackathon logo / Stellar branding.

**VO:**

> **ProveKit Gate** — RISC Zero to Groth16 to Soroban, with composable settlement.  
> Repo: **github.com/Aeyod7/stellar-provekit-gate**.  
> Run smoke-judge, click the txs, read SUBMISSION and DORAHACKS_PASTE.  
> Thanks — happy to walk through Path B live.

**ON-SCREEN TEXT:**  
https://github.com/Aeyod7/stellar-provekit-gate  
`./scripts/smoke-judge.sh`

---

## Full voiceover (single take, ~380 words)

Use this if you prefer one continuous read without scene breaks:

> Real-world apps need to prove something happened in private — without dumping secrets on-chain. We built ProveKit Gate: RISC Zero off-chain, Groth16 on Stellar Soroban, plus a gate for policy commitment and proof-bound nullifiers — with live testnet transactions.
>
> Think access control or underwriting: prove a policy ran on sensitive inputs without publishing them. ZK gives you the proof; the hard part is verifying on-chain. We prove with RISC Zero, re-encode to a Groth16 seal Soroban checks via BN254 pairings, and add application rules in a gate contract.
>
> The guest commits a public policy hash from private score and threshold. The host exports proof limbs and five public inputs. The risc0-verifier contract verifies Groth16. The gate cross-invokes it, checks your initialized policy commitment, and spends once — nullifier equals a hash of the proof, so replay fails.
>
> Locally, run ./scripts/smoke-judge.sh: eight gate tests on locked artifacts, verifier WASM, re-encode — one judge path.
>
> On testnet, first spend returns true; the same proof again returns false. Explorer links are in SUBMISSION.md.
>
> Honest caveat: execution is proved by Groth16; policy version and replay rules are enforced by the gate — documented in DEMO_POLICY. Testnet demo, not mainnet audit.
>
> Repo: github.com/Aeyod7/stellar-provekit-gate. Thanks.

---

## Optional B-roll (no voice)

- Scroll `provekit/methods/guest/src/main.rs` (journal commit)
- Flash `artifacts/soroban_groth16_invoke.json` (proof + public_inputs)
- CI green on GitHub Actions

---

## Recording tips

1. **1080p terminal** — font 16–18pt, dark theme, high contrast.  
2. **Speed up** smoke-judge middle section; keep final `OK` at real time.  
3. **Zoom** Stellar Expert return value / events if visible.  
4. **Don’t** read contract IDs aloud — show on screen only.  
5. Export **MP4 < 100MB** if the form has a size cap; upload unlisted YouTube + link in Dorahacks if needed.