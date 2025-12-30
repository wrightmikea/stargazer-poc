# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stargazer is a WebAssembly-based star name quiz game built with Rust and the Yew framework. Users interact with an SVG star map to identify stars from multiple-choice options.

## Build Commands

```bash
# Development server with hot reload (opens browser automatically)
trunk serve --open

# Production build (outputs to docs/ for GitHub Pages)
trunk build --release

# Run Rust tests
cargo test

# Run WASM tests in headless Chrome
wasm-pack test --headless --chrome

# Lint with Clippy (runs automatically as pre-build hook)
cargo clippy
```

## Architecture

### Module Structure

- **src/lib.rs** - WASM entry point, initializes logger and mounts Yew app
- **src/app.rs** - Root `App` component, manages global state via `use_reducer`
- **src/components/** - UI components (StarMap, QuizDropdown, Controls, ScoreDisplay, SummaryPopup)
- **src/data/** - Star catalog, coordinate types, tile system for LOD rendering
- **src/game/** - Game state reducer pattern (`GameState`, `GameAction`) and quiz generation
- **src/utils/** - Coordinate projection (RA/Dec to screen coordinates)

### State Management

The app uses Yew's `use_reducer` with a Redux-like pattern:
- `GameState` holds viewport, quiz state, score, and UI state
- `GameAction` enum defines all state transitions
- `game_reducer` in `src/game/state.rs` handles state updates

### Key Patterns

- Stars are rendered via SVG with magnitude-based filtering for performance
- Tile system (`TileSystem`) enables level-of-detail rendering for 6000+ stars
- Quiz generation uses nearby stars from the tile system for contextually relevant choices
- Viewport state tracks zoom, pan offset, and center coordinates

### Coordinate System

- Stars use equatorial coordinates: Right Ascension (0-24h), Declination (-90 to +90 deg)
- Mercator projection converts to screen coordinates in `utils/projection.rs`

## Development Notes

- Trunk configuration is in `Trunk.toml` (serves on port 8080, outputs to `docs/`)
- Static assets go in `static/` directory
- Entry HTML is `index.html` with Trunk directives for WASM bundling
- Release builds use `opt-level = "s"` and LTO for smaller WASM size
