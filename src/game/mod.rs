//! Game logic module
//!
//! Contains state management, quiz generation, and game rules.

pub mod quiz;
pub mod state;

pub use quiz::{Difficulty, QuizConfig, QuizGenerator, QuizQuestion};
pub use state::{game_reducer, GameAction, GameState, QuizState, ScoreState, UiState, GuessSummary};
