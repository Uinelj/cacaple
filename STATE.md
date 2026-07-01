# Pipiple — STATE

## What Is This?
A French word-chain puzzle game (clone of Poople.io) where you transform a daily start word into **PIPI** (and later CACA, VOMI), one letter at a time. Built in **Rust + WASM**, served statically.

## Current Phase: ✅ MVP Complete

## Architecture
- `crate/` — Rust library compiled to WASM via wasm-pack
  - Core game logic: word validation, BFS distances, daily puzzle
  - Word list + precomputed distances embedded at compile time via `include_str!`
- `web/` — Static HTML/CSS/JS frontend
  - Calls into WASM for game logic
  - AZERTY keyboard, grid, modals (help/stats/yesterday), share
- `data/` — Generated data files (words, distances, start words)
- `Makefile` — `make build`, `make test`, `make serve`

## Key Decisions
- **Target word:** PIPI (for now — "Pipiple")
- **Accents:** Normalized to ASCII (é→e, è→e, etc.) for simpler keyboard UX
- **Word list source:** Crawled from lalanguefrancaise.com (71 pages) + UD French GSD/Sequoia treebanks
- **Epoch:** 2026-07-01 08:00 UTC (`EPOCH_MS = 1_751_356_800_000`)
- **Game change hour:** 08:00 UTC (10:00 Paris summer time)
- **WASM bundle:** ~111KB including all word data

## Stats
- **2,945 reachable words** from PIPI (out of 3,135 total in dictionary)
- **Max distance:** 13 steps
- **1,500 curated start words** at distances 4–8
- **6 Rust tests** all passing

## Word List Status
- [x] Crawl lalanguefrancaise.com (71 pages of 4-letter words) → 3,445 words
- [x] Merge with UD French treebank words (GSD + Sequoia)
- [x] Normalize accents to ASCII → 3,135 unique words
- [x] BFS from PIPI → 2,945 reachable, max distance 13
- [x] Generate distances.csv and start_words.csv
- [ ] Manual cleanup (some obscure words may remain — low priority)

## Implementation Checklist
- [x] Rust crate skeleton (lib.rs with wasm-bindgen)
- [x] Word list as embedded data (`include_str!`)
- [x] BFS from PIPI → precomputed distances
- [x] `is_valid_move(word, prev)` — in dict + exactly 1 letter different
- [x] `get_distance(word)` → precomputed distance to PIPI
- [x] `get_start_word(day_number)` → daily start word
- [x] `get_adjacent_words(word)` → all valid 1-letter neighbors
- [x] `get_shortest_path(word)` → optimal path (for "yesterday" feature)
- [x] `validate_error(word, prev)` → French error messages
- [x] Web frontend: game grid, AZERTY keyboard, modals
- [x] Share / copy results (🟨⬜ grid format)
- [x] localStorage for state (guesses, games, streaks)
- [x] Stats modal (wins, avg extra, streak, histogram, countdown)
- [x] Yesterday modal (optimal path display)
- [x] Help modal (how to play)
- [x] Emoji rain 🚽 on win
- [ ] Deploy as static site
- [ ] PWA / offline support

## Data Files
- `data/words.txt` — 2,945 reachable normalized 4-letter French words
- `data/distances.csv` — `word,distance` for all reachable words
- `data/start_words.csv` — 1,500 curated daily start words with distances (4–8)

## How to Run
```bash
make test    # Run Rust unit tests
make build   # Build WASM into web/pkg/
make serve   # Build + serve on http://localhost:8080
```

## Future Work
- [ ] Add CACA and VOMI as additional targets (multi-target rotation)
- [ ] Manual word list curation / cleanup
- [ ] Word frequency data for smarter "best path" selection
- [ ] PWA manifest + service worker for offline play
- [ ] Custom domain + deployment
- [ ] Dark/light theme toggle
