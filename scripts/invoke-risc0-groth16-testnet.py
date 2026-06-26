#!/usr/bin/env python3
"""Invoke risc0-verifier verify_proof on testnet from artifacts/soroban_groth16_invoke.json.

Set STELLAR_SEND=no (default) to simulate only; STELLAR_SEND=yes to submit on-chain.
"""
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "artifacts" / "soroban_groth16_invoke.json"
ENV_LOCAL = ROOT / "scripts" / "env.local"


def main() -> int:
    if not ART.is_file():
        print(f"missing {ART}", file=sys.stderr)
        return 1
    data = json.loads(ART.read_text())
    verifier_id = os.environ.get("SOROBAN_RISC0_VERIFIER_ID", "").strip()
    if not verifier_id and ENV_LOCAL.is_file():
        for line in ENV_LOCAL.read_text().splitlines():
            line = line.strip()
            if line.startswith("SOROBAN_RISC0_VERIFIER_ID="):
                verifier_id = line.split("=", 1)[1].strip().strip('"').strip("'")
                break
    if not verifier_id:
        print("set SOROBAN_RISC0_VERIFIER_ID or scripts/env.local", file=sys.stderr)
        return 1

    inputs = data["public_inputs_hex"]
    # stellar CLI: Vec<BytesN<32>> as repeated --public_inputs or JSON array
    cmd = [
        "stellar",
        "contract",
        "invoke",
        "--send",
        os.environ.get("STELLAR_SEND", "no"),
        "--id",
        verifier_id,
        "--source-account",
        os.environ.get("SOURCE_ACCOUNT", "provekit-deployer"),
        "--network",
        os.environ.get("NETWORK", "testnet"),
        "--",
        "verify_proof",
        "--proof_a",
        data["proof_a_hex"],
        "--proof_b",
        data["proof_b_hex"],
        "--proof_c",
        data["proof_c_hex"],
        "--public_inputs",
        json.dumps(inputs),
    ]
    env = os.environ.copy()
    env.setdefault(
        "STELLAR_NETWORK_PASSPHRASE", "Test SDF Network ; September 2015"
    )
    env.setdefault("STELLAR_RPC_URL", "https://soroban-testnet.stellar.org")
    print("==> stellar contract invoke verify_proof")
    proc = subprocess.run(cmd, env=env, text=True, capture_output=True)
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())