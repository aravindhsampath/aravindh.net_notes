# Agent Instructions (aravindh.net public notes)

## Goal
Maintain a simple Hugo-based static site that acts like a public “Apple Notes”: fast WYSIWYG editing (Typora), instant local preview, full‑text search (Pagefind), and a manual publish step (rsync to a web server).

## Primary Workflow
1. Write/edit notes in Typora.
2. On save, rebuild locally (Hugo) and update search index (Pagefind).
3. Preview locally.
4. When satisfied, run `publish.sh` to rsync the built site to the server.

## Content Rules (Hugo)
- Notes live under `content/` (prefer `content/notes/` if/when created).
- Use Markdown files with Hugo front matter (TOML/YAML/JSON). Prefer consistent front matter across notes.
- Filenames should be URL-friendly (kebab-case) and stable.
- Keep generated output out of git:
  - `public/` and Hugo `resources/` are build artifacts (ignored).

## Typora Setup Expectations
- Edit the site’s Markdown files in place (inside this repo).
- Prefer relative paths for images and links so Hugo can render them correctly.
- If images are used, store them in a predictable place (e.g. `static/` or page bundles) and keep links relative.

## Images (Simple Notes Folder Workflow)
If you want to keep notes as flat files under `content/notes/` (no per-note folders), store images under `static/images/` and configure Typora to copy images there.

- Create folder: `static/images/`
- Typora → Preferences → Image:
  - When Insert Local Images: Copy image to custom folder
  - Custom folder: `../../static/images` (from `content/notes/`)
  - Use relative path if possible: enabled

This repo includes a Hugo image render hook that rewrites paths containing `static/` to the correct published URL under the site base path.

### Responsive image variants
To avoid shipping multi‑MB originals to every device, the build workflow generates responsive variants into `static/images/gen/` and the Hugo render hook emits `srcset`/`sizes` with a WebP `<source>` when available.

- Variants are generated via `./scripts/optimize-images.sh` (requires `vips` and `svgo`).
- `static/images/gen/` is generated and ignored by git; it is included in deploy output.

## Local Build + Search Index
- Hugo builds the site to `public/` (default).
- Pagefind indexes the built HTML from `public/` and writes its index into the built output (commonly `public/pagefind/`).
- The intended dev loop is:
  - run a local watcher (Hugo server and/or a small wrapper script),
  - then run Pagefind after a successful rebuild.

## Publishing
- Publishing is manual and explicit via `publish.sh`.
- `publish.sh` should:
  - build the site,
  - run Pagefind,
  - rsync `public/` to the web server destination.

## What to Optimize For
- Minimal friction when writing in Typora.
- Predictable URLs (stable note slugs).
- Fast search and fast local preview.
- Simple, auditable scripts over complex tooling.
