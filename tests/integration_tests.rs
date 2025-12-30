//! Integration tests for Stargazer
//!
//! These tests verify the integration between different modules.

use stargazer_poc::data::{generate_placeholder_catalog, BrightnessCategory, CelestialCoord};
use stargazer_poc::game::{game_reducer, GameAction, GameState, QuizConfig, QuizGenerator};
use stargazer_poc::utils::{LodSettings, Projection, Viewport};
use std::rc::Rc;

#[test]
fn test_catalog_quiz_integration() {
    // Generate catalog
    let catalog = generate_placeholder_catalog();

    // Create quiz generator
    let config = QuizConfig {
        num_choices: 5,
        include_none_option: false,
        none_probability: 0.0,
    };
    let generator = QuizGenerator::new(&catalog, config);

    // Generate multiple questions
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let question = generator.generate_random(&mut rng);
        assert!(question.is_some());

        let q = question.unwrap();
        assert_eq!(q.choices.len(), 5);
        assert!(q.choices.contains(&q.correct_answer));

        // Verify target star exists
        let star = catalog.get(q.target_star);
        assert!(star.is_some());
    }
}

#[test]
fn test_viewport_projection_consistency() {
    let viewport = Viewport::new(1200.0, 600.0);

    // Test multiple coordinates
    let test_coords = vec![
        CelestialCoord::new(0.0, 0.0),
        CelestialCoord::new(12.0, 45.0),
        CelestialCoord::new(6.0, -30.0),
        CelestialCoord::new(18.0, 60.0),
    ];

    for coord in test_coords {
        let screen = viewport.celestial_to_screen(&coord);
        let back = viewport.screen_to_celestial(screen);

        assert!(back.is_some(), "Failed to reverse project {:?}", coord);

        let recovered = back.unwrap();
        assert!(
            (coord.ra - recovered.ra).abs() < 0.01 || (coord.ra - recovered.ra).abs() > 23.99,
            "RA mismatch: {} vs {}",
            coord.ra,
            recovered.ra
        );
        assert!(
            (coord.dec - recovered.dec).abs() < 0.1,
            "Dec mismatch: {} vs {}",
            coord.dec,
            recovered.dec
        );
    }
}

#[test]
fn test_game_state_full_quiz_flow() {
    let catalog = generate_placeholder_catalog();
    let state = Rc::new(GameState::default());

    // Find a named star
    let sirius = catalog
        .named_stars()
        .into_iter()
        .find(|s| s.name.as_deref() == Some("Sirius"))
        .expect("Sirius should exist");

    // Select the star
    let state = game_reducer(state, GameAction::SelectStar(sirius.id));
    assert_eq!(state.selected_star, Some(sirius.id));

    // Start quiz
    let state = game_reducer(
        state,
        GameAction::StartQuiz {
            target_star_id: sirius.id,
            correct_name: "Sirius".into(),
            choices: vec![
                "Sirius".into(),
                "Vega".into(),
                "Arcturus".into(),
                "Betelgeuse".into(),
                "none of above".into(),
            ],
        },
    );
    assert!(state.quiz.is_some());

    // Select wrong answer
    let state = game_reducer(state, GameAction::SelectAnswer("Vega".into()));
    assert_eq!(
        state.quiz.as_ref().unwrap().selected_answer,
        Some("Vega".into())
    );

    // Submit wrong answer
    let state = game_reducer(state, GameAction::SubmitAnswer);
    assert!(state.quiz.as_ref().unwrap().answered);
    assert_eq!(state.quiz.as_ref().unwrap().was_correct, Some(false));
    assert_eq!(state.score.incorrect, 1);
    assert_eq!(state.score.correct, 0);

    // Close quiz
    let state = game_reducer(state, GameAction::CloseQuiz);
    assert!(state.quiz.is_none());

    // Start another quiz with correct answer
    let state = game_reducer(
        state,
        GameAction::StartQuiz {
            target_star_id: sirius.id,
            correct_name: "Sirius".into(),
            choices: vec!["Sirius".into(), "Vega".into()],
        },
    );

    let state = game_reducer(state, GameAction::SelectAnswer("Sirius".into()));
    let state = game_reducer(state, GameAction::SubmitAnswer);

    assert_eq!(state.quiz.as_ref().unwrap().was_correct, Some(true));
    assert_eq!(state.score.correct, 1);
    assert_eq!(state.score.incorrect, 1);
    assert_eq!(state.score.streak, 1);
}

#[test]
fn test_lod_with_catalog() {
    let catalog = generate_placeholder_catalog();
    let lod = LodSettings::default();

    // At zoom 1, fewer stars visible
    let zoom1_mag = lod.magnitude_limit(1.0);
    let stars_zoom1 = catalog.stars_brighter_than(zoom1_mag).len();

    // At zoom 5, more stars visible
    let zoom5_mag = lod.magnitude_limit(5.0);
    let stars_zoom5 = catalog.stars_brighter_than(zoom5_mag).len();

    assert!(stars_zoom5 > stars_zoom1);
}

#[test]
fn test_brightness_categories_with_catalog() {
    let catalog = generate_placeholder_catalog();

    let brilliant = catalog.stars_in_category(BrightnessCategory::Brilliant);
    let bright = catalog.stars_in_category(BrightnessCategory::Bright);
    let medium = catalog.stars_in_category(BrightnessCategory::Medium);

    // Categories should be cumulative
    assert!(brilliant.len() <= bright.len());
    assert!(bright.len() <= medium.len());

    // Brilliant stars should be very bright
    for star in &brilliant {
        assert!(star.magnitude < BrightnessCategory::Brilliant.magnitude_limit());
    }
}

#[test]
fn test_viewport_zoom_maintains_named_star_visibility() {
    let catalog = generate_placeholder_catalog();
    let viewport = Viewport::default();

    // Find a star in the initial view
    let (ra_min, ra_max) = viewport.ra_range();
    let (dec_min, dec_max) = viewport.dec_range();

    // Helper function to check if RA is in range, handling wraparound
    let ra_in_range = |ra: f64| {
        // When full sky is visible (ra_min == ra_max), all RAs are visible
        if (ra_max - ra_min).abs() < 0.001 && viewport.zoom <= 1.01 {
            true // Full sky view
        } else if ra_min <= ra_max {
            ra >= ra_min && ra <= ra_max
        } else {
            ra >= ra_min || ra <= ra_max // Wraparound case
        }
    };

    let visible_named: Vec<_> = catalog
        .named_stars()
        .into_iter()
        .filter(|s| s.coord.dec >= dec_min && s.coord.dec <= dec_max && ra_in_range(s.coord.ra))
        .collect();

    // Should have some named stars visible
    assert!(
        !visible_named.is_empty(),
        "Expected visible named stars at zoom {}",
        viewport.zoom
    );

    // Each should project to a valid screen position
    for star in visible_named {
        let screen = viewport.celestial_to_screen(&star.coord);
        assert!(
            screen.x >= 0.0 && screen.x <= viewport.width,
            "Star {} x={} out of bounds",
            star.display_name(),
            screen.x
        );
        assert!(
            screen.y >= 0.0 && screen.y <= viewport.height,
            "Star {} y={} out of bounds",
            star.display_name(),
            screen.y
        );
    }
}

#[test]
fn test_score_persistence_through_actions() {
    let state = Rc::new(GameState::default());

    // Simulate multiple correct answers
    let mut current_state = state;
    for i in 1..=5 {
        current_state = game_reducer(
            current_state,
            GameAction::StartQuiz {
                target_star_id: stargazer_poc::data::StarId(i),
                correct_name: "Test".into(),
                choices: vec!["Test".into()],
            },
        );
        current_state = game_reducer(current_state, GameAction::SelectAnswer("Test".into()));
        current_state = game_reducer(current_state, GameAction::SubmitAnswer);
        current_state = game_reducer(current_state, GameAction::CloseQuiz);
    }

    assert_eq!(current_state.score.correct, 5);
    assert_eq!(current_state.score.streak, 5);
    assert_eq!(current_state.score.best_streak, 5);

    // Now get one wrong
    current_state = game_reducer(
        current_state,
        GameAction::StartQuiz {
            target_star_id: stargazer_poc::data::StarId(6),
            correct_name: "Right".into(),
            choices: vec!["Right".into(), "Wrong".into()],
        },
    );
    current_state = game_reducer(current_state, GameAction::SelectAnswer("Wrong".into()));
    current_state = game_reducer(current_state, GameAction::SubmitAnswer);

    assert_eq!(current_state.score.correct, 5);
    assert_eq!(current_state.score.incorrect, 1);
    assert_eq!(current_state.score.streak, 0);
    assert_eq!(current_state.score.best_streak, 5); // Best streak preserved
}
