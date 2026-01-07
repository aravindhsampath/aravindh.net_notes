#!/usr/bin/env bash
set -euo pipefail

# Generate responsive image variants for images in `static/images/`.
# - Writes variants to `static/images/gen/` (gitignored).
# - Uses `vips` for raster images and `svgo` for SVGs.
#
# Supported:
# - jpg/jpeg: resized + re-encoded at JPEG_QUALITY (default 72) + WebP variants
# - png: resized (keeps png) + WebP variants
# - webp: resized (keeps webp)
# - heic/heif: resized + converted to jpg + WebP variants
# - svg: optimized copy (no resizing)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGES_DIR="$ROOT_DIR/static/images"
OUT_DIR="$IMAGES_DIR/gen"

if [[ ! -d "$IMAGES_DIR" ]]; then
  exit 0
fi

if ! command -v vips >/dev/null 2>&1; then
  echo "vips not found; cannot optimize images." >&2
  exit 1
fi

if ! command -v svgo >/dev/null 2>&1; then
  echo "svgo not found; cannot optimize SVGs." >&2
  exit 1
fi

IFS=',' read -r -a WIDTHS <<< "${IMAGE_WIDTHS:-480,960,1440}"
JPEG_QUALITY="${JPEG_QUALITY:-72}"
WEBP_QUALITY="${WEBP_QUALITY:-80}"

mkdir -p "$OUT_DIR"

shopt -s nullglob
inputs=("$IMAGES_DIR"/*)
shopt -u nullglob

is_outdated() {
  local src="$1"
  local out="$2"
  [[ ! -f "$out" || "$out" -ot "$src" ]]
}

for src in "${inputs[@]}"; do
  [[ -f "$src" ]] || continue

  filename="$(basename "$src")"
  [[ "$filename" == .* ]] && continue
  [[ "$filename" == "gen" ]] && continue

  ext="${filename##*.}"
  lower_ext="$(printf "%s" "$ext" | tr '[:upper:]' '[:lower:]')"
  base="${filename%.*}"

  if [[ "$lower_ext" == "svg" ]]; then
    out_svg="$OUT_DIR/${base}.svg"
    if is_outdated "$src" "$out_svg"; then
      svgo --multipass --quiet --input "$src" --output "$out_svg" || true
    fi
    continue
  fi

  for w in "${WIDTHS[@]}"; do
    [[ "$w" =~ ^[0-9]+$ ]] || continue

    out_ext="$lower_ext"
    if [[ "$lower_ext" == "heic" || "$lower_ext" == "heif" ]]; then
      out_ext="jpg"
    fi

    fallback_out="$OUT_DIR/${base}-w${w}.${out_ext}"
    webp_out="$OUT_DIR/${base}-w${w}.webp"

    needs_fallback=false
    needs_webp=false
    if is_outdated "$src" "$fallback_out"; then needs_fallback=true; fi
    if [[ "$out_ext" != "webp" ]] && is_outdated "$src" "$webp_out"; then needs_webp=true; fi

    if [[ "$needs_fallback" == false && "$needs_webp" == false ]]; then
      continue
    fi

    tmp="$OUT_DIR/.tmp-${base}-w${w}.v"
    vips thumbnail "$src" "$tmp" "$w" --size down --fail-on none >/dev/null 2>&1 || {
      rm -f "$tmp" >/dev/null 2>&1 || true
      continue
    }

    if [[ "$needs_fallback" == true ]]; then
      case "$out_ext" in
        jpg|jpeg)
          vips jpegsave "$tmp" "$fallback_out" --Q="$JPEG_QUALITY" --strip --interlace --optimize-coding >/dev/null 2>&1 || true
          ;;
        png)
          vips pngsave "$tmp" "$fallback_out" --compression=9 --strip >/dev/null 2>&1 || true
          ;;
        webp)
          vips webpsave "$tmp" "$fallback_out" --Q="$WEBP_QUALITY" --strip --smart-subsample --effort=4 >/dev/null 2>&1 || true
          ;;
      esac
    fi

    if [[ "$needs_webp" == true ]]; then
      vips webpsave "$tmp" "$webp_out" --Q="$WEBP_QUALITY" --strip --smart-subsample --effort=4 >/dev/null 2>&1 || true
    fi

    rm -f "$tmp" >/dev/null 2>&1 || true
  done
done
