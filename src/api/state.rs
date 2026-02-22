//! Application state shared across API handlers.
//!
//! Holds the game registry — a thread-safe map of active poker games.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::engine::game::Game;

/// Shared application state (wrapped in `Arc` for Axum).
pub type SharedState = Arc<AppState>;

/// Application state holding all active games.
#[derive(Debug)]
pub struct AppState {
    /// Map of game ID → Game.
    pub games: RwLock<HashMap<String, Game>>,
}

impl AppState {
    /// Create a new empty application state.
    pub fn new() -> SharedState {
        Arc::new(Self {
            games: RwLock::new(HashMap::new()),
        })
    }
}
