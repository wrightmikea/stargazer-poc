//! Summary Popup Component
//!
//! Displays a summary of guesses when user clicks "Done".

use crate::game::{GameAction, GameState, GuessSummary, ScoreState};
use yew::prelude::*;

/// Props for SummaryPopup component
#[derive(Properties, PartialEq)]
pub struct SummaryPopupProps {
    /// Guess history
    pub guesses: Vec<GuessSummary>,

    /// Score state
    pub score: ScoreState,

    /// Callback for dispatching game actions
    pub on_action: Callback<GameAction>,
}

/// The summary popup component
#[function_component(SummaryPopup)]
pub fn summary_popup(props: &SummaryPopupProps) -> Html {
    let total = props.guesses.len();
    let correct = props.score.correct;
    let incorrect = props.score.incorrect;
    let accuracy = props.score.accuracy();
    let streak = props.score.streak;
    let best_streak = props.score.best_streak;

    let guess_rows: Html = if total == 0 {
        html! {
            <div class="summary-empty">
                <p>{ "No guesses yet! Start quizzing some stars." }</p>
            </div>
        }
    } else {
        html! {
            <div class="summary-list">
                { props.guesses.iter().enumerate().rev().map(|(i, guess)| {
                    let result_icon = if guess.was_correct { "✓" } else { "✗" };
                    let result_class = if guess.was_correct { "correct" } else { "incorrect" };

                    html! {
                        <div key={i} class={classes!("summary-row", result_class)}>
                            <span class="summary-icon">{ result_icon }</span>
                            <span class="summary-star">{ &guess.star_name }</span>
                            <span class="summary-answer">{ &guess.user_answer }</span>
                        </div>
                    }
                }).collect::<Html>() }
            </div>
        }
    };

    html! {
        <div class="summary-overlay">
            <div class="summary-popup">
                <div class="summary-header">
                    <h2>{ "Session Summary" }</h2>
                    <button onclick={props.on_action.reform(|_| GameAction::HideSummary)} class="close-button">
                        { "×" }
                    </button>
                </div>

                <div class="summary-stats">
                    <div class="stat-item">
                        <span class="stat-label">{ "Total Questions:" }</span>
                        <span class="stat-value">{ total }</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{ "Correct:" }</span>
                        <span class="stat-value correct">{ correct }</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{ "Incorrect:" }</span>
                        <span class="stat-value incorrect">{ incorrect }</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{ "Accuracy:" }</span>
                        <span class="stat-value">{ format!("{:.1}%", accuracy) }</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{ "Streak:" }</span>
                        <span class="stat-value">{ streak }</span>
                    </div>
                    <div class="stat-item">
                        <span class="stat-label">{ "Best Streak:" }</span>
                        <span class="stat-value">{ best_streak }</span>
                    </div>
                </div>

                <div class="summary-guesses">
                    <h3>{ "Guess History" }</h3>
                    { guess_rows }
                </div>

                <div class="summary-actions">
                    <button class="reset-button" onclick={props.on_action.reform(|_| GameAction::ResetScore)}>
                        { "Reset & Start Over" }
                    </button>
                    <button class="close-btn" onclick={props.on_action.reform(|_| GameAction::HideSummary)}>
                        { "Close" }
                    </button>
                </div>
            </div>
        </div>
    }
}
