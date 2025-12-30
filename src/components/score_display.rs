//! Score Display Component
//!
//! Shows the player's current score, streak, and accuracy.

use crate::game::ScoreState;
use yew::prelude::*;

/// Props for the ScoreDisplay component
#[derive(Properties, PartialEq)]
pub struct ScoreDisplayProps {
    /// Current score state
    pub score: ScoreState,
}

/// The score display component
#[function_component(ScoreDisplay)]
pub fn score_display(props: &ScoreDisplayProps) -> Html {
    let score = &props.score;
    let total = score.correct + score.incorrect;

    html! {
        <div class="score-display">
            <div class="score-item">
                <span class="score-label">{ "Score" }</span>
                <span class="score-value correct-score">
                    { format!("{}/{}", score.correct, total) }
                </span>
            </div>

            <div class="score-item">
                <span class="score-label">{ "Accuracy" }</span>
                <span class="score-value">
                    { format!("{:.0}%", score.accuracy()) }
                </span>
            </div>

            <div class="score-item">
                <span class="score-label">{ "Streak" }</span>
                <span class="score-value streak">
                    { score.streak }
                    { if score.streak > 0 { "🔥" } else { "" } }
                </span>
            </div>

            { if score.best_streak > 0 {
                html! {
                    <div class="score-item best-streak">
                        <span class="score-label">{ "Best" }</span>
                        <span class="score-value">{ score.best_streak }</span>
                    </div>
                }
            } else {
                Html::default()
            }}
        </div>
    }
}
