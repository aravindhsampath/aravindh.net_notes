#!/usr/bin/env bash
set -euo pipefail

if ! command -v hugo >/dev/null 2>&1; then
  echo "hugo not found. Install Hugo to build the site." >&2
  exit 1
fi

HUGO_ARGS=(--minify)
if [[ -n "${BASEURL:-}" ]]; then
  HUGO_ARGS+=(--baseURL "$BASEURL")
fi

hugo "${HUGO_ARGS[@]}"

if command -v npx >/dev/null 2>&1; then
  rm -rf public/pagefind
  npx -y pagefind --site public --output-path public/pagefind
else
  cat >&2 <<'EOF'
npx not found. Install Node.js (which includes npx) and re-run.
Tip: https://nodejs.org/ or `brew install node`
EOF
  exit 1
fi
