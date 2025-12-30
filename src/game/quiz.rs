//! Quiz generation and management
//!
//! Handles the logic for creating quiz questions, selecting distractors,
//! and managing quiz sessions.

use crate::data::{Star, StarCatalog, StarId};
use rand::prelude::*;

/// Configuration for quiz generation
#[derive(Debug, Clone)]
pub struct QuizConfig {
    /// Number of choices to present (including correct answer)
    pub num_choices: usize,

    /// Whether to include "none of above" option
    pub include_none_option: bool,

    /// Probability of "none of above" being the correct answer
    pub none_probability: f64,
}

impl Default for QuizConfig {
    fn default() -> Self {
        Self {
            num_choices: 5,
            include_none_option: true,
            none_probability: 0.1,
        }
    }
}

/// A generated quiz question
#[derive(Debug, Clone)]
pub struct QuizQuestion {
    /// The star being asked about
    pub target_star: StarId,

    /// The correct answer (star name)
    pub correct_answer: String,

    /// All answer choices (shuffled)
    pub choices: Vec<String>,

    /// Whether this is a "none of above" question
    pub is_none_question: bool,
}

/// Quiz generator
pub struct QuizGenerator<'a> {
    catalog: &'a StarCatalog,
    config: QuizConfig,
}

impl<'a> QuizGenerator<'a> {
    /// Create a new quiz generator
    pub fn new(catalog: &'a StarCatalog, config: QuizConfig) -> Self {
        Self { catalog, config }
    }

    /// Generate a question for a specific star
    pub fn generate_for_star<R: Rng>(&self, star: &Star, rng: &mut R) -> Option<QuizQuestion> {
        let correct_name = star.name.clone()?;

        // Decide if this will be a "none of above" question
        let is_none_question = self.config.include_none_option
            && rng.gen::<f64>() < self.config.none_probability;

        let mut choices = Vec::with_capacity(self.config.num_choices);

        if is_none_question {
            // Get distractors (not including the correct answer)
            let distractors = self.catalog.random_distractors(
                &correct_name,
                self.config.num_choices - 1,
                rng,
            );
            choices.extend(distractors);
            choices.push("none of above".to_string());
        } else {
            // Include correct answer plus distractors
            choices.push(correct_name.clone());
            let distractors = self.catalog.random_distractors(
                &correct_name,
                self.config.num_choices - 1,
                rng,
            );
            choices.extend(distractors);
        }

        // Shuffle choices
        choices.shuffle(rng);

        let actual_correct = if is_none_question {
            "none of above".to_string()
        } else {
            correct_name
        };

        Some(QuizQuestion {
            target_star: star.id,
            correct_answer: actual_correct,
            choices,
            is_none_question,
        })
    }

    /// Generate a random question from named stars
    pub fn generate_random<R: Rng>(&self, rng: &mut R) -> Option<QuizQuestion> {
        let star = self.catalog.random_named_star(rng)?;
        self.generate_for_star(star, rng)
    }

    /// Generate a question for a star within a magnitude range
    pub fn generate_for_magnitude_range<R: Rng>(
        &self,
        min_mag: f64,
        max_mag: f64,
        rng: &mut R,
    ) -> Option<QuizQuestion> {
        let candidates: Vec<_> = self
            .catalog
            .named_stars()
            .into_iter()
            .filter(|s| s.magnitude >= min_mag && s.magnitude < max_mag)
            .collect();

        let star = candidates.choose(rng)?;
        self.generate_for_star(star, rng)
    }
}

/// Difficulty levels for the quiz
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Only very bright, famous stars (mag < 2)
    Easy,
    /// Bright stars (mag < 3.5)
    Medium,
    /// All named stars
    Hard,
}

impl Difficulty {
    /// Get the magnitude range for this difficulty
    pub fn magnitude_range(&self) -> (f64, f64) {
        match self {
            Difficulty::Easy => (-2.0, 2.0),
            Difficulty::Medium => (-2.0, 3.5),
            Difficulty::Hard => (-2.0, 6.5),
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::generate_placeholder_catalog;

    #[test]
    fn test_quiz_generation() {
        let catalog = generate_placeholder_catalog();
        let generator = QuizGenerator::new(&catalog, QuizConfig::default());
        let mut rng = rand::thread_rng();

        let question = generator.generate_random(&mut rng);
        assert!(question.is_some());

        let q = question.unwrap();
        assert_eq!(q.choices.len(), 5);
        assert!(q.choices.contains(&q.correct_answer));
    }

    #[test]
    fn test_quiz_for_specific_star() {
        let catalog = generate_placeholder_catalog();
        let generator = QuizGenerator::new(&catalog, QuizConfig::default());
        let mut rng = rand::thread_rng();

        // Find Sirius
        let sirius = catalog.named_stars().into_iter().find(|s| {
            s.name.as_deref() == Some("Sirius")
        });

        assert!(sirius.is_some());
        let q = generator.generate_for_star(sirius.unwrap(), &mut rng);
        assert!(q.is_some());
    }

    #[test]
    fn test_difficulty_magnitude_ranges() {
        assert!(Difficulty::Easy.magnitude_range().1 < Difficulty::Medium.magnitude_range().1);
        assert!(Difficulty::Medium.magnitude_range().1 < Difficulty::Hard.magnitude_range().1);
    }

    #[test]
    fn test_no_duplicate_choices() {
        let catalog = generate_placeholder_catalog();
        let config = QuizConfig {
            num_choices: 5,
            include_none_option: false,
            none_probability: 0.0,
        };
        let generator = QuizGenerator::new(&catalog, config);
        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            if let Some(q) = generator.generate_random(&mut rng) {
                let mut sorted = q.choices.clone();
                sorted.sort();
                sorted.dedup();
                assert_eq!(sorted.len(), q.choices.len(), "Duplicate choices found");
            }
        }
    }
}
