//! Quiz Dropdown Component
//!
//! Displays the multiple-choice quiz interface when a star is selected.

use crate::game::{GameAction, QuizState};
use yew::prelude::*;

/// Props for the QuizDropdown component
#[derive(Properties, PartialEq)]
pub struct QuizDropdownProps {
    /// Current quiz state
    pub quiz: QuizState,

    /// Position to display the dropdown (x, y)
    pub position: (f64, f64),

    /// Callback for dispatching game actions
    pub on_action: Callback<GameAction>,
}

// Approximate dropdown dimensions for positioning
const DROPDOWN_WIDTH: f64 = 220.0;
const DROPDOWN_HEIGHT: f64 = 320.0;
const MARGIN: f64 = 15.0;

// Star map viewport dimensions (SVG coordinate space)
const MAP_WIDTH: f64 = 1200.0;
const MAP_HEIGHT: f64 = 600.0;

/// The quiz dropdown component
#[function_component(QuizDropdown)]
pub fn quiz_dropdown(props: &QuizDropdownProps) -> Html {
    let quiz = &props.quiz;
    let (x, y) = props.position;

    // Position coordinates are in SVG space (0-1200, 0-600)
    // Adjust X position to keep dropdown on screen
    let adjusted_x = if x + DROPDOWN_WIDTH + MARGIN > MAP_WIDTH {
        // Would overflow right - position to the left of the star
        (x - DROPDOWN_WIDTH - MARGIN).max(MARGIN)
    } else {
        x + MARGIN
    };

    // Adjust Y position to keep dropdown on screen
    // If star is in lower half, show dropdown above the star
    let adjusted_y = if y > MAP_HEIGHT / 2.0 {
        // Star is in lower half - position dropdown above
        (y - DROPDOWN_HEIGHT - MARGIN).max(MARGIN)
    } else {
        // Star is in upper half - position dropdown below
        y + MARGIN
    };

    let on_close = {
        let on_action = props.on_action.clone();
        Callback::from(move |_| {
            on_action.emit(GameAction::CloseQuiz);
        })
    };

    let choice_elements: Html = quiz
        .choices
        .iter()
        .enumerate()
        .map(|(i, choice)| {
            let is_selected = quiz.selected_answer.as_ref() == Some(choice);
            let is_correct = quiz.answered && choice == &quiz.correct_name;
            let is_wrong = quiz.answered && is_selected && quiz.was_correct == Some(false);

            let choice_class = classes!(
                "quiz-choice",
                is_selected.then_some("selected"),
                is_correct.then_some("correct"),
                is_wrong.then_some("wrong"),
            );

            let choice_clone = choice.clone();
            let on_action = props.on_action.clone();
            let answered = quiz.answered;

            let on_click = Callback::from(move |_| {
                if !answered {
                    on_action.emit(GameAction::SelectAndSubmitAnswer(choice_clone.clone()));
                }
            });

            html! {
                <div
                    key={i}
                    class={choice_class}
                    onclick={on_click}
                >
                    <span class="choice-number">{ i + 1 }</span>
                    <span class="choice-text">{ choice }</span>
                </div>
            }
        })
        .collect();

    // Result message (shown after clicking a choice)
    let action_area = if quiz.answered {
        let was_correct = quiz.was_correct.unwrap_or(false);
        let message = if was_correct { "Correct!" } else { "Incorrect" };
        let message_class = if was_correct {
            "result correct"
        } else {
            "result wrong"
        };

        html! {
            <div class="quiz-result">
                <div class={message_class}>{ message }</div>
                { if !was_correct {
                    html! {
                        <div class="correct-answer">
                            { format!("The answer was: {}", quiz.correct_name) }
                        </div>
                    }
                } else {
                    Html::default()
                }}
            </div>
        }
    } else {
        Html::default()
    };

    html! {
        <div
            class="quiz-dropdown"
            style={format!(
                "position: absolute; left: {}px; top: {}px;",
                adjusted_x,
                adjusted_y
            )}
        >
            <div class="quiz-header">
                <span class="quiz-title">{ "What star is this?" }</span>
                <button class="close-button" onclick={on_close}>{ "×" }</button>
            </div>
            <div class="quiz-choices">
                { choice_elements }
            </div>
            <div class="quiz-actions">
                { action_area }
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::StarId;

    #[test]
    fn test_quiz_state_creation() {
        let quiz = QuizState {
            target_star_id: StarId(1),
            correct_name: "Sirius".into(),
            choices: vec![
                "Sirius".into(),
                "Vega".into(),
                "Arcturus".into(),
                "Betelgeuse".into(),
                "none of above".into(),
            ],
            selected_answer: None,
            answered: false,
            was_correct: None,
        };

        assert_eq!(quiz.choices.len(), 5);
        assert!(!quiz.answered);
    }
}
