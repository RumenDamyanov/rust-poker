//! # rumenx-poker
//!
//! A multi-variant poker game engine and REST API server written in Rust.
//!
//! ## Supported Variants
//!
//! - **Texas Hold'em** — 2 hole cards, 5 community, no-limit
//! - **Omaha** — 4 hole cards, 5 community, pot-limit
//! - **Omaha Hi-Lo** — 4 hole cards, split pot (high/low)
//! - **Five-Card Draw** — 5 hole cards, draw phase
//! - **Seven-Card Stud** — 7 cards dealt per player
//!
//! ## Modules
//!
//! - [`engine`] — Pure game logic: cards, deck, hand evaluation, game state machine.
//! - [`rules`] — Poker variant definitions and hand evaluation dispatch.
//! - [`api`] — Axum HTTP server with REST endpoints.
//! - [`config`] — Environment-based server configuration.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rumenx_poker::engine::game::Game;
//! use rumenx_poker::rules::texas_holdem;
//!
//! let ruleset = texas_holdem::ruleset();
//! let mut game = Game::new(ruleset, 10, 20);
//!
//! game.join("Alice", 1000).expect("should join");
//! game.join("Bob", 1000).expect("should join");
//! game.start_hand().expect("should deal");
//!
//! println!("Phase: {}", game.phase());
//! println!("Players: {}", game.players().len());
//! ```

pub mod api;
pub mod config;
pub mod engine;
pub mod rules;
