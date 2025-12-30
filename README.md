# Stargazer - Star Name Matching Game

A WebAssembly-based interactive game for learning star names, built with Rust and Yew framework.

[![Live Demo](https://img.shields.io/badge/demo-live-blue?style=for-the-badge&logo=github)](https://wrightmikea.github.io/stargazer-poc/)

<img src="https://raw.githubusercontent.com/wrightmikea/stargazer-poc/main/images/screenshot.png" alt="Stargazer Screenshot" width="800">

## Overview

Stargazer presents an interactive celestial map where users can test their knowledge of star names. The game displays a zoomable, pannable star map using SVG, and challenges users to identify stars from a multiple-choice dropdown menu.

## Features

### Interactive Gameplay
- Click on named stars to start a quiz
- Multiple-choice questions with 4-5 options
- Real-time feedback on correct/incorrect answers
- Score tracking with streak counter
- "Done" button to view session summary

### Responsive Layout
- **Left Panel (Star Map)**: Fills the available viewport space, displaying clickable stars
- **Right Panel (Controls)**: Fixed-width panel with zoom, magnitude, and display controls

The layout automatically adjusts to browser window resizing and properly handles page reloads with viewport-aware sizing.

## Architecture

This project uses a hierarchical tile-based system for efficient star rendering, similar to map tiles (OSM, Google Maps).

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Stargazer Application                         │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────────────────┐   │
│  │   App       │   │  Star Map    │   │   Quiz Panel            │   │
│  │  Component  │──▶│  Component   │──▶│   Component             │   │
│  └─────────────┘   └──────────────┘   └─────────────────────────┘   │
│         │                 │                       │                  │
│         ▼                 ▼                       ▼                  │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Game State Manager                        │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │    │
│  │  │ View     │  │ Quiz     │  │ Score    │  │ Settings │     │    │
│  │  │ State    │  │ State    │  │ Tracker  │  │ State    │     │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │    │
│  └─────────────────────────────────────────────────────────────┘    │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      Data Layer                              │    │
│  │  ┌──────────────┐  ┌────────────────┐  ┌─────────────────┐  │    │
│  │  │ Star Catalog │  │ Tile Manager   │  │ Name Database   │  │    │
│  │  │ (RA/Dec/Mag) │  │ (LOD System)   │  │ (~400 names)    │  │    │
│  │  └──────────────┘  └────────────────┘  └─────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### Design Rationale

#### Why Rust + Yew + WASM?
- **Performance**: WASM runs at near-native speed, critical for rendering thousands of stars
- **Type Safety**: Rust's type system catches errors at compile time
- **Memory Safety**: No garbage collection pauses during animations
- **Small Bundle**: Optimized WASM binaries are typically smaller than equivalent JS

#### Level of Detail (LOD) System
To handle 6000+ stars without overwhelming the browser:

1. **Magnitude-based Filtering**: Stars filtered by apparent magnitude
   - Bright stars (mag < 2.0): ~25 stars, always visible
   - Medium stars (mag < 4.5): ~500 stars
   - Faint stars (mag < 6.0): ~6000 stars

2. **Spatial Tiling**: Sky divided into tiles
   - Tiles loaded on-demand as user pans
   - Background loading of adjacent tiles

3. **Zoom-dependent Rendering**:
   - Low zoom: Fewer, larger star symbols
   - High zoom: More stars, smaller symbols

### Component Hierarchy

```
App
├── Header
│   ├── ScoreDisplay
│   └── SettingsButton
├── StarMapContainer
│   ├── StarMapSVG
│   │   ├── GridOverlay
│   │   ├── ConstellationLines
│   │   └── StarLayer
│   │       └── Star (clickable)
│   ├── ZoomControls
│   └── MagnitudeSlider
├── QuizDropdown (conditional)
│   ├── ChoiceList
│   └── FeedbackMessage
└── Footer
```

### State Management

Using Yew's `use_reducer` for predictable state updates:

```rust
pub enum GameAction {
    // View actions
    SetZoom(f64),
    Pan(f64, f64),
    SetMagnitudeLimit(f64),
    
    // Quiz actions
    SelectStar(StarId),
    SubmitAnswer(String),
    NextQuestion,
    
    // Settings
    ToggleGrid,
    ToggleConstellations,
}
```

### Non-Functional Requirements

| Requirement     | Target                | Implementation Strategy                      |
|-----------------|----------------------|---------------------------------------------|
| Availability    | 99.9%                | Static hosting, no server dependencies      |
| Performance     | <100ms interactions  | WASM, LOD system, incremental rendering     |
| Usability       | Responsive layout    | Two-panel flexbox layout, viewport-aware SVG |
| Maintainability | Modular components  | Clean separation of concerns                |
| Testability     | >80% coverage       | Unit tests, integration tests, WASM tests   |

### Data Model

```rust
/// A celestial coordinate in the equatorial system
pub struct CelestialCoord {
    /// Right Ascension in hours (0-24)
    pub ra: f64,
    /// Declination in degrees (-90 to +90)
    pub dec: f64,
}

/// A star in the catalog
pub struct Star {
    pub id: StarId,
    pub coord: CelestialCoord,
    pub magnitude: f64,
    pub name: Option<String>,
    pub bayer: Option<String>,  // e.g., "α Ori"
    pub constellation: Option<String>,
}
```

### Coordinate Transformations

The map uses Mercator projection (equirectangular for simplicity in PoC):

```
Screen X = (RA / 24) * width
Screen Y = ((90 - Dec) / 180) * height
```

## Building and Running

### Prerequisites
- Rust toolchain (1.70+)
- wasm-pack
- trunk (for development server)

### Development
```bash
# Install tools
cargo install trunk wasm-pack

# Run development server
trunk serve --open

# Run tests
cargo test
wasm-pack test --headless --chrome
```

### Production Build
```bash
# Build for docs directory
trunk build --public-url ./docs --dist docs

# Regular release build
trunk build --release
```

### Screenshot Updates
The screenshot is generated using Playwright MCP:
1. Run `trunk serve --open` in background
2. Navigate to http://127.0.0.1:8080/
3. Take screenshot and save to `./images/screenshot.png`
4. Copy to docs: `cp ./images/screenshot.png ./docs/images/screenshot.png`
5. Rebuild with updated screenshot in place

## Project Structure

```
stargazer-poc/
├── Cargo.toml
├── README.md
├── index.html
├── src/
│   ├── lib.rs           # Library entry point
│   ├── app.rs           # Main App component
│   ├── components/      # UI components
│   │   ├── mod.rs
│   │   ├── star_map.rs
│   │   ├── quiz_dropdown.rs
│   │   ├── score_display.rs
│   │   └── controls.rs
│   ├── data/            # Data structures and catalog
│   │   ├── mod.rs
│   │   ├── star.rs
│   │   ├── catalog.rs
│   │   └── generator.rs # Placeholder data generator
│   ├── game/            # Game logic
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── quiz.rs
│   └── utils/           # Utilities
│       ├── mod.rs
│       └── projection.rs
├── tests/               # Integration tests
└── static/              # Static assets
```

## Future Enhancements

1. **Real Star Data**: Import from HYG database or similar
2. **Constellation Rendering**: Draw constellation lines
3. **Spaced Repetition**: Implement SM-2 algorithm for learning
4. **Offline Support**: Service worker for PWA
5. **User Accounts**: Persist progress across devices

## License

MIT License - See LICENSE file for details
