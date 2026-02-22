//! Omaha ruleset — higher-action poker with 4 hole cards.
//!
//! ## Rules
//!
//! - **Hole cards**: 4 per player (face-down)
//! - **Community cards**: 5 (flop 3, turn 1, river 1)
//! - **Betting structure**: Pot-Limit (default)
//! - **Players**: 2–9
//! - **Hand evaluation**: Must use **exactly 2** hole cards + **exactly 3** community cards
//!
//! ### Key Differences from Texas Hold'em
//!
//! - You receive 4 hole cards instead of 2
//! - You **must** use exactly 2 of your hole cards and 3 community cards
//! - This is the most common mistake new Omaha players make!
//! - Pot-limit betting: you can raise up to the current pot size
//! - More possible hand combinations → stronger average hands at showdown

use super::{EvalMode, RuleSet};
use crate::engine::types::{BettingStructure, GamePhase};

/// Create the Omaha ruleset.
pub fn ruleset() -> RuleSet {
    RuleSet {
        name: "omaha".to_string(),
        display_name: "Omaha".to_string(),
        description: "A high-action poker variant where each player receives 4 hole cards. \
            You must use exactly 2 of your hole cards and exactly 3 of the 5 community cards \
            to make your best 5-card hand. Pot-limit betting means you can raise up to the \
            current pot size."
            .to_string(),
        hole_cards: 4,
        community_cards: 5,
        betting_structure: BettingStructure::PotLimit,
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
        max_players: 9,
        has_draw: false,
        max_discard: 0,
        eval_mode: EvalMode::OmahaExact,
        uses_blinds: true,
        hi_lo: false,
        mixed_visibility: false,
        face_down_cards: 0,
        face_up_cards: 0,
    }
}
