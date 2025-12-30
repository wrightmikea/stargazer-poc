//! Main Application Component
//!
//! The root component that assembles all UI pieces and manages global state.

use crate::components::{Controls, QuizDropdown, ScoreDisplay, StarMap};
use crate::data::generate_placeholder_catalog;
use crate::game::{game_reducer, GameAction, GameState, QuizConfig, QuizGenerator};
use gloo::events::EventListener;
use rand::SeedableRng;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew::prelude::*;

/// The main application component
#[function_component(App)]
pub fn app() -> Html {
    // Initialize the star catalog (would be loaded async in production)
    let catalog = use_memo((), |_| generate_placeholder_catalog());

    // Game state with reducer
    let state = use_reducer(GameState::default);

    // Create action dispatcher
    let dispatch = {
        let state = state.clone();
        Callback::from(move |action: GameAction| {
            state.dispatch(action);
        })
    };

    // Handle star selection to start quiz
    let on_action = {
        let dispatch = dispatch.clone();
        let catalog = catalog.clone();
        let _state_snapshot = (*state).clone();

        Callback::from(move |action: GameAction| {
            // Special handling for star selection
            if let GameAction::SelectStar(star_id) = &action {
                // If clicking a named star, start a quiz
                if let Some(star) = catalog.get(*star_id) {
                    if star.has_name() {
                        let mut rng = rand::rngs::SmallRng::from_entropy();
                        let config = QuizConfig::default();
                        let generator = QuizGenerator::new(&catalog, config);

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

    // Keyboard listener for Escape key to close quiz
    {
        let dispatch = dispatch.clone();
        let has_quiz = state.quiz.is_some();
        use_effect_with(has_quiz, move |_| {
            let listener = if has_quiz {
                let window = web_sys::window().expect("no window");
                Some(EventListener::new(&window, "keydown", move |event| {
                    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                    if event.key() == "Escape" {
                        dispatch.emit(GameAction::CloseQuiz);
                    }
                }))
            } else {
                None
            };
            move || drop(listener)
        });
    }

    // Window resize listener to update viewport size
    {
        let dispatch = dispatch.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no window");

            let update_size = {
                let window = window.clone();
                let dispatch = dispatch.clone();
                Callback::from(move |_| {
                    let width = window.inner_width().unwrap().as_f64().unwrap_or(1200.0);
                    let height = window.inner_height().unwrap().as_f64().unwrap_or(600.0);

                    dispatch.emit(GameAction::SetViewportSize(width, height));
                })
            };

            update_size.emit(());

            let listener = EventListener::new(&window, "resize", move |_| {
                update_size.emit(());
            });

            move || drop(listener)
        });
    }

    // Build the quiz dropdown if active
    let quiz_panel = if let (Some(quiz), Some(pos)) = (&state.quiz, state.ui.dropdown_position) {
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

    html! {
        <div class="app-container">
            <header class="app-header">
                <h1 class="app-title">{ "✦ Stargazer" }</h1>
                <p class="app-subtitle">{ "Test your knowledge of the night sky" }</p>
                <ScoreDisplay score={state.score.clone()} />
            </header>

            <main class="app-main">
                <div class="star-map-wrapper">
                    <div class="star-map-container">
                        <StarMap
                            catalog={catalog.clone()}
                            viewport={state.viewport}
                            magnitude_limit={state.magnitude_limit}
                            show_grid={state.show_grid}
                            selected_star={state.selected_star}
                            on_action={on_action.clone()}
                        />
                    </div>
                    { quiz_panel }
                </div>

                <aside class="sidebar">
                    <Controls
                        zoom={state.viewport.zoom}
                        magnitude_limit={state.magnitude_limit}
                        show_grid={state.show_grid}
                        on_action={on_action.clone()}
                    />
                </aside>
            </main>

            <footer class="app-footer">
                <p>{ "Stargazer PoC • Built with Rust + Yew + WebAssembly" }</p>
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
