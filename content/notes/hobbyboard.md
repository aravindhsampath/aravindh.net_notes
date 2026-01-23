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
I’m a hobbyist woodworker. I got tired of collecting inspiration in Apple Notes, Photos.app, or random folders, only to scroll hopelessly later trying to find "that one image of those nice tapered table legs" I knew I stashed *somewhere*.

I realized I wasn't just hoarding files; I was losing them.

Hobbyboard was built to solve that. It bridges the gap between collecting inspiration and actually finding it again when you need it.

Existing solutions didn't fit:
- **Pinterest:** Great for discovery, bad for organizing *my* specific files (and I prefer not to be served ads for spatulas when I'm looking for joinery).
- **Cloud SaaS:** I didn't want to pay a subscription to access my own files.

## What does it do?

- **🧠 It Has Eyes:** Uses Vision AI (Ollama, OpenAI, or Gemini) to look at your images so you don't have to tag them. It turns `IMG_4421.JPG` into "Brutalist concrete architecture, overcast sky, 1970s style."
- **🔍 Hybrid Search:** Combines **Vector Search** (vibes/concepts) with **Full Text Search** (keywords). Search for "moody lighting" or "Helvetica" and get results that make sense.
- **📝 Notes (New!):** Add your own snarky comments to images. Hobbyboard indexes those too. Auto-saving, because we know you'll forget.
- **⚡ Rust Backend:** Fast, memory-safe, and morally superior.
- **🎨 Responsive Masonry Grid:** Because simple grids are for spreadsheets. Customizable density.
- **📂 Zero-Lock-in:** Your data lives in SQLite and local folders. If you hate Hobbyboard later (you might), you can take your images, tags, notes, and boards elsewhere.

## Who is this for?
- **Woodworkers / Makers:** "I saw a joinery technique once..."
- **Designers:** "I need that specific shade of 'depressed corporate blue'..."
- **Data Hoarders:** You know who you are.

## Quick Start
*(Choose your fighter)*

### "I'm a Docker Everything Guy"
1.  Setup Qdrant
2.  Run the container
3.  Profit?

### "I'd Rather Run a Binary"
1.  Setup Qdrant
2.  Download the binary
3.  `./hobbyboard init`
4.  `./hobbyboard serve`

### "I Ain't Running a Binary from GitHub!"
*Grab your reading glasses (not the drinking kind).*
1.  Audit the source code (I dare you).
2.  Start Qdrant.
3.  `cargo build --release`
4.  `./target/release/hobbyboard init`
5.  `./target/release/hobbyboard serve`

## "Hey, this looks like it was vibe-coded in a few hours!"
First of all, rude. Second, I am so glad you read this far.
It took **days**. Yes, Hobbyboard is built with Gemini-3 and a lot of patience. Mostly patience. I take pride in knowing exactly what to ask the LLMs to do, and then verifying that they actually did it correctly (they usually don't).

## The Little Things (I Hope You Notice)
1.  **The Dark Mode:** It's not just black (#000000). We have standards.
2.  **The Light Mode:** It's not just white (#FFFFFF). My eyes aren't bleeding.
3.  **The Logo:** It symbolizes a masonry grid. It morphs on hover. Does it serve a function? No. Is it cool? Yes.
4.  **Translucency:** That top bar blur? Chef's kiss.
5.  **Modal UX:** Click outside the image to close. It just works.
6.  **Drag & Drop:** Upload media like it's 2026.
7.  **Native Share Sheet:** Share a board/image using your OS's native tools.
8.  **Google Takeout-style Export:** Download a ZIP of your original assets and metadata.
9.  **Logs:** Something broke? Check the web UI logs.
10. **The Search Bar:** It expands. It has a gradient. It was "Variant Z" and it was the right choice.

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
