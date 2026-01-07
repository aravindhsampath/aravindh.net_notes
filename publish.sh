#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/scripts/build.sh"

: "${RSYNC_DEST:?Set RSYNC_DEST (e.g. user@server:/var/www/aravindh.net/)}"

rsync -az --delete public/ "$RSYNC_DEST"

