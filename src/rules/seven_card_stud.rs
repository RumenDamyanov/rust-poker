//! Seven-Card Stud ruleset — classic stud poker with mixed visibility.
//!
//! ## Rules
//!
//! - **Cards per player**: 7 (3 face-down, 4 face-up)
//! - **Community cards**: 0
//! - **Betting structure**: Fixed-Limit (default)
//! - **Players**: 2–8
//! - **Hand evaluation**: Best 5 from 7
//!
//! ### Game Flow
//!
//! 1. **Ante**: Each player posts a small forced bet.
//! 2. **Third Street**: 2 cards face-down + 1 card face-up per player. Betting begins
//!    with the lowest face-up card (the "bring-in").
//! 3. **Fourth Street**: 1 more card face-up per player. Betting begins with the best
//!    visible hand.
//! 4. **Fifth Street**: 1 more card face-up. Betting with doubled increment.
//! 5. **Sixth Street**: 1 more card face-up. Betting with doubled increment.
//! 6. **Seventh Street (River)**: 1 final card face-down. Final betting round.
//! 7. **Showdown**: Best 5-card hand from 7 cards wins.
//!
//! ### Key Differences
//!
//! - No community cards — each player has their own 7 cards
//! - Some cards are visible to opponents (face-up)
//! - More information available than Hold'em (you can see some of opponents' cards)
//! - Antes instead of blinds
//! - "Bring-in" bet on third street (lowest showing card)
//! - Fixed-limit with doubling on later streets

use super::{EvalMode, RuleSet};
use crate::engine::types::{BettingStructure, GamePhase};

/// Create the Seven-Card Stud ruleset.
pub fn ruleset() -> RuleSet {
    RuleSet {
        name: "seven_card_stud".to_string(),
        display_name: "Seven-Card Stud".to_string(),
        description: "Classic stud poker with no community cards. Each player receives 7 \
            cards throughout the hand — 3 face-down and 4 face-up. Some of your cards are \
            visible to opponents, adding a layer of strategy. Make your best 5-card hand \
            from your 7 cards. Fixed-limit betting with antes instead of blinds."
            .to_string(),
        hole_cards: 7,
        community_cards: 0,
        betting_structure: BettingStructure::FixedLimit,
        betting_rounds: vec![
            GamePhase::PreFlop, // Third Street
            GamePhase::Flop,    // Fourth Street
            GamePhase::Turn,    // Fifth Street
            GamePhase::River,   // Sixth + Seventh Street
        ],
        community_schedule: vec![],
        min_players: 2,
        max_players: 8,
        has_draw: false,
        max_discard: 0,
        eval_mode: EvalMode::BestOfAll,
        uses_blinds: false, // Uses antes
        hi_lo: false,
        mixed_visibility: true,
        face_down_cards: 3, // 2 initial + 1 final
        face_up_cards: 4,   // 4 visible cards dealt during streets
    }
}
