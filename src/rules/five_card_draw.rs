//! Five-Card Draw ruleset — classic draw poker.
//!
//! ## Rules
//!
//! - **Hole cards**: 5 per player (all face-down)
//! - **Community cards**: 0
//! - **Betting structure**: Fixed-Limit (default)
//! - **Players**: 2–6
//! - **Hand evaluation**: Your 5 cards are your hand (after optional draw)
//!
//! ### Game Flow
//!
//! 1. **Ante**: Each player posts a forced bet before cards are dealt.
//! 2. **Deal**: Each player receives 5 private cards face-down.
//! 3. **First Betting Round**: Starting left of the dealer.
//! 4. **Draw Phase**: Each player may discard 0–3 cards and receive replacements.
//! 5. **Second Betting Round**: Starting left of the dealer.
//! 6. **Showdown**: Best 5-card hand wins.
//!
//! ### Key Differences
//!
//! - No community cards — all information is private
//! - Only 2 betting rounds (vs 4 in Hold'em)
//! - Draw phase lets you improve your hand
//! - Fixed-limit betting with doubled increment in second round
//! - Smaller tables (max 6) due to card usage (5 cards × 6 players = 30 cards)

use super::{EvalMode, RuleSet};
use crate::engine::types::{BettingStructure, GamePhase};

/// Create the Five-Card Draw ruleset.
pub fn ruleset() -> RuleSet {
    RuleSet {
        name: "five_card_draw".to_string(),
        display_name: "Five-Card Draw".to_string(),
        description: "Classic draw poker — each player receives 5 private cards with no \
            community cards. After the first betting round, players may discard up to 3 cards \
            and draw replacements. A second betting round follows, then showdown. \
            Fixed-limit betting with doubled bets in the second round."
            .to_string(),
        hole_cards: 5,
        community_cards: 0,
        betting_structure: BettingStructure::FixedLimit,
        betting_rounds: vec![GamePhase::PreFlop, GamePhase::Draw],
        community_schedule: vec![],
        min_players: 2,
        max_players: 6,
        has_draw: true,
        max_discard: 3,
        eval_mode: EvalMode::ExactFive,
        uses_blinds: true,
        hi_lo: false,
        mixed_visibility: false,
        face_down_cards: 0,
        face_up_cards: 0,
    }
}
