# Agent Instructions (aravindh.net public notes)

## Goal
Maintain a simple Hugo-based static site that acts like a public “Apple Notes”: fast WYSIWYG editing (Typora), instant local preview, full‑text search (Pagefind), and a manual publish step (rsync to a web server).

## Primary Workflow
1. Write/edit notes in Typora.
2. Run `./scripts/dev.sh` to start the local server and search indexer.
3. Preview locally (default: `http://localhost:1313`).
4. When satisfied, run `./deploy.sh` to rsync the built site to the server.

## Project Structure & Scripts
- `scripts/dev.sh`:
  - Optimizes images.
  - Builds site and indexes search (Pagefind).
  - Starts Hugo watcher and a Python HTTP server (serving `public/`).
  - Uses `fswatch` (if available) to update Pagefind index on HTML changes.
- `scripts/build.sh`:
  - Optimizes images.
  - Builds production site (minified).
  - Generates Pagefind index.
- `deploy.sh`:
  - Runs `scripts/build.sh`.
  - Rsyncs `public/` to the web server.

## Content Rules (Hugo)
- Notes live under `content/` (prefer `content/notes/`).
- Use Markdown files with Hugo front matter (TOML).
- Filenames should be URL-friendly (kebab-case) and stable.
- Keep generated output out of git:
  - `public/` and Hugo `resources/` are build artifacts (ignored).

## Typora Setup Expectations
- Edit the site’s Markdown files in place (inside this repo).
- Prefer relative paths for images and links so Hugo can render them correctly.
- If images are used, store them in a predictable place and keep links relative.

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
- Pagefind indexes the built HTML from `public/` and writes its index into the built output (`public/pagefind/`).
- The `scripts/dev.sh` script automates this process (Hugo server + Pagefind).

## Publishing
- Publishing is manual and explicit via `./deploy.sh`.
- `deploy.sh` will:
  - Build the site (via `scripts/build.sh`).
  - Rsync `public/` to the web server destination defined in the script.

## What to Optimize For
- Minimal friction when writing in Typora.
- Predictable URLs (stable note slugs).
- Fast search and fast local preview.
- Simple, auditable scripts over complex tooling.
