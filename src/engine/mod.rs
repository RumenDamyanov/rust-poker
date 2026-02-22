//! Poker game engine.
//!
//! Pure game logic with no I/O or framework dependencies.
//! The engine can be used as a standalone library or behind the REST API.

pub mod betting;
pub mod deck;
pub mod evaluator;
pub mod game;
pub mod player;
pub mod pot;
pub mod types;
