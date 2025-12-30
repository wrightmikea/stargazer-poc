//! Control Panel Component
//!
//! Provides UI controls for zoom, magnitude filter, and display settings.

use crate::game::GameAction;
use yew::prelude::*;
use web_sys::HtmlInputElement;

/// Props for the Controls component
#[derive(Properties, PartialEq)]
pub struct ControlsProps {
    /// Current zoom level
    pub zoom: f64,

    /// Current magnitude limit
    pub magnitude_limit: f64,

    /// Whether grid is shown
    pub show_grid: bool,

    /// Callback for dispatching game actions
    pub on_action: Callback<GameAction>,
}

/// The controls panel component
#[function_component(Controls)]
pub fn controls(props: &ControlsProps) -> Html {
    // Zoom controls
    let on_zoom_in = {
        let on_action = props.on_action.clone();
        Callback::from(move |_| {
            on_action.emit(GameAction::ZoomBy(1.5));
        })
    };

    let on_zoom_out = {
        let on_action = props.on_action.clone();
        Callback::from(move |_| {
            on_action.emit(GameAction::ZoomBy(0.67));
        })
    };

    let on_reset = {
        let on_action = props.on_action.clone();
        Callback::from(move |_| {
            on_action.emit(GameAction::ResetView);
        })
    };

    // Magnitude slider
    let on_magnitude_change = {
        let on_action = props.on_action.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<f64>() {
                on_action.emit(GameAction::SetMagnitudeLimit(value));
            }
        })
    };

    // Grid toggle
    let on_grid_toggle = {
        let on_action = props.on_action.clone();
        Callback::from(move |_| {
            on_action.emit(GameAction::ToggleGrid);
        })
    };

    // Star count estimate based on magnitude
    let star_estimate = estimate_visible_stars(props.magnitude_limit);

    html! {
        <div class="controls-panel">
            // Zoom controls
            <div class="control-group">
                <label class="control-label">{ "Zoom" }</label>
                <div class="zoom-buttons">
                    <button class="control-btn" onclick={on_zoom_out} title="Zoom Out">
                        { "−" }
                    </button>
                    <span class="zoom-level">{ format!("{:.1}×", props.zoom) }</span>
                    <button class="control-btn" onclick={on_zoom_in} title="Zoom In">
                        { "+" }
                    </button>
                    <button class="control-btn reset" onclick={on_reset} title="Reset View">
                        { "⟲" }
                    </button>
                </div>
            </div>

            // Magnitude slider
            <div class="control-group">
                <label class="control-label">
                    { "Star Brightness" }
                    <span class="control-hint">
                        { format!(" (mag < {:.1})", props.magnitude_limit) }
                    </span>
                </label>
                <input
                    type="range"
                    class="magnitude-slider"
                    min="1.5"
                    max="6.5"
                    step="0.5"
                    value={props.magnitude_limit.to_string()}
                    oninput={on_magnitude_change}
                />
                <div class="slider-labels">
                    <span>{ "Bright" }</span>
                    <span class="star-count">{ format!("~{} stars", star_estimate) }</span>
                    <span>{ "Faint" }</span>
                </div>
            </div>

            // Display toggles
            <div class="control-group">
                <label class="control-label">{ "Display" }</label>
                <div class="toggle-buttons">
                    <button
                        class={classes!("toggle-btn", props.show_grid.then_some("active"))}
                        onclick={on_grid_toggle}
                    >
                        { "Grid" }
                    </button>
                </div>
            </div>

            // Help text
            <div class="control-help">
                <p>{ "🖱️ Drag to pan • Scroll to zoom" }</p>
                <p>{ "Click on a " }<span class="named-star-hint">{ "bright star" }</span>{ " to test your knowledge!" }</p>
            </div>
        </div>
    }
}

/// Estimate number of visible stars for a given magnitude limit
fn estimate_visible_stars(magnitude_limit: f64) -> u32 {
    // Rough approximation based on real star counts
    if magnitude_limit < 2.0 {
        20
    } else if magnitude_limit < 3.0 {
        100
    } else if magnitude_limit < 4.0 {
        300
    } else if magnitude_limit < 5.0 {
        900
    } else if magnitude_limit < 6.0 {
        2500
    } else {
        6000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_estimates() {
        assert!(estimate_visible_stars(2.0) < estimate_visible_stars(4.0));
        assert!(estimate_visible_stars(4.0) < estimate_visible_stars(6.0));
    }
}
