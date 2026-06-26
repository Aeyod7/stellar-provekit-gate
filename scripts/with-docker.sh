#!/usr/bin/env bash
# Run a command with docker socket access (group added via: wsl -u root usermod -aG docker $USER)
set -euo pipefail
export PATH="/usr/bin:$HOME/.risc0/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
exec sg docker -c "$*"