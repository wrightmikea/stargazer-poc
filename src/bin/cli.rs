//! Stargazer CLI Tool
//!
//! Command-line interface for data generation, testing, and utilities.
//!
//! # Usage
//!
//! ```bash
//! # Generate placeholder star catalog
//! cargo run --bin stargazer-cli --features cli -- generate
//!
//! # Run quiz in terminal (for testing)
//! cargo run --bin stargazer-cli --features cli -- quiz
//!
//! # Show catalog statistics
//! cargo run --bin stargazer-cli --features cli -- stats
//! ```

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
use stargazer_poc::data::{generate_placeholder_catalog, BrightnessCategory};

#[cfg(feature = "cli")]
use stargazer_poc::game::{QuizConfig, QuizGenerator};

#[cfg(feature = "cli")]
use rand::SeedableRng;

#[cfg(feature = "cli")]
use std::io::{self, Write};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "stargazer-cli")]
#[command(about = "Stargazer CLI - Star catalog and quiz tools")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Generate and display the placeholder star catalog
    Generate {
        /// Output format (json, csv, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,
    },

    /// Show catalog statistics
    Stats,

    /// Run interactive quiz in terminal
    Quiz {
        /// Number of questions
        #[arg(short, long, default_value = "10")]
        count: usize,
    },

    /// List all named stars
    ListNamed {
        /// Maximum magnitude to show
        #[arg(short, long, default_value = "6.5")]
        max_magnitude: f64,
    },
}

#[cfg(feature = "cli")]
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { format } => {
            let catalog = generate_placeholder_catalog();

            match format.as_str() {
                "json" => {
                    let named: Vec<_> = catalog
                        .named_stars()
                        .iter()
                        .map(|s| {
                            serde_json::json!({
                                "id": s.id.0,
                                "name": s.name,
                                "ra": s.coord.ra,
                                "dec": s.coord.dec,
                                "magnitude": s.magnitude,
                                "constellation": s.constellation,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&named).unwrap());
                }
                "csv" => {
                    println!("id,name,ra,dec,magnitude,constellation");
                    for star in catalog.named_stars() {
                        println!(
                            "{},{},{:.3},{:.2},{:.2},{}",
                            star.id.0,
                            star.name.as_deref().unwrap_or(""),
                            star.coord.ra,
                            star.coord.dec,
                            star.magnitude,
                            star.constellation.as_deref().unwrap_or("")
                        );
                    }
                }
                _ => {
                    println!("Generated placeholder catalog:");
                    println!("  Total stars: {}", catalog.count());
                    println!("  Named stars: {}", catalog.named_count());
                    println!("\nSample of named stars:");
                    for star in catalog.named_stars().iter().take(10) {
                        println!(
                            "  {} (mag {:.2}) at RA {:.2}h, Dec {:.1}°",
                            star.display_name(),
                            star.magnitude,
                            star.coord.ra,
                            star.coord.dec
                        );
                    }
                }
            }
        }

        Commands::Stats => {
            let catalog = generate_placeholder_catalog();

            println!("=== Star Catalog Statistics ===\n");
            println!("Total stars:     {}", catalog.count());
            println!("Named stars:     {}", catalog.named_count());

            println!("\nBy brightness category:");
            for category in [
                BrightnessCategory::Brilliant,
                BrightnessCategory::Bright,
                BrightnessCategory::Medium,
                BrightnessCategory::Faint,
                BrightnessCategory::VeryFaint,
            ] {
                let count = catalog.stars_in_category(category).len();
                println!(
                    "  {:?} (mag < {:.1}): {} stars",
                    category,
                    category.magnitude_limit(),
                    count
                );
            }

            let named = catalog.named_stars();
            if !named.is_empty() {
                let avg_mag: f64 =
                    named.iter().map(|s| s.magnitude).sum::<f64>() / named.len() as f64;
                println!("\nNamed star statistics:");
                println!("  Average magnitude: {:.2}", avg_mag);

                let brightest = named
                    .iter()
                    .min_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());
                if let Some(star) = brightest {
                    println!(
                        "  Brightest: {} (mag {:.2})",
                        star.display_name(),
                        star.magnitude
                    );
                }
            }
        }

        Commands::Quiz { count } => {
            let catalog = generate_placeholder_catalog();
            let config = QuizConfig::default();
            let generator = QuizGenerator::new(&catalog, config);
            let mut rng = rand::rngs::SmallRng::from_entropy();

            let mut correct = 0;
            let mut total = 0;

            println!("=== Stargazer Quiz ===\n");
            println!("Answer each question by typing the number of your choice.\n");

            for q_num in 1..=count {
                if let Some(question) = generator.generate_random(&mut rng) {
                    println!("Question {}/{}:", q_num, count);
                    println!(
                        "Which star is located at RA {:.2}h, Dec {:.1}°?",
                        catalog
                            .get(question.target_star)
                            .map(|s| s.coord.ra)
                            .unwrap_or(0.0),
                        catalog
                            .get(question.target_star)
                            .map(|s| s.coord.dec)
                            .unwrap_or(0.0)
                    );

                    for (i, choice) in question.choices.iter().enumerate() {
                        println!("  {}. {}", i + 1, choice);
                    }

                    print!("\nYour answer: ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();

                    if let Ok(choice_num) = input.trim().parse::<usize>() {
                        if choice_num > 0 && choice_num <= question.choices.len() {
                            let selected = &question.choices[choice_num - 1];
                            if selected == &question.correct_answer {
                                println!("✓ Correct!\n");
                                correct += 1;
                            } else {
                                println!("✗ Wrong! The answer was: {}\n", question.correct_answer);
                            }
                            total += 1;
                        } else {
                            println!("Invalid choice.\n");
                        }
                    } else {
                        println!("Please enter a number.\n");
                    }
                }
            }

            println!("=== Results ===");
            println!(
                "Score: {}/{} ({:.0}%)",
                correct,
                total,
                if total > 0 {
                    (correct as f64 / total as f64) * 100.0
                } else {
                    0.0
                }
            );
        }

        Commands::ListNamed { max_magnitude } => {
            let catalog = generate_placeholder_catalog();

            println!("Named stars (magnitude < {:.1}):\n", max_magnitude);
            println!(
                "{:<20} {:>6} {:>8} {:>8} {:>10}",
                "Name", "Mag", "RA(h)", "Dec(°)", "Const"
            );
            println!("{}", "-".repeat(56));

            let mut named: Vec<_> = catalog
                .named_stars()
                .into_iter()
                .filter(|s| s.magnitude < max_magnitude)
                .collect();
            named.sort_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());

            for star in named {
                println!(
                    "{:<20} {:>6.2} {:>8.3} {:>8.2} {:>10}",
                    star.display_name(),
                    star.magnitude,
                    star.coord.ra,
                    star.coord.dec,
                    star.constellation.as_deref().unwrap_or("-")
                );
            }
        }
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Run with: cargo run --features cli");
}
