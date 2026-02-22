//! Omaha Hi-Lo (8 or better) ruleset — split pot variant.
//!
//! ## Rules
//!
//! - **Hole cards**: 4 per player (face-down)
//! - **Community cards**: 5 (flop 3, turn 1, river 1)
//! - **Betting structure**: Pot-Limit (default)
//! - **Players**: 2–9
//! - **Hand evaluation**: Split pot — best high hand and best qualifying low hand
//!
//! ### How It Works
//!
//! The pot is split between the best high hand and the best qualifying low hand.
//!
//! **High hand**: Same as regular Omaha — must use exactly 2 hole + 3 community.
//!
//! **Low hand**: Must also use exactly 2 hole + 3 community, but:
//! - All 5 cards must be rank 8 or lower (Ace counts as low = 1)
//! - All 5 cards must have different ranks (no pairs)
//! - Straights and flushes don't count against you for the low
//! - The best possible low is A-2-3-4-5 ("the wheel")
//!
//! If no player has a qualifying low hand, the entire pot goes to the high hand winner.
//!
//! ### Example
//!
//! - Best low: A-2-3-4-5 (called "the wheel")
//! - Second best: A-2-3-4-6
//! - Not qualifying: A-2-3-4-9 (9 is too high)

use super::{EvalMode, RuleSet};
use crate::engine::types::{BettingStructure, GamePhase};

/// Create the Omaha Hi-Lo ruleset.
pub fn ruleset() -> RuleSet {
    RuleSet {
        name: "omaha_hilo".to_string(),
        display_name: "Omaha Hi-Lo".to_string(),
        description: "A split-pot Omaha variant where the pot is divided between the best \
            high hand and the best qualifying low hand. Low hands must have 5 cards of rank \
            8 or lower with no pairs (aces count as low). Must use exactly 2 hole cards + \
            3 community cards for both high and low hands. If no qualifying low exists, \
            the high hand takes the entire pot."
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
        eval_mode: EvalMode::OmahaHiLo,
        uses_blinds: true,
        hi_lo: true,
        mixed_visibility: false,
        face_down_cards: 0,
        face_up_cards: 0,
    }
}
