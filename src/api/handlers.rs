//! API endpoint handlers.
//!
//! Each function is a thin wrapper that delegates to the engine.
//! Handlers validate input, call engine methods, and return JSON responses.

use axum::Json;
use axum::extract::{Path, State};

use super::errors::ApiError;
use super::models::{
    ActionHistoryEntry, ActionRequest, CreateGameRequest, GameResponse, GameSummary,
    HealthResponse, HistoryResponse, JoinRequest, PlayerViewResponse, RuleSetResponse,
};
use super::state::SharedState;
use crate::engine::types::PlayerAction;
use crate::rules;

/// `GET /health` — Health check.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// `GET /api/rulesets` — List all available poker variants.
pub async fn list_rulesets() -> Json<Vec<RuleSetResponse>> {
    let rulesets: Vec<RuleSetResponse> = rules::all_rulesets()
        .iter()
        .map(RuleSetResponse::from_ruleset)
        .collect();
    Json(rulesets)
}

/// `POST /api/games` — Create a new poker game.
pub async fn create_game(
    State(state): State<SharedState>,
    Json(body): Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, ApiError> {
    let variant = body.variant.as_deref().unwrap_or("texas_holdem");
    let ruleset = rules::find_ruleset(variant)
        .ok_or_else(|| ApiError::InvalidAction(format!("unknown variant: {variant}")))?;

    let small_blind = body.small_blind.unwrap_or(10);
    let big_blind = body.big_blind.unwrap_or(20);

    let game = crate::engine::game::Game::new(ruleset, small_blind, big_blind);
    let response = GameResponse::from_game(&game);
    let id = game.id().to_string();

    state.games.write().await.insert(id, game);

    Ok(Json(response))
}

/// `GET /api/games` — List all active games.
pub async fn list_games(State(state): State<SharedState>) -> Json<Vec<GameSummary>> {
    let games = state.games.read().await;
    let summaries: Vec<GameSummary> = games.values().map(GameSummary::from_game).collect();
    Json(summaries)
}

/// `GET /api/games/{id}` — Get game state.
pub async fn get_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let games = state.games.read().await;
    let game = games.get(&id).ok_or(ApiError::GameNotFound(id))?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `DELETE /api/games/{id}` — Delete a game.
pub async fn delete_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut games = state.games.write().await;
    games
        .remove(&id)
        .ok_or(ApiError::GameNotFound(id.clone()))?;
    Ok(Json(serde_json::json!({ "deleted": id })))
}

/// `POST /api/games/{id}/join` — Join a game.
pub async fn join_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<JoinRequest>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;

    let chips = body.chips.unwrap_or(1000);
    game.join(&body.name, chips).map_err(ApiError::from)?;

    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/leave` — Leave a game.
pub async fn leave_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;

    let player_id = body["player_id"]
        .as_str()
        .ok_or_else(|| ApiError::InvalidAction("missing player_id".into()))?;

    game.leave(player_id).map_err(ApiError::from)?;

    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/start` — Start a new hand.
pub async fn start_hand(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;

    game.start_hand().map_err(ApiError::from)?;

    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/action` — Submit a player action.
pub async fn player_action(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<ActionRequest>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;

    let action = parse_action(&body)?;
    game.player_action(&body.player_id, action)
        .map_err(ApiError::from)?;

    Ok(Json(GameResponse::from_game(game)))
}

/// `GET /api/games/{id}/player/{player_id}` — Get player-specific view.
pub async fn player_view(
    State(state): State<SharedState>,
    Path((id, player_id)): Path<(String, String)>,
) -> Result<Json<PlayerViewResponse>, ApiError> {
    let games = state.games.read().await;
    let game = games.get(&id).ok_or(ApiError::GameNotFound(id))?;

    if game.find_player(&player_id).is_none() {
        return Err(ApiError::PlayerNotFound(player_id));
    }

    Ok(Json(GameResponse::with_player_view(game, &player_id)))
}

/// `GET /api/games/{id}/history` — Get action history.
pub async fn history(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let games = state.games.read().await;
    let game = games.get(&id).ok_or(ApiError::GameNotFound(id))?;

    let actions: Vec<ActionHistoryEntry> = game
        .history()
        .iter()
        .map(|a| ActionHistoryEntry {
            player_id: a.player_id.clone(),
            action: a.action.to_string(),
            phase: a.phase.to_string(),
        })
        .collect();

    Ok(Json(HistoryResponse {
        id: game.id().to_string(),
        hand_number: game.hand_number(),
        actions,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse an action string into a `PlayerAction`.
fn parse_action(body: &ActionRequest) -> Result<PlayerAction, ApiError> {
    match body.action.to_lowercase().as_str() {
        "fold" => Ok(PlayerAction::Fold),
        "check" => Ok(PlayerAction::Check),
        "call" => Ok(PlayerAction::Call),
        "raise" => {
            let amount = body
                .amount
                .ok_or_else(|| ApiError::InvalidAction("raise requires an amount".into()))?;
            Ok(PlayerAction::Raise(amount))
        }
        "all_in" | "allin" | "all-in" => Ok(PlayerAction::AllIn),
        "discard" => {
            let indices = body.discard_indices.clone().ok_or_else(|| {
                ApiError::InvalidAction("discard requires discard_indices".into())
            })?;
            Ok(PlayerAction::Discard(indices))
        }
        other => Err(ApiError::InvalidAction(format!("unknown action: {other}"))),
    }
}
