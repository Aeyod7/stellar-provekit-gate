#!/usr/bin/env bash
# Install Docker engine in WSL (Ubuntu). Requires your sudo password once.
# Alternative: install Docker Desktop on Windows and enable WSL integration — then skip this script.
set -euo pipefail

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  echo "Docker already works: $(docker --version)"
  exit 0
fi

echo "==> Installing docker.io + compose plugin (Ubuntu)"
sudo apt-get update -qq
sudo apt-get install -y docker.io docker-compose-v2

echo "==> Starting docker service"
sudo service docker start || sudo systemctl start docker 2>/dev/null || true

echo "==> Adding user $USER to docker group (log out/in or: newgrp docker)"
sudo usermod -aG docker "$USER"

echo ""
echo "Done. Run ONE of:"
echo "  newgrp docker"
echo "  # or close and reopen this WSL terminal"
echo "Then verify:"
echo "  docker --version && docker info"
echo ""
echo "Groth16 prover image (pulled on first prove):"
echo "  risczero/risc0-groth16-prover:v2025-04-03.1"