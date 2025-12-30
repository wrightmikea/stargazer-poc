//! Main Application Component
//!
//! The root component that assembles all UI pieces and manages global state.

use crate::components::{Controls, QuizDropdown, ScoreDisplay, StarMap, SummaryPopup};
use crate::data::{generate_placeholder_catalog, TileSystem, ZoomLevel};
use crate::game::{game_reducer, GameAction, GameState, QuizConfig, QuizGenerator};
use gloo::events::EventListener;
use rand::SeedableRng;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew::prelude::*;

/// The main application component
#[function_component(App)]
pub fn app() -> Html {
    // Initialize star catalog - try to load from JSON, fallback to placeholder
    let catalog = use_memo((), |_| {
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, use placeholder for now
            // TODO: Implement async loading from HTTP
            return generate_placeholder_catalog();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Try loading from JSON in development/testing
            if let Ok(stars) = load_stars_from_json() {
                let mut catalog = StarCatalog::new();
                for star in stars {
                    catalog.add_star(star);
                }
                catalog.rebuild_indices();
                return catalog;
            }
        }

        #[allow(unreachable_code)]
        {
            generate_placeholder_catalog()
        }
    });

    // Build tile system from catalog
    let tile_system = use_memo(catalog.clone(), |cat| {
        let stars: Vec<_> = cat.all_stars().cloned().collect();
        TileSystem::from_stars(&stars)
    });

    // Game state with reducer
    let state = use_reducer(GameState::default);

    // Create a clone of state for use in callbacks
    let state_clone = state.clone();

    // Create action dispatcher
    let dispatch = {
        let state = state_clone.clone();
        Callback::from(move |action: GameAction| {
            state.dispatch(action);
        })
    };

    // Handle star selection to start quiz
    let on_action = {
        let dispatch = dispatch.clone();
        let catalog = catalog.clone();
        let tile_system = tile_system.clone();
        let state_for_quiz = state_clone.clone();

        Callback::from(move |action: GameAction| {
            // Special handling for star selection
            if let GameAction::SelectStar(star_id) = &action {
                // If clicking a named star, start a quiz
                if let Some(star) = catalog.get(*star_id) {
                    if star.has_name() {
                        let mut rng = rand::rngs::SmallRng::from_entropy();
                        let config = QuizConfig::default();

                        // Calculate zoom level based on viewport zoom
                        let current_zoom = state_for_quiz.viewport.zoom;
                        let zoom_level = ZoomLevel((current_zoom.log2().floor() as u8).clamp(0, 5));

                        // Use tile-aware quiz generator
                        let generator =
                            QuizGenerator::with_tiles(&catalog, config, &tile_system, zoom_level);

                        if let Some(question) = generator.generate_for_star(star, &mut rng) {
                            dispatch.emit(GameAction::StartQuiz {
                                target_star_id: question.target_star,
                                correct_name: question.correct_answer,
                                choices: question.choices,
                            });
                        }
                    }
                }
            }

            dispatch.emit(action);
        })
    };

    // Build the quiz dropdown if active
    let quiz_panel = if let (Some(quiz), Some(pos)) =
        (state_clone.quiz.clone(), state_clone.ui.dropdown_position)
    {
        html! {
            <QuizDropdown
                quiz={quiz.clone()}
                position={pos}
                on_action={on_action.clone()}
            />
        }
    } else {
        Html::default()
    };

    // Build summary popup if active
    let summary_panel = if let (Some(_quiz), Some(_pos)) =
        (state_clone.quiz.clone(), state_clone.ui.dropdown_position)
    {
        html! {
            <SummaryPopup
                guesses={state_clone.guess_history.clone()}
                score={state_clone.score.clone()}
                on_action={on_action.clone()}
            />
        }
    } else {
        Html::default()
    };

    // ESC key listener to dismiss summary popup
    {
        let dispatch = dispatch.clone();
        let summary_shown = state_clone.ui.summary_shown;
        use_effect_with(summary_shown, move |_| {
            let listener = if summary_shown {
                let window = web_sys::window().expect("no window");
                Some(EventListener::new(&window, "keydown", move |event| {
                    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                    if event.key() == "Escape" {
                        dispatch.emit(GameAction::HideSummary);
                    }
                }))
            } else {
                None
            };
            move || drop(listener)
        });
    }

    html! {
        <div class="app-container">
            <a href="https://github.com/wrightmikea/stargazer-poc" class="github-fork-ribbon" target="_blank" rel="noopener noreferrer" title="Fork me on GitHub">
                <span>{ "Fork me on GitHub" }</span>
            </a>
            <header class="app-header">
                <div class="header-left">
                    <h1 class="app-title">{ "✦ Stargazer" }</h1>
                    <p class="app-subtitle">{ "Test your knowledge of night sky" }</p>
                </div>
                <ScoreDisplay score={state_clone.score.clone()} />
            </header>

            <main class="app-main">
                <div class="star-map-wrapper">
                    <div class="star-map-container">
                        <StarMap
                            catalog={catalog.clone()}
                            viewport={state_clone.viewport}
                            magnitude_limit={state_clone.magnitude_limit}
                            show_grid={state_clone.show_grid}
                            selected_star={state_clone.selected_star}
                            on_action={on_action.clone()}
                        />
                    </div>
                    { quiz_panel }
                </div>

                <aside class="sidebar">
                    <Controls
                        zoom={state_clone.viewport.zoom}
                        magnitude_limit={state_clone.magnitude_limit}
                        show_grid={state_clone.show_grid}
                        on_action={on_action.clone()}
                    />
                    { summary_panel }
                </aside>
            </main>

            <footer class="app-footer">
                <div class="footer-content">
                    <p>
                        <span class="copyright">{ "© 2025 Michael A. Wright" }</span>
                        <span class="separator">{ "•" }</span>
                        <span class="license">{ "MIT License" }</span>
                        <span class="separator">{ "•" }</span>
                        <span class="build-info">{ format!("Build: 2025-12-30T10:55-08:00 • SHA: 7e39ace") }</span>
                    </p>
                </div>
            </footer>
        </div>
    }
}

// Required for use_reducer
impl Reducible for GameState {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        game_reducer(self, action)
    }
}
