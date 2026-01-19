#!/usr/bin/env bash
set -euo pipefail

if ! command -v hugo >/dev/null 2>&1; then
  echo "hugo not found. Install Hugo to run the dev server." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found. Install Python 3 to serve the built site." >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "npx not found. Install Node.js (which includes npx) to enable search in dev." >&2
  echo "Tip: https://nodejs.org/ or `brew install node`" >&2
  exit 1
fi

PORT="${PORT:-1313}"

echo "Optimizing images (responsive variants)…"
./scripts/optimize-images.sh || echo "Image optimization skipped/failed; originals may be served." >&2

run_pagefind() {
  rm -rf public/pagefind
  npx -y pagefind --site public --output-path public/pagefind
  if [[ ! -f public/pagefind/pagefind.js ]]; then
    echo "Pagefind completed but didn't produce public/pagefind/pagefind.js" >&2
    exit 1
  fi
}

echo "Cleaning previous build..."
rm -rf public

echo "Building once (so Pagefind can index)…"
hugo --minify --buildDrafts

echo "Indexing search (Pagefind)…"
run_pagefind >/dev/null 2>&1

echo "Starting Hugo watcher…"
hugo --minify --buildDrafts --watch &
HUGO_PID=$!

echo "Serving ./public at http://localhost:${PORT} …"
SERVER_LOG="$(mktemp -t aravindh.net.devserver.XXXXXX)"
python3 -m http.server "$PORT" --directory public >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

cleanup() {
  kill "$HUGO_PID" 2>/dev/null || true
  kill "$SERVER_PID" 2>/dev/null || true
  kill "${WATCH_PID:-}" 2>/dev/null || true
  rm -f "${SERVER_LOG:-}" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

sleep 0.2
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
  echo "Failed to start the local web server. Is PORT=${PORT} already in use?" >&2
  if [[ -f "$SERVER_LOG" ]]; then
    cat "$SERVER_LOG" >&2 || true
  fi
  exit 1
fi

if command -v fswatch >/dev/null 2>&1; then
  echo "Watching for rebuilt HTML and updating Pagefind index…"
  fswatch -o -e ".*" -i "\\.html$" public | while read -r _; do
    run_pagefind >/dev/null 2>&1 || true
  done &
  WATCH_PID=$!
else
  echo "Tip: install 'fswatch' for automatic indexing while you edit." >&2
fi

wait "$HUGO_PID"
