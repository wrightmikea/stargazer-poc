//! UI Components for the Stargazer application
//!
//! Built with Yew framework for WebAssembly rendering.

pub mod controls;
pub mod quiz_dropdown;
pub mod score_display;
pub mod star_map;
pub mod summary_popup;

pub use controls::Controls;
pub use quiz_dropdown::QuizDropdown;
pub use score_display::ScoreDisplay;
pub use star_map::StarMap;
pub use summary_popup::SummaryPopup;
