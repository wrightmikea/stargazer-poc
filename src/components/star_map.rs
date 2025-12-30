//! Star Map SVG Component
//!
//! Renders the interactive star map using SVG, handling
//! pan, zoom, and star selection interactions.

use crate::data::{Star, StarCatalog, StarId};
use crate::game::GameAction;
use crate::utils::{Projection, Viewport};
use std::rc::Rc;
use web_sys::{MouseEvent, WheelEvent};
use yew::prelude::*;

/// Props for the StarMap component
#[derive(Properties, PartialEq)]
pub struct StarMapProps {
    /// The star catalog to render
    pub catalog: Rc<StarCatalog>,

    /// Current viewport configuration
    pub viewport: Viewport,

    /// Current magnitude limit
    pub magnitude_limit: f64,

    /// Whether to show grid lines
    pub show_grid: bool,

    /// Currently selected star
    pub selected_star: Option<StarId>,

    /// Callback for dispatching game actions
    pub on_action: Callback<GameAction>,
}

/// The star map component
#[function_component(StarMap)]
pub fn star_map(props: &StarMapProps) -> Html {
    let is_dragging = use_state(|| false);
    let last_pos = use_state(|| (0.0, 0.0));

    // Get visible stars
    let (ra_min, ra_max) = props.viewport.ra_range();
    let (dec_min, dec_max) = props.viewport.dec_range();
    let visible_stars = props.catalog.stars_in_range(
        ra_min,
        ra_max,
        dec_min,
        dec_max,
        props.magnitude_limit,
    );

    // Event handlers
    let on_mouse_down = {
        let is_dragging = is_dragging.clone();
        let last_pos = last_pos.clone();
        Callback::from(move |e: MouseEvent| {
            is_dragging.set(true);
            last_pos.set((e.client_x() as f64, e.client_y() as f64));
        })
    };

    let on_mouse_move = {
        let is_dragging = is_dragging.clone();
        let last_pos = last_pos.clone();
        let on_action = props.on_action.clone();
        Callback::from(move |e: MouseEvent| {
            if *is_dragging {
                let (last_x, last_y) = *last_pos;
                let dx = e.client_x() as f64 - last_x;
                let dy = e.client_y() as f64 - last_y;
                last_pos.set((e.client_x() as f64, e.client_y() as f64));
                on_action.emit(GameAction::Pan(dx, dy));
            }
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |_: MouseEvent| {
            is_dragging.set(false);
        })
    };

    let on_mouse_leave = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |_: MouseEvent| {
            is_dragging.set(false);
        })
    };

    let on_wheel = {
        let on_action = props.on_action.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            let factor = if e.delta_y() < 0.0 { 1.2 } else { 0.8 };
            on_action.emit(GameAction::ZoomBy(factor));
        })
    };

    // Background click to dismiss quiz dialog
    let on_background_click = {
        let on_action = props.on_action.clone();
        Callback::from(move |_: MouseEvent| {
            on_action.emit(GameAction::CloseQuiz);
        })
    };

    // Generate SVG elements
    let grid_lines = if props.show_grid {
        render_grid(&props.viewport)
    } else {
        Html::default()
    };

    let star_elements: Html = visible_stars
        .iter()
        .map(|star| {
            render_star(
                star,
                &props.viewport,
                props.selected_star == Some(star.id),
                props.on_action.clone(),
            )
        })
        .collect();

    html! {
        <svg
            class="star-map"
            viewBox={format!("0 0 {} {}", props.viewport.width, props.viewport.height)}
            preserveAspectRatio="xMidYMid slice"
            onmousedown={on_mouse_down}
            onmousemove={on_mouse_move}
            onmouseup={on_mouse_up}
            onmouseleave={on_mouse_leave}
            onwheel={on_wheel}
        >
            // Background (click to dismiss quiz)
            <rect
                x="0"
                y="0"
                width={props.viewport.width.to_string()}
                height={props.viewport.height.to_string()}
                fill="#0a0a14"
                onclick={on_background_click}
            />

            // Grid
            {grid_lines}

            // Stars
            {star_elements}
        </svg>
    }
}

/// Render grid lines
fn render_grid(viewport: &Viewport) -> Html {
    let mut lines = Vec::new();

    // RA lines (every hour at zoom 1, more at higher zooms)
    let ra_step = (2.0 / viewport.zoom).max(0.5);
    let mut ra = 0.0;
    while ra < 24.0 {
        let _coord = crate::data::CelestialCoord::new(ra, 0.0);
        let screen_top = viewport.celestial_to_screen(&crate::data::CelestialCoord::new(ra, 90.0));
        let screen_bot = viewport.celestial_to_screen(&crate::data::CelestialCoord::new(ra, -90.0));

        if screen_top.x >= 0.0 && screen_top.x <= viewport.width {
            lines.push(html! {
                <line
                    key={format!("ra-{}", ra)}
                    x1={screen_top.x.to_string()}
                    y1={screen_top.y.to_string()}
                    x2={screen_bot.x.to_string()}
                    y2={screen_bot.y.to_string()}
                    stroke="#1a3a5a"
                    stroke-width="1"
                    stroke-opacity="0.5"
                />
            });
        }
        ra += ra_step;
    }

    // Dec lines (every 10 degrees at zoom 1, more at higher zooms)
    let dec_step = (30.0 / viewport.zoom).max(5.0);
    let mut dec = -80.0;
    while dec <= 80.0 {
        let screen_left = viewport.celestial_to_screen(&crate::data::CelestialCoord::new(0.0, dec));
        let screen_right = viewport.celestial_to_screen(&crate::data::CelestialCoord::new(24.0, dec));

        // Celestial equator gets special treatment
        let stroke_color = if (dec.abs()) < 0.1 { "#7a2a5a" } else { "#1a3a5a" };
        let stroke_width = if (dec.abs()) < 0.1 { "2" } else { "1" };

        lines.push(html! {
            <line
                key={format!("dec-{}", dec)}
                x1="0"
                y1={screen_left.y.to_string()}
                x2={viewport.width.to_string()}
                y2={screen_right.y.to_string()}
                stroke={stroke_color}
                stroke-width={stroke_width}
                stroke-opacity="0.5"
            />
        });
        dec += dec_step;
    }

    html! { <>{ for lines }</> }
}

/// Render a single star
fn render_star(
    star: &Star,
    viewport: &Viewport,
    is_selected: bool,
    on_action: Callback<GameAction>,
) -> Html {
    let screen = viewport.celestial_to_screen(&star.coord);
    let base_radius = 3.0 / viewport.zoom.sqrt();
    let radius = star.render_radius(base_radius);

    // Color based on whether star is named
    let fill_color = if star.has_name() {
        "#fffaf0" // Warmer white for named stars
    } else {
        "#c0c8d0" // Cooler for unnamed
    };

    let star_id = star.id;
    let has_name = star.has_name();
    let screen_x = screen.x;
    let screen_y = screen.y;

    let on_click = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if has_name {
            on_action.emit(GameAction::SelectStar(star_id));
            on_action.emit(GameAction::SetDropdownPosition(screen_x, screen_y));
        }
    });

    // Selection ring for selected star
    let selection_ring = if is_selected {
        html! {
            <circle
                cx={screen.x.to_string()}
                cy={screen.y.to_string()}
                r={(radius * 3.0).to_string()}
                fill="none"
                stroke="#ff4444"
                stroke-width="2"
            />
        }
    } else {
        Html::default()
    };

    html! {
        <g key={format!("star-{}", star.id.0)} class="star-group">
            {selection_ring}
            <circle
                cx={screen.x.to_string()}
                cy={screen.y.to_string()}
                r={radius.to_string()}
                fill={fill_color}
                class={if star.has_name() { "star named-star" } else { "star" }}
                onclick={on_click}
                style={if star.has_name() { "cursor: pointer;" } else { "" }}
            >
                { if star.has_name() {
                    html! {
                        <title>{ star.display_name() }</title>
                    }
                } else {
                    Html::default()
                }}
            </circle>
        </g>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full component tests require wasm-bindgen-test
    // These are basic unit tests for helper functions

    #[test]
    fn test_render_functions_compile() {
        // Just ensure the render functions are valid Rust
        let viewport = Viewport::default();
        let _grid = render_grid(&viewport);
    }
}
