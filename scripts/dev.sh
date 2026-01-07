#!/usr/bin/env bash
set -euo pipefail

if ! command -v hugo >/dev/null 2>&1; then
  echo "hugo not found. Install Hugo to run the dev server." >&2
  exit 1
fi

echo "Starting Hugo dev server (renders to disk so Pagefind can index)…"
hugo server --renderStaticToDisk --disableFastRender --buildDrafts &
HUGO_PID=$!

cleanup() {
  kill "$HUGO_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

if command -v pagefind >/dev/null 2>&1 && command -v fswatch >/dev/null 2>&1; then
  echo "Watching for rebuilt HTML and updating Pagefind index…"
  fswatch -o -e ".*" -i "\\.html$" public | while read -r _; do
    pagefind --site public --output-path public/pagefind >/dev/null 2>&1 || true
  done
else
  echo "Tip: install 'pagefind' and 'fswatch' for automatic indexing while you edit." >&2
fi

wait "$HUGO_PID"
