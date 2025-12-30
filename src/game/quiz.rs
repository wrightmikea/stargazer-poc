//! Quiz generation and management
//!
//! Handles logic for creating quiz questions, selecting distractors,
//! and managing quiz sessions.

use crate::data::{Star, StarCatalog, StarId, TileSystem, ZoomLevel};
use rand::prelude::*;
use std::collections::HashSet;

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
    tile_system: Option<&'a TileSystem>,
    current_zoom: ZoomLevel,
}

impl<'a> QuizGenerator<'a> {
    /// Create a new quiz generator
    pub fn new(catalog: &'a StarCatalog, config: QuizConfig) -> Self {
        Self {
            catalog,
            config,
            tile_system: None,
            current_zoom: ZoomLevel(0),
        }
    }

    /// Create quiz generator with tile system for spatial distractors
    pub fn with_tiles(
        catalog: &'a StarCatalog,
        config: QuizConfig,
        tile_system: &'a TileSystem,
        zoom: ZoomLevel,
    ) -> Self {
        Self {
            catalog,
            config,
            tile_system: Some(tile_system),
            current_zoom: zoom,
        }
    }

    /// Update current zoom level (for tile-based distractor selection)
    pub fn set_zoom(&mut self, zoom: ZoomLevel) {
        self.current_zoom = zoom;
    }

    /// Generate tile-aware distractors (mix of nearby and distant stars)
    ///
    /// This method selects distractors from:
    /// - Same tile (if available)
    /// - Adjacent tiles (nearby stars)
    /// - Random distant tiles
    pub fn generate_tile_distractors<R: Rng>(
        &self,
        correct_star: &Star,
        count: usize,
        rng: &mut R,
    ) -> Vec<String> {
        let mut used_names: HashSet<String> = HashSet::new();
        let mut distractors = Vec::new();
        let correct_name = correct_star.name.clone().unwrap_or_default();

        used_names.insert(correct_name.clone());

        // Try to use tile system if available
        if let Some(tile_system) = self.tile_system.as_ref() {
            if let Some(star_tiles) = tile_system.get_tiles_for_star(correct_star.id) {
                // Find tile at current zoom
                let current_tile = star_tiles.iter().find(|t| t.zoom == self.current_zoom);

                if let Some(tile_id) = current_tile {
                    // Get named stars in same tile
                    if let Some(tile) = tile_system.get_tile(tile_id) {
                        for &star_id in &tile.named_star_ids {
                            if let Some(star) = self.catalog.get(star_id) {
                                if let Some(ref name) = star.name {
                                    if !used_names.contains(name) && name.len() >= 3 {
                                        distractors.push(name.clone());
                                        used_names.insert(name.clone());

                                        if distractors.len() >= count {
                                            return distractors;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // If still need more, check adjacent tiles
                    if distractors.len() < count {
                        let adjacent = tile_system.get_adjacent_tiles(tile_id);

                        for tile in adjacent {
                            for &star_id in &tile.named_star_ids {
                                if let Some(star) = self.catalog.get(star_id) {
                                    if let Some(ref name) = star.name {
                                        if !used_names.contains(name) && name.len() >= 3 {
                                            distractors.push(name.clone());
                                            used_names.insert(name.clone());

                                            if distractors.len() >= count {
                                                return distractors;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if distractors.len() >= count {
                            return distractors;
                        }
                    }
                }
            }
        }

        // Fall back to random distant stars if needed
        let all_named: Vec<_> = self
            .catalog
            .named_stars()
            .into_iter()
            .filter(|s| {
                s.name
                    .as_ref()
                    .is_some_and(|n| !used_names.contains(n) && n.len() >= 3)
            })
            .collect();

        let remaining = count - distractors.len();
        let random_stars: Vec<_> = all_named.choose_multiple(rng, remaining).collect();

        for star in random_stars {
            if let Some(ref name) = star.name {
                distractors.push(name.clone());
                used_names.insert(name.clone());
            }
        }

        distractors
    }

    /// Generate a question for a specific star
    pub fn generate_for_star<R: Rng>(&self, star: &Star, rng: &mut R) -> Option<QuizQuestion> {
        let correct_name = star.name.clone()?;

        // Decide if this will be a "none of above" question
        let is_none_question =
            self.config.include_none_option && rng.gen::<f64>() < self.config.none_probability;

        let mut choices = Vec::with_capacity(self.config.num_choices);

        if is_none_question {
            // Use tile-aware distractors if available, otherwise random
            let distractors = if self.tile_system.is_some() {
                self.generate_tile_distractors(star, self.config.num_choices - 1, rng)
            } else {
                self.catalog
                    .random_distractors(&correct_name, self.config.num_choices - 1, rng)
            };

            choices.extend(distractors);
            choices.push("none of above".to_string());
        } else {
            // Include correct answer
            choices.push(correct_name.clone());

            // Use tile-aware distractors if available, otherwise random
            let distractors = if self.tile_system.is_some() {
                self.generate_tile_distractors(star, self.config.num_choices - 1, rng)
            } else {
                self.catalog
                    .random_distractors(&correct_name, self.config.num_choices - 1, rng)
            };

            choices.extend(distractors);
        }

        // Decide if this will be a "none of above" question
        let is_none_question =
            self.config.include_none_option && rng.gen::<f64>() < self.config.none_probability;

        let mut choices = Vec::with_capacity(self.config.num_choices);

        if is_none_question {
            // Get distractors (not including the correct answer)
            let distractors =
                self.catalog
                    .random_distractors(&correct_name, self.config.num_choices - 1, rng);
            choices.extend(distractors);
            choices.push("none of above".to_string());
        } else {
            // Include correct answer plus distractors
            choices.push(correct_name.clone());
            let distractors =
                self.catalog
                    .random_distractors(&correct_name, self.config.num_choices - 1, rng);
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
        let sirius = catalog
            .named_stars()
            .into_iter()
            .find(|s| s.name.as_deref() == Some("Sirius"));

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
