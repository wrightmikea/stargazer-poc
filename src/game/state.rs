//! Game state management
//!
//! Uses a reducer pattern for predictable state updates,
//! compatible with Yew's use_reducer hook.

use crate::data::{StarId};
use crate::utils::Viewport;
use std::rc::Rc;

/// The complete game state
#[derive(Debug, Clone)]
pub struct GameState {
    /// Current viewport configuration
    pub viewport: Viewport,

    /// Current magnitude limit for display
    pub magnitude_limit: f64,

    /// Whether to show grid lines
    pub show_grid: bool,

    /// Whether to show constellation lines
    pub show_constellations: bool,

    /// Current quiz state (if a quiz is active)
    pub quiz: Option<QuizState>,

    /// Score tracker
    pub score: ScoreState,

    /// Currently selected star (highlighted)
    pub selected_star: Option<StarId>,

    /// UI state
    pub ui: UiState,
}

/// State for an active quiz question
#[derive(Debug, Clone, PartialEq)]
pub struct QuizState {
    /// The star being quizzed
    pub target_star_id: StarId,

    /// The correct answer
    pub correct_name: String,

    /// All choices (including correct answer)
    pub choices: Vec<String>,

    /// User's selected answer (if any)
    pub selected_answer: Option<String>,

    /// Whether the answer has been submitted
    pub answered: bool,

    /// Whether the answer was correct
    pub was_correct: Option<bool>,
}

/// Score tracking
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoreState {
    /// Number of correct answers
    pub correct: u32,

    /// Number of incorrect answers
    pub incorrect: u32,

    /// Current streak of correct answers
    pub streak: u32,

    /// Best streak achieved
    pub best_streak: u32,
}

impl ScoreState {
    /// Calculate accuracy as a percentage
    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            0.0
        } else {
            (self.correct as f64 / total as f64) * 100.0
        }
    }

    /// Record a correct answer
    pub fn record_correct(&mut self) {
        self.correct += 1;
        self.streak += 1;
        if self.streak > self.best_streak {
            self.best_streak = self.streak;
        }
    }

    /// Record an incorrect answer
    pub fn record_incorrect(&mut self) {
        self.incorrect += 1;
        self.streak = 0;
    }
}

/// UI-specific state
#[derive(Debug, Clone, Default)]
pub struct UiState {
    /// Position for dropdown menu
    pub dropdown_position: Option<(f64, f64)>,

    /// Whether settings panel is open
    pub settings_open: bool,

    /// Whether help overlay is shown
    pub help_shown: bool,

    /// Toast/notification message
    pub toast_message: Option<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            magnitude_limit: 4.5,
            show_grid: true,
            show_constellations: false,
            quiz: None,
            score: ScoreState::default(),
            selected_star: None,
            ui: UiState::default(),
        }
    }
}

/// Actions that can modify the game state
#[derive(Debug, Clone)]
pub enum GameAction {
    // Viewport actions
    SetZoom(f64),
    ZoomBy(f64),
    Pan(f64, f64),
    SetCenter(f64, f64),
    ResetView,
    SetViewportSize(f64, f64),

    // Display settings
    SetMagnitudeLimit(f64),
    ToggleGrid,
    ToggleConstellations,

    // Star selection
    SelectStar(StarId),
    ClearSelection,

    // Quiz actions
    StartQuiz {
        target_star_id: StarId,
        correct_name: String,
        choices: Vec<String>,
    },
    SelectAnswer(String),
    SubmitAnswer,
    /// Combined action: select and immediately evaluate the answer
    SelectAndSubmitAnswer(String),
    CloseQuiz,
    NextQuestion,

    // UI actions
    SetDropdownPosition(f64, f64),
    ToggleSettings,
    ShowHelp,
    HideHelp,
    ShowToast(String),
    ClearToast,

    // Score
    ResetScore,

    /// Force a view refresh without changing zoom
    RefreshView,
}

/// Implement the reducer pattern for GameState
pub fn game_reducer(state: Rc<GameState>, action: GameAction) -> Rc<GameState> {
    let mut new_state: GameState = (*state).clone();

    match action {
        // Viewport actions
        GameAction::SetZoom(zoom) => {
            new_state.viewport.zoom = zoom.clamp(1.0, 50.0);
        }
        GameAction::ZoomBy(factor) => {
            new_state.viewport.zoom_by(factor, None);
        }
        GameAction::Pan(dx, dy) => {
            new_state.viewport.pan(dx, dy);
        }
        GameAction::SetCenter(ra, dec) => {
            new_state.viewport.center_ra = ra;
            new_state.viewport.center_dec = dec;
        }
        GameAction::ResetView => {
            new_state.viewport = Viewport::default();
            new_state.viewport.width = state.viewport.width;
            new_state.viewport.height = state.viewport.height;
        }
        GameAction::SetViewportSize(width, height) => {
            new_state.viewport.width = width;
            new_state.viewport.height = height;
        }

        // Display settings
        GameAction::SetMagnitudeLimit(mag) => {
            new_state.magnitude_limit = mag.clamp(1.0, 6.5);
        }
        GameAction::ToggleGrid => {
            new_state.show_grid = !new_state.show_grid;
        }
        GameAction::ToggleConstellations => {
            new_state.show_constellations = !new_state.show_constellations;
        }

        // Star selection
        GameAction::SelectStar(id) => {
            new_state.selected_star = Some(id);
        }
        GameAction::ClearSelection => {
            new_state.selected_star = None;
            new_state.quiz = None;
            new_state.ui.dropdown_position = None;
        }

        // Quiz actions
        GameAction::StartQuiz {
            target_star_id,
            correct_name,
            choices,
        } => {
            new_state.quiz = Some(QuizState {
                target_star_id,
                correct_name,
                choices,
                selected_answer: None,
                answered: false,
                was_correct: None,
            });
        }
        GameAction::SelectAnswer(answer) => {
            if let Some(ref mut quiz) = new_state.quiz {
                if !quiz.answered {
                    quiz.selected_answer = Some(answer);
                }
            }
        }
        GameAction::SubmitAnswer => {
            if let Some(ref mut quiz) = new_state.quiz {
                if !quiz.answered {
                    if let Some(ref answer) = quiz.selected_answer {
                        quiz.answered = true;
                        let correct = answer == &quiz.correct_name;
                        quiz.was_correct = Some(correct);

                        if correct {
                            new_state.score.record_correct();
                        } else {
                            new_state.score.record_incorrect();
                        }
                    }
                }
            }
        }
        GameAction::SelectAndSubmitAnswer(answer) => {
            if let Some(ref mut quiz) = new_state.quiz {
                if !quiz.answered {
                    quiz.selected_answer = Some(answer.clone());
                    quiz.answered = true;
                    let correct = answer == quiz.correct_name;
                    quiz.was_correct = Some(correct);

                    if correct {
                        new_state.score.record_correct();
                    } else {
                        new_state.score.record_incorrect();
                    }
                }
            }
        }
        GameAction::CloseQuiz => {
            new_state.quiz = None;
            new_state.selected_star = None;
            new_state.ui.dropdown_position = None;
            // Force refresh to redraw stars
            new_state.viewport.center_ra = (new_state.viewport.center_ra + 0.0001) % 24.0;
        }
        GameAction::NextQuestion => {
            new_state.quiz = None;
            new_state.selected_star = None;
            new_state.ui.dropdown_position = None;
            // Force refresh to redraw stars
            new_state.viewport.center_ra = (new_state.viewport.center_ra + 0.0001) % 24.0;
        }

        // UI actions
        GameAction::SetDropdownPosition(x, y) => {
            new_state.ui.dropdown_position = Some((x, y));
        }
        GameAction::ToggleSettings => {
            new_state.ui.settings_open = !new_state.ui.settings_open;
        }
        GameAction::ShowHelp => {
            new_state.ui.help_shown = true;
        }
        GameAction::HideHelp => {
            new_state.ui.help_shown = false;
        }
        GameAction::ShowToast(msg) => {
            new_state.ui.toast_message = Some(msg);
        }
        GameAction::ClearToast => {
            new_state.ui.toast_message = None;
        }

        // Score
        GameAction::ResetScore => {
            new_state.score = ScoreState::default();
        }

        // Force a view refresh by slightly nudging center_ra
        GameAction::RefreshView => {
            // Tiny nudge that forces re-render without visible change
            new_state.viewport.center_ra = (new_state.viewport.center_ra + 0.0001) % 24.0;
        }
    }

    Rc::new(new_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = GameState::default();
        assert_eq!(state.viewport.zoom, 1.0);
        assert!(state.show_grid);
        assert!(state.quiz.is_none());
    }

    #[test]
    fn test_score_tracking() {
        let mut score = ScoreState::default();

        score.record_correct();
        score.record_correct();
        score.record_incorrect();

        assert_eq!(score.correct, 2);
        assert_eq!(score.incorrect, 1);
        assert_eq!(score.streak, 0);
        assert_eq!(score.best_streak, 2);
    }

    #[test]
    fn test_accuracy() {
        let mut score = ScoreState::default();
        assert_eq!(score.accuracy(), 0.0);

        score.record_correct();
        score.record_correct();
        score.record_correct();
        score.record_incorrect();

        assert!((score.accuracy() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_reducer_zoom() {
        let state = Rc::new(GameState::default());
        let new_state = game_reducer(state, GameAction::SetZoom(2.0));

        assert_eq!(new_state.viewport.zoom, 2.0);
    }

    #[test]
    fn test_reducer_quiz_flow() {
        let state = Rc::new(GameState::default());

        // Start quiz
        let state = game_reducer(
            state,
            GameAction::StartQuiz {
                target_star_id: StarId(1),
                correct_name: "Sirius".into(),
                choices: vec!["Sirius".into(), "Vega".into(), "Arcturus".into()],
            },
        );
        assert!(state.quiz.is_some());

        // Select answer
        let state = game_reducer(state, GameAction::SelectAnswer("Sirius".into()));
        assert_eq!(
            state.quiz.as_ref().unwrap().selected_answer,
            Some("Sirius".into())
        );

        // Submit
        let state = game_reducer(state, GameAction::SubmitAnswer);
        assert!(state.quiz.as_ref().unwrap().answered);
        assert_eq!(state.quiz.as_ref().unwrap().was_correct, Some(true));
        assert_eq!(state.score.correct, 1);
    }

    #[test]
    fn test_magnitude_limit_clamp() {
        let state = Rc::new(GameState::default());

        let state = game_reducer(state, GameAction::SetMagnitudeLimit(10.0));
        assert_eq!(state.magnitude_limit, 6.5);

        let state = game_reducer(state, GameAction::SetMagnitudeLimit(0.0));
        assert_eq!(state.magnitude_limit, 1.0);
    }
}
