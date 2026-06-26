#!/usr/bin/env python3
"""
Invoke gate verify_and_spend_risc0 on testnet (delegates to risc0-verifier).

Nullifier algorithm (must match contracts/gate `compute_proof_id`):
  SHA-256( proof_a_bytes || proof_b_bytes || proof_c_bytes || each public_input_32_bytes )
  where proof limbs are the raw hex-decoded bytes from soroban_groth16_invoke.json.

Policy commitment: read artifacts/policy_commitment.hex (demo: score=42, threshold=100).
Set GATE_NULLIFIER_HEX only for negative tests (wrong nullifier → false).
"""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "artifacts" / "soroban_groth16_invoke.json"
POLICY_ART = ROOT / "artifacts" / "policy_commitment.hex"
ENV_LOCAL = ROOT / "scripts" / "env.local"


def _env_var(name: str) -> str:
    val = os.environ.get(name, "").strip()
    if val:
        return val
    if ENV_LOCAL.is_file():
        for line in ENV_LOCAL.read_text().splitlines():
            line = line.strip()
            if line.startswith(f"{name}="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    return ""


def _hex_to_bytes(h: str) -> bytes:
    return bytes.fromhex(h.removeprefix("0x"))


def _stellar_alias_gate() -> str:
    network = os.environ.get("NETWORK", "testnet")
    try:
        out = subprocess.run(
            ["stellar", "contract", "alias", "show", "provekit-gate", "--network", network],
            capture_output=True,
            text=True,
            check=False,
        )
        if out.returncode != 0:
            return ""
        # last line is contract id
        lines = [ln.strip() for ln in out.stdout.splitlines() if ln.strip()]
        return lines[-1] if lines else ""
    except OSError:
        return ""


def resolve_gate_id() -> str:
    gate_id = _env_var("SOROBAN_GATE_ID")
    alias = _stellar_alias_gate()
    if alias and gate_id and gate_id != alias:
        print(
            f"Note: SOROBAN_GATE_ID ({gate_id}) != alias provekit-gate ({alias}); using alias",
            file=sys.stderr,
        )
        return alias
    return gate_id or alias


def compute_proof_id(data: dict) -> str:
    buf = b"".join(
        [
            _hex_to_bytes(data["proof_a_hex"]),
            _hex_to_bytes(data["proof_b_hex"]),
            _hex_to_bytes(data["proof_c_hex"]),
            *(_hex_to_bytes(pi) for pi in data["public_inputs_hex"]),
        ]
    )
    return hashlib.sha256(buf).hexdigest()


def main() -> int:
    if not ART.is_file():
        print(f"missing {ART}", file=sys.stderr)
        return 1
    if not POLICY_ART.is_file():
        print(f"missing {POLICY_ART} — run provekit-groth16-reencode", file=sys.stderr)
        return 1
    gate_id = resolve_gate_id()
    if not gate_id:
        print("set SOROBAN_GATE_ID, scripts/env.local, or provekit-gate stellar alias", file=sys.stderr)
        return 1

    data = json.loads(ART.read_text())
    nullifier = os.environ.get("GATE_NULLIFIER_HEX") or compute_proof_id(data)
    policy_commitment = POLICY_ART.read_text().strip()

    cmd = [
        "stellar",
        "contract",
        "invoke",
        "--send",
        os.environ.get("STELLAR_SEND", "no"),
        "--id",
        gate_id,
        "--source-account",
        os.environ.get("SOURCE_ACCOUNT", "provekit-deployer"),
        "--network",
        os.environ.get("NETWORK", "testnet"),
        "--",
        "verify_and_spend_risc0",
        "--nullifier",
        nullifier,
        "--policy_commitment",
        policy_commitment,
        "--proof_a",
        data["proof_a_hex"],
        "--proof_b",
        data["proof_b_hex"],
        "--proof_c",
        data["proof_c_hex"],
        "--public_inputs",
        json.dumps(data["public_inputs_hex"]),
    ]
    env = os.environ.copy()
    env.setdefault("STELLAR_NETWORK_PASSPHRASE", "Test SDF Network ; September 2015")
    env.setdefault("STELLAR_RPC_URL", "https://soroban-testnet.stellar.org")
    print("==> stellar contract invoke verify_and_spend_risc0")
    proc = subprocess.run(cmd, env=env, text=True, capture_output=True)
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())