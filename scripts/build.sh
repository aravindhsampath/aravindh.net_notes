#!/usr/bin/env bash
set -euo pipefail

if ! command -v hugo >/dev/null 2>&1; then
  echo "hugo not found. Install Hugo to build the site." >&2
  exit 1
fi

echo "Optimizing images (responsive variants)…" >&2
./scripts/optimize-images.sh || echo "Image optimization skipped/failed; originals may be served." >&2

HUGO_ARGS=(--minify)
if [[ -n "${BASEURL:-}" ]]; then
  HUGO_ARGS+=(--baseURL "$BASEURL")
fi

echo "Cleaning previous build..."
rm -rf public

hugo "${HUGO_ARGS[@]}"

if ! command -v npx >/dev/null 2>&1; then
  echo "npx not found. Install Node.js (which includes npx) and re-run." >&2
  echo "Tip: https://nodejs.org/ or \`brew install node\`" >&2
  exit 1
fi

rm -rf public/pagefind
npx -y pagefind --site public --output-path public/pagefind
