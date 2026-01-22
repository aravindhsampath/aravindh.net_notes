#!/usr/bin/env bash
set -euo pipefail

# Publishes the built site to the server via rsync over SSH.
# Requirements: hugo, node (npx), rsync, ssh.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SSH_KEY="${SSH_KEY:-$HOME/.ssh/id_rsa_aravindh.net}"
SSH_TARGET="${SSH_TARGET:-root@49.12.190.41}"
DEST_DIR="${DEST_DIR:-/home/caddy/www/}"
BASEURL="${BASEURL:-https://aravindh.net/}"

if [[ ! -f "$SSH_KEY" ]]; then
  echo "SSH key not found: $SSH_KEY" >&2
  exit 1
fi

BASEURL="$BASEURL" "$SCRIPT_DIR/scripts/build.sh"

RSYNC_RSH=(ssh -i "$SSH_KEY")
rsync -az --delete -e "${RSYNC_RSH[*]}" "$SCRIPT_DIR/public/" "${SSH_TARGET}:${DEST_DIR}"
