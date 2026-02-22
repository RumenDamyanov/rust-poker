//! Request/response DTOs for the REST API.
//!
//! These are thin JSON-serializable structs separate from the engine types.
//! They control the exact API surface sent to clients.

use serde::{Deserialize, Serialize};

use crate::engine::game::{Game, ShowdownResult};
use crate::engine::types::{Card, GamePhase, PlayerStatus};
use crate::rules::RuleSet;

// ---------------------------------------------------------------------------
// Request DTOs
// ---------------------------------------------------------------------------

/// Request body for creating a new game.
#[derive(Deserialize)]
pub struct CreateGameRequest {
    /// Variant name (e.g. "texas_holdem", "omaha"). Defaults to "texas_holdem".
    pub variant: Option<String>,
    /// Small blind amount. Defaults to 10.
    pub small_blind: Option<u32>,
    /// Big blind amount. Defaults to 20.
    pub big_blind: Option<u32>,
}

/// Request body for joining a game.
#[derive(Deserialize)]
pub struct JoinRequest {
    /// Player display name.
    pub name: String,
    /// Starting chip count. Defaults to 1000.
    pub chips: Option<u32>,
}

/// Request body for a player action.
#[derive(Deserialize)]
pub struct ActionRequest {
    /// Player ID (UUID).
    pub player_id: String,
    /// Action type: "fold", "check", "call", "raise", "all_in", "discard".
    pub action: String,
    /// Amount for raise actions.
    pub amount: Option<u32>,
    /// Card indices to discard (Five-Card Draw).
    pub discard_indices: Option<Vec<usize>>,
}

// ---------------------------------------------------------------------------
// Response DTOs
// ---------------------------------------------------------------------------

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// JSON response for a single card.
#[derive(Serialize)]
pub struct CardResponse {
    pub rank: String,
    pub suit: String,
}

/// JSON response for a player (public view).
#[derive(Serialize)]
pub struct PlayerResponse {
    pub id: String,
    pub name: String,
    pub chips: u32,
    pub status: String,
    pub total_bet: u32,
    pub hole_cards: Option<Vec<CardResponse>>,
}

/// JSON response for a ruleset.
#[derive(Serialize)]
pub struct RuleSetResponse {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub min_players: usize,
    pub max_players: usize,
    pub hole_cards: usize,
    pub community_cards: usize,
    pub betting_structure: String,
}

/// JSON response for a showdown result.
#[derive(Serialize)]
pub struct ShowdownResponse {
    pub player_id: String,
    pub player_name: String,
    pub hand_rank: String,
    pub best_cards: Vec<CardResponse>,
    pub chips_won: u32,
}

/// Full game state response.
#[derive(Serialize)]
pub struct GameResponse {
    pub id: String,
    pub variant: String,
    pub phase: String,
    pub pot: u32,
    pub community_cards: Vec<CardResponse>,
    pub players: Vec<PlayerResponse>,
    pub current_player_id: Option<String>,
    pub available_actions: Vec<String>,
    pub dealer_position: usize,
    pub hand_number: u32,
    pub small_blind: u32,
    pub big_blind: u32,
    pub showdown: Vec<ShowdownResponse>,
}

/// Minimal game summary for listing.
#[derive(Serialize)]
pub struct GameSummary {
    pub id: String,
    pub variant: String,
    pub phase: String,
    pub player_count: usize,
    pub pot: u32,
}

/// Player-specific view (shows only their hole cards).
#[derive(Serialize)]
pub struct PlayerViewResponse {
    pub game: GameResponse,
    pub your_hole_cards: Vec<CardResponse>,
}

/// Action history response.
#[derive(Serialize)]
pub struct HistoryResponse {
    pub id: String,
    pub hand_number: u32,
    pub actions: Vec<ActionHistoryEntry>,
}

/// A single action in the history.
#[derive(Serialize)]
pub struct ActionHistoryEntry {
    pub player_id: String,
    pub action: String,
    pub phase: String,
}

// ---------------------------------------------------------------------------
// Conversions from engine types
// ---------------------------------------------------------------------------

impl CardResponse {
    /// Create from an engine `Card`.
    pub fn from_card(card: &Card) -> Self {
        Self {
            rank: card.rank.symbol().to_string(),
            suit: card.suit.name().to_string(),
        }
    }
}

impl RuleSetResponse {
    /// Build from a `RuleSet`.
    pub fn from_ruleset(rs: &RuleSet) -> Self {
        Self {
            name: rs.name.clone(),
            display_name: rs.display_name.clone(),
            description: rs.description.clone(),
            min_players: rs.min_players,
            max_players: rs.max_players,
            hole_cards: rs.hole_cards,
            community_cards: rs.community_cards,
            betting_structure: format!("{:?}", rs.betting_structure),
        }
    }
}

impl ShowdownResponse {
    /// Build from a `ShowdownResult`.
    pub fn from_result(r: &ShowdownResult) -> Self {
        Self {
            player_id: r.player_id.clone(),
            player_name: r.player_name.clone(),
            hand_rank: format!("{}", r.hand_rank),
            best_cards: r.best_cards.iter().map(CardResponse::from_card).collect(),
            chips_won: r.chips_won,
        }
    }
}

impl GameResponse {
    /// Build a game response from a `Game`.
    ///
    /// By default, hole cards are hidden (only shown in player-specific views).
    pub fn from_game(game: &Game) -> Self {
        let show_cards =
            game.phase() == GamePhase::Showdown || game.phase() == GamePhase::HandComplete;

        let players: Vec<PlayerResponse> = game
            .players()
            .iter()
            .map(|p| {
                let hole_cards = if show_cards && p.status() != PlayerStatus::Folded {
                    Some(p.hole_cards().iter().map(CardResponse::from_card).collect())
                } else {
                    None
                };

                PlayerResponse {
                    id: p.id().to_string(),
                    name: p.name().to_string(),
                    chips: p.chips(),
                    status: p.status().to_string(),
                    total_bet: p.total_bet(),
                    hole_cards,
                }
            })
            .collect();

        Self {
            id: game.id().to_string(),
            variant: game.ruleset().name.clone(),
            phase: game.phase().to_string(),
            pot: game.pot_total(),
            community_cards: game
                .community_cards()
                .iter()
                .map(CardResponse::from_card)
                .collect(),
            players,
            current_player_id: game.current_player_id().map(|s| s.to_string()),
            available_actions: game.available_actions(),
            dealer_position: game.dealer_position(),
            hand_number: game.hand_number(),
            small_blind: game.small_blind(),
            big_blind: game.big_blind(),
            showdown: game
                .last_showdown()
                .iter()
                .map(ShowdownResponse::from_result)
                .collect(),
        }
    }

    /// Build a player-specific view (shows that player's hole cards).
    pub fn with_player_view(game: &Game, player_id: &str) -> PlayerViewResponse {
        let response = Self::from_game(game);

        let your_cards = game
            .find_player(player_id)
            .map(|p| p.hole_cards().iter().map(CardResponse::from_card).collect())
            .unwrap_or_default();

        PlayerViewResponse {
            game: response,
            your_hole_cards: your_cards,
        }
    }
}

impl GameSummary {
    /// Build a summary from a `Game`.
    pub fn from_game(game: &Game) -> Self {
        Self {
            id: game.id().to_string(),
            variant: game.ruleset().name.clone(),
            phase: game.phase().to_string(),
            player_count: game.players().len(),
            pot: game.pot_total(),
        }
    }
}
