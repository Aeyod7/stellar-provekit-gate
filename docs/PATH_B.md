# Path B — RISC Zero Groth16 on Soroban

Off-chain: RISC Zero **Groth16** receipt for the ProveKit guest. On-chain: fixed **BN254** verifying key with **5 public inputs** (RISC Zero wrapper circuit).

## Artifacts

| Piece | Location |
|-------|----------|
| VK generator | `scripts/gen_risc0_soroban_vk.py` (constants from `risc0-groth16` 3.0.4) |
| Soroban verifier | `contracts/risc0-verifier/` (`PUBLIC_INPUT_COUNT = 5`) |
| Host export | `provekit/host` bin `provekit-groth16-soroban` |
| Deploy | `scripts/deploy-risc0-verifier.sh` |

Path A (Poseidon / 1 public input) stays in `contracts/verifier/` for the hackathon interim demo.

## Generate VK

```bash
python3 scripts/gen_risc0_soroban_vk.py > contracts/risc0-verifier/src/vk_constants.rs
```

## Build & deploy verifier

```bash
./scripts/deploy-risc0-verifier.sh
```

Requires `stellar` CLI, funded `provekit-deployer` on testnet (`scripts/env.local`).

**Deployed testnet ID:** `CB6BHX3KGHVAEEBAAARM27YCK6VLKEDTR3KGPU6KVSJ7T3VKH7JDDLFV` (alias `provekit-risc0-verifier`)

## Docker (required for Groth16 prove)

RISC Zero only supports Groth16 proving via **Docker** (image `risczero/risc0-groth16-prover:v2025-04-03.1`) on this machine.

**Option A — WSL (one-time, needs your sudo password):**

```bash
chmod +x scripts/install-docker-wsl.sh
bash scripts/install-docker-wsl.sh
newgrp docker   # or reopen terminal
docker --version && docker info
```

**Option B — Docker Desktop on Windows:** install [Docker Desktop](https://docs.docker.com/desktop/setup/install/windows-install/), enable **WSL integration** for your Ubuntu distro. Then `docker` should work inside WSL.

Optional shim if only `docker.exe` exists: `export PATH="$PWD/scripts:$PATH"` and symlink or use `scripts/docker.sh`.

## Groth16 prove + Soroban JSON (host)

Needs **`risc0-groth16`** extension (Docker on this machine — no CUDA):

```bash
rzup install risc0-groth16
cd provekit/host
cargo run --release --bin provekit-groth16-soroban -- 42 100
```

Writes `demo/soroban_groth16_invoke.json` with `proof_a/b/c` and five `public_inputs` hex strings.

Local check: `risc0_groth16::Verifier::new(...).verify()` before invoking the contract.

## Invoke on testnet

```bash
stellar contract invoke \
  --id <RISC0_VERIFIER_ID> \
  --source-account provekit-deployer \
  --network testnet \
  -- verify_proof \
  --proof_a <hex> --proof_b <hex> --proof_c <hex> \
  --public_inputs '[<5x hex32>]'
```

## Public input layout

Matches `risc0_groth16::Verifier::new`:

1–2: `control_root` digest halves  
3–4: `claim_digest` halves  
5: `bn254_control_id` (digest bytes reversed, as in the verifier)

## Gate (Path B)

Deploy: `scripts/deploy-gate.sh` → `scripts/initialize-gate-admin.sh` → `scripts/init-gate-risc0-verifier.sh` → simulate spend:

```bash
provekit/target/release/provekit-groth16-reencode   # writes policy_commitment.hex
STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py
```

Hackathon bundle: `docs/SUBMISSION.md`

## Redeploy required (gate security upgrade)

Gate WASM changed: **admin-gated** `init_risc0_verifier`, **policy commitment** at `initialize`, **proof-bound nullifier**. Existing testnet gate ID (`CBZJBBDRQYXB2O4P4NBKI7WJVEKLYFMQJRUYY2CS2E5JMCGDMNXHY7CL`) does **not** include these checks.

After rebuild:

```bash
./scripts/deploy-gate.sh
./scripts/initialize-gate-admin.sh
./scripts/init-gate-risc0-verifier.sh
# Update SOROBAN_GATE_ID in scripts/env.local
STELLAR_SEND=no python3 scripts/invoke-gate-risc0-testnet.py
```

RISC0 verifier ID is unchanged unless guest ELF / VK regen (Docker prove).