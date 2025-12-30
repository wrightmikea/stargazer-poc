//! Stargazer - Star Name Matching Game
//!
//! A WebAssembly-based interactive game for learning star names,
//! built with Rust and the Yew framework.
//!
//! # Architecture
//!
//! The application is structured as follows:
//!
//! - **data**: Star catalog and celestial coordinate types
//! - **game**: Game state management and quiz logic
//! - **utils**: Coordinate projections and utilities
//! - **components**: Yew UI components
//! - **app**: Main application component
//!
//! # Usage
//!
//! Build and run with Trunk:
//! ```bash
//! trunk serve --open
//! ```

pub mod app;
pub mod components;
pub mod data;
pub mod game;
pub mod utils;

pub use app::App;

use wasm_bindgen::prelude::*;

/// Entry point for the WebAssembly module
///
/// This function is called when the WASM module is loaded.
/// It initializes logging and mounts the Yew application.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Stargazer starting...");

    // Mount the Yew application
    yew::Renderer::<App>::new().render();

    Ok(())
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
