# Testnet deployment (ProveKit Gate)

Deployed **2026-06-25** with identity `provekit-deployer` on **Test SDF Network**.

| Contract | ID | Lab |
|----------|-----|-----|
| **Verifier** (soroban-zk VK) | `CA6MRAJE3TKFM6PSGWTNRVCIFWKO26UP6LUDEL4FFKAXB466CQUII5JM` | [Stellar Lab](https://lab.stellar.org/r/testnet/contract/CA6MRAJE3TKFM6PSGWTNRVCIFWKO26UP6LUDEL4FFKAXB466CQUII5JM) |
| **Gate** (verify + nullifier) | `CAUQJXG5VAAPDCUFMIKG6OXFAJAIZCRB5XMDEC64NFKNISJP6WP75TZA` | [Stellar Lab](https://lab.stellar.org/r/testnet/contract/CAUQJXG5VAAPDCUFMIKG6OXFAJAIZCRB5XMDEC64NFKNISJP6WP75TZA) |
| **RISC0 verifier** (Groth16, 5 public inputs) | `CAIKW5434NZXXVPENP6ODUZ4BBGPCUPPNCHXYWYARRLR4KHQTJNNYYPN` | [Stellar Lab](https://lab.stellar.org/r/testnet/contract/CAIKW5434NZXXVPENP6ODUZ4BBGPCUPPNCHXYWYARRLR4KHQTJNNYYPN) |

Deployer public key: `GB74LU6GLYDCYGINSHPHXGDWGDHGJXPS7C5ZFVIY7IWWQRHO72CGESNO`

## On-chain proof (Path A demo)

Using `~/soroban-zk-ref/demo` against **our** verifier ID:

```bash
export PATH="$HOME/.local/bin:$HOME/.hermes/node/bin:$PATH"
cd ~/soroban-zk-ref/circuits/poseidon_preimage
# one-time: circom in ~/.local/bin (v2.2.3)
circom circuit.circom --r1cs --wasm --sym -o build -l ../../demo/node_modules

cd ~/soroban-zk-ref/demo
# .env: SOROBAN_RPC_URL, SOROBAN_CONTRACT_ID=<verifier id>, SOROBAN_SECRET_KEY
set -a && source .env && set +a
./node_modules/.bin/ts-node src/run.ts
```

**Verified run:**

- `txHash`: `05856c4cbd55d628f315f870ddad14f760be6ddaecd2fe89130363d4a1514c71`
- Explorer: https://stellar.expert/explorer/testnet/tx/05856c4cbd55d628f315f870ddad14f760be6ddaecd2fe89130363d4a1514c71
- Result: `✓ Proof verified on-chain: true` (ledger `3280404`)

## Redeploy

```bash
export SOURCE_ACCOUNT=provekit-deployer
./scripts/deploy-testnet.sh
```

Local env (no secrets in git): `scripts/env.local` — contract IDs only; fund key via `stellar keys fund`.