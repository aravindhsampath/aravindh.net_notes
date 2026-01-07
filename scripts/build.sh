#!/usr/bin/env bash
set -euo pipefail

if ! command -v hugo >/dev/null 2>&1; then
  echo "hugo not found. Install Hugo to build the site." >&2
  exit 1
fi

hugo --minify

if command -v pagefind >/dev/null 2>&1; then
  pagefind --site public --output-path public/pagefind
else
  cat >&2 <<'EOF'
pagefind not found. Install Pagefind and re-run:
  https://pagefind.app/
EOF
  exit 1
fi

