#!/usr/bin/env bash
# Prefer WSL docker; fall back to Docker Desktop's docker.exe on Windows.
set -euo pipefail
if command -v docker >/dev/null 2>&1; then
  exec docker "$@"
fi
WIN_DOCKER="/mnt/c/Program Files/Docker/Docker/resources/bin/docker.exe"
if [[ -x "$WIN_DOCKER" ]]; then
  exec "$WIN_DOCKER" "$@"
fi
echo "No docker found. Run: bash scripts/install-docker-wsl.sh" >&2
echo "Or install Docker Desktop for Windows + WSL integration." >&2
exit 127