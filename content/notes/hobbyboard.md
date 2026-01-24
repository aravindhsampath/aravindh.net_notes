+++
title = "hobbyboard"
date = 2026-01-23T23:18:35+01:00
draft = true
tags = []

+++

# Hobbyboard
> Your local media library, organized by AI, searchable by concept.
> *Think of it as a private, self-hosted Pinterest that actually respects your data.*

***Screenshots and "Right-Click > Save As" are a Lifestyle.***

## Why Hobbyboard?
I’m a hobbyist woodworker. I got tired of collecting inspiration in Apple Notes, Photos.app, or random folders, only to scroll hopelessly later trying to find "that one image of those nice tapered table legs" I knew I stashed *somewhere* a while ago.

Hobbyboard was built to make sense of all the collected inspiration without much effort and actually finding it again when you need it.

## What does it do?

 **Use AI**: Uses Vision models (Ollama, OpenAI, or Gemini) to caption, tag, and OCR your images. It turns `IMG_4421.JPG` into "Brutalist concrete architecture, overcast sky, 1970s style."

 **Hybrid Search:** Combines **Vector Search** (vibes/concepts) with **Full Text Search** (keywords, OCR, manual tags and notes). Search for "moody lighting" or "Helvetica" and get results that actually match.

 **Manual input:** Accepts that AI is not always great. Add your own tags, snarky comments to images. Hobbyboard indexes those too. Auto-saving.

**Responsive Masonry Grid:** Because it is nice. Customise how dense you want.

**KISS - Keep it standard and simple** Your data lives in SQLite and local folders. If you hate HobbyBoard later, you can take your images, tags, notes, and boards elsewhere.

## Who is this for?
- **Woodworkers / Makers:** "I saw a joinery technique once..."
- **Designers:** "I need that specific shade of 'depressed corporate blue'..."
- **Data Hoarders:** You know who you are.

## What is it made of?

- **Backend:** Rust (Axum, Tokio).
- **Frontend:** HTML, CSS and vanilla JS
- **Database:** SQLite (Metadata) + Qdrant (Vectors).
- **AI:** `fastembed-rs` for local embedding, plus vision capable model - local or accessible via API.

## What does it look like?

![screenshot_1](../../static/images/screenshot_1.png)

![screenshot_2](../../static/images/screenshot_2.png)







## Quick Start

*(Choose your fighter)*

### "I'm a Docker Everything Guy"
1. Setup Qdrant 

   ```bash
   docker compose up qdrant -d
   ```

2. **Configure:** Run the interactive setup wizard to configure your media path, AI provider, and download models.

   ```bash
   docker compose run --rm app setup
   ```

3. **Initialize:** This downloads the local embedding model and sets up the database.

   ```bash
   docker compose run --rm app init
   ```

4. **Build:** Place your media in `raw_images/`, then process them:

   ```bash
   docker compose run --rm app build
   ```

5. **Launch Hobbyboard:**

   ```bash
   docker compose up app -d
   ```

6. Open `http://localhost:9625` in your browser.

### "I'd Rather Run a Binary"
1. Setup Qdrant 

   ```bash
   docker compose up qdrant -d
   ```
2. Download the binary - [Releases](https://github.com/aravindhsampath/hobbyboard/releases)
3. `./hobbyboard setup`
4. `./hobbyboard init`
5. `./hobbyboard serve`

### "I Ain't Running a Binary from GitHub!"
*Grab your glasses (not the drinking kind).*

1. Audit the source code (I thank you).

2. Setup Qdrant 

   ```bash
   docker compose up qdrant -d
   ```

3. `cargo build --release`

4. `./target/release/hobbyboard setup`

5. `./target/release/hobbyboard init`

6. `./target/release/hobbyboard serve`

## "Hey, this looks like it was vibe-coded in a few hours!"
First of all, rude. Second, I am so glad you read this far.
It took **days**. Yes, Hobbyboard is built with Gemini-3 and a lot of patience. Mostly patience. I take pride in knowing exactly what to ask the LLMs to do, and then verifying that they actually did it correctly (they usually don't).

## The Little Things (I Hope You Notice)
1. **Dark Mode and Light mode:** It's not just black (#000000) and white(#FFFFFF). It is much more. 

   <details>
     <summary>Click me for the details</summary>

The visual foundation uses a hybrid approach: a solid base color, a noise texture overlay (for grain), and a radial light effect (exclusive to dark mode).

  1. Global Grain Texture (All Modes)
  A subtle noise filter is applied globally via a fixed pseudo-element to create a "film grain" or paper-like texture.
   * Implementation: body::before (fixed overlay).
   * Effect: SVG feTurbulence (fractal noise).
   * Opacity: 0.03 (3%).
   * CSS Snippet:

   1     background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3'
     stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");

  2. Light Mode ("Warm Stone")
  Designed to reduce glare while maintaining a clean, editorial look.
   * Base Color (`--bg`): #EAE9E6 (Warm, grayish stone).
   * Background Image: none (The radial gradient is explicitly removed).
   * Foreground (`--fg`): #111111 (Charcoal).
   * Muted Text (`--muted`): #4A4A45 (Warm dark grey).
   * Borders: #DCDbd8.

  3. Dark Mode ("Deep Obsidian Ink")
  Features a spotlight effect to add depth to the dark surface.
   * Base Color (`--bg`): #0D0D0C (Nearly black).
   * Radial Depth Effect: A subtle top-down spotlight.
       * radial-gradient(circle at top, rgba(255, 255, 255, 0.03) 0%, transparent 70%)
   * Foreground (`--fg`): #e5e5e5 (Off-white).
   * Muted Text (`--muted`): #888888.
   * Borders: #262626.

   </details>

2. **The Logo:** It symbolizes a masonry grid. It morphs on hover. Does it serve a function? No. Is it cool? Yes.

3. **Translucency:** That top bar blur? It floats over the board like frosted glass. 

4. **Modal UX:** Click outside the image to close. It just works.

5. **Drag & Drop:** Upload media like it's 2026.

6. **Native Share Sheet:** Share a board/image using your OS's native tools.

7. **Google Takeout-style Export:** Download a ZIP of your original assets and metadata.

8. **Logs:** Something broke? Check the web UI logs.

9. **The Search Bar:** It expands. It has a gradient. It was "Variant Z" and it was the right choice.

## FAQ

**I have 50K images, will Hobbyboard handle it?**
What kind of machine do you have? Find out and tell me. (It probably will, it's Rust).

**Why not just use $SaaS?**
I am an engineer who would rather spend €300 on tools and a month of free time to build a desk for my kids than go to IKEA and buy a similar one for €30. It's about the *principle* (and the suffering).

**Can my AI send PRs?**
My AI agent will review your AI agent's PR. If my AI deems it worthy, I might manually click "Merge".

**How do I pay you?**
You don't. Hobbyboard solves *my* problem. I don't want to turn this into a job. If it solves your problem too, that's cool.

## About the Author
**Aravindh.net**
I work a full-time job. This is what happens when I have free time and too much coffee.
