//! API error types with HTTP status mapping.
//!
//! Provides a unified error type for the REST API that automatically
//! converts to appropriate HTTP responses via Axum's `IntoResponse`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::engine::types::GameError;

/// Structured API error.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Game not found by ID.
    #[error("game not found: {0}")]
    GameNotFound(String),

    /// Player not found by ID.
    #[error("player not found: {0}")]
    PlayerNotFound(String),

    /// Action not valid in the current game state.
    #[error("{0}")]
    InvalidAction(String),

    /// Unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// JSON body for error responses.
#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_key) = match &self {
            ApiError::GameNotFound(_) => (StatusCode::NOT_FOUND, "game_not_found"),
            ApiError::PlayerNotFound(_) => (StatusCode::NOT_FOUND, "player_not_found"),
            ApiError::InvalidAction(_) => (StatusCode::BAD_REQUEST, "invalid_action"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = ErrorBody {
            error: error_key.to_string(),
            message: self.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}

/// Convert engine errors to API errors.
impl From<GameError> for ApiError {
    fn from(err: GameError) -> Self {
        match &err {
            GameError::PlayerNotFound(id) => ApiError::PlayerNotFound(id.clone()),
            GameError::NotYourTurn
            | GameError::InvalidAction(_)
            | GameError::InvalidBet(_)
            | GameError::InvalidDiscard(_)
            | GameError::WrongPhase { .. } => ApiError::InvalidAction(err.to_string()),
            GameError::TableFull { .. } | GameError::NotEnoughPlayers { .. } => {
                ApiError::InvalidAction(err.to_string())
            }
            GameError::DeckEmpty => ApiError::Internal("deck is empty".to_string()),
        }
    }
}
