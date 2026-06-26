#!/usr/bin/env python3
"""Re-encode proof limbs from existing invoke JSON via seal hex (if present) or re-prove."""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Fast path: run host binary only if we add seal to json later.
print("Re-run: provekit-groth16-soroban after host rebuild", file=sys.stderr)
sys.exit(1)