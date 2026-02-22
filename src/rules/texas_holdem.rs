//! Texas Hold'em ruleset — the world's most popular poker variant.
//!
//! ## Rules
//!
//! - **Hole cards**: 2 per player (face-down)
//! - **Community cards**: 5 (flop 3, turn 1, river 1)
//! - **Betting structure**: No-Limit (default)
//! - **Players**: 2–10
//! - **Hand evaluation**: Best 5 from 7 (any combination of hole + community)
//!
//! ### Game Flow
//!
//! 1. **Pre-Flop**: Each player receives 2 hole cards. Betting starts left of big blind.
//! 2. **Flop**: 3 community cards dealt face-up. Betting starts left of dealer.
//! 3. **Turn**: 1 more community card. Betting round.
//! 4. **River**: 1 final community card. Final betting round.
//! 5. **Showdown**: Best 5-card hand from any combination of 7 cards wins.

use super::{EvalMode, RuleSet};
use crate::engine::types::{BettingStructure, GamePhase};

/// Create the Texas Hold'em ruleset.
pub fn ruleset() -> RuleSet {
    RuleSet {
        name: "texas_holdem".to_string(),
        display_name: "Texas Hold'em".to_string(),
        description: "The world's most popular poker variant. Each player receives 2 private \
            hole cards and shares 5 community cards. Make the best 5-card hand from any \
            combination of your hole cards and the community cards. No-limit betting means \
            you can raise up to your entire chip stack at any time."
            .to_string(),
        hole_cards: 2,
        community_cards: 5,
        betting_structure: BettingStructure::NoLimit,
        betting_rounds: vec![
            GamePhase::PreFlop,
            GamePhase::Flop,
            GamePhase::Turn,
            GamePhase::River,
        ],
        community_schedule: vec![
            (GamePhase::Flop, 3),
            (GamePhase::Turn, 1),
            (GamePhase::River, 1),
        ],
        min_players: 2,
        max_players: 10,
        has_draw: false,
        max_discard: 0,
        eval_mode: EvalMode::BestOfAll,
        uses_blinds: true,
        hi_lo: false,
        mixed_visibility: false,
        face_down_cards: 0,
        face_up_cards: 0,
    }
}
