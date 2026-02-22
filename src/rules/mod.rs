//! Configurable poker rulesets.
//!
//! Each poker variant is defined as a `RuleSet` struct that configures
//! hole cards, community cards, betting structure, and hand evaluation.

pub mod five_card_draw;
pub mod omaha;
pub mod omaha_hilo;
pub mod seven_card_stud;
pub mod texas_holdem;

use serde::{Deserialize, Serialize};

use crate::engine::types::{BettingStructure, Card, GamePhase, HandRank};

/// Configuration for a poker variant.
///
/// Instead of a trait, we use a data-driven config struct for simplicity
/// and ease of serialization. Variant-specific logic (like "must use exactly 2
/// hole cards" in Omaha) is handled by the `evaluate_fn` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    /// Internal name (e.g. "texas_holdem").
    pub name: String,
    /// Display name (e.g. "Texas Hold'em").
    pub display_name: String,
    /// Description of the variant for players.
    pub description: String,
    /// Number of hole cards dealt to each player.
    pub hole_cards: usize,
    /// Total community cards dealt during the hand.
    pub community_cards: usize,
    /// Betting structure.
    pub betting_structure: BettingStructure,
    /// Betting round phases in order.
    pub betting_rounds: Vec<GamePhase>,
    /// How many community cards to deal per phase: (phase, count).
    pub community_schedule: Vec<(GamePhase, usize)>,
    /// Minimum players to start.
    pub min_players: usize,
    /// Maximum players allowed.
    pub max_players: usize,
    /// Whether the variant has a draw phase.
    pub has_draw: bool,
    /// Maximum cards that can be discarded in draw phase.
    pub max_discard: usize,
    /// Hand evaluation mode.
    #[serde(skip)]
    pub eval_mode: EvalMode,
    /// Whether the variant uses blinds (vs antes).
    pub uses_blinds: bool,
    /// Whether the variant supports hi-lo split.
    pub hi_lo: bool,
    /// Stud-specific: whether some cards are dealt face-up.
    pub mixed_visibility: bool,
    /// Number of cards dealt face-down in stud variants.
    pub face_down_cards: usize,
    /// Number of cards dealt face-up in stud variants.
    pub face_up_cards: usize,
}

/// How to evaluate the best hand for this variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvalMode {
    /// Best 5 from all available cards (Texas Hold'em, Stud).
    #[default]
    BestOfAll,
    /// Must use exactly 2 hole cards + 3 community (Omaha).
    OmahaExact,
    /// Hand is exactly the 5 hole cards (Five-Card Draw).
    ExactFive,
    /// Omaha Hi-Lo split.
    OmahaHiLo,
}

/// Evaluate the best hand for a player given the ruleset's evaluation mode.
pub fn evaluate_hand(
    eval_mode: EvalMode,
    hole_cards: &[Card],
    community_cards: &[Card],
) -> (HandRank, Vec<Card>) {
    use crate::engine::evaluator;

    match eval_mode {
        EvalMode::BestOfAll => {
            let mut all_cards: Vec<Card> = hole_cards.to_vec();
            all_cards.extend_from_slice(community_cards);
            evaluator::best_hand(&all_cards)
        }
        EvalMode::OmahaExact | EvalMode::OmahaHiLo => {
            // For hi-lo, we just return the high hand here.
            // The low hand is evaluated separately.
            if hole_cards.len() == 4 && community_cards.len() == 5 {
                let hole: [Card; 4] = [hole_cards[0], hole_cards[1], hole_cards[2], hole_cards[3]];
                let comm: [Card; 5] = [
                    community_cards[0],
                    community_cards[1],
                    community_cards[2],
                    community_cards[3],
                    community_cards[4],
                ];
                evaluator::best_hand_omaha(&hole, &comm)
            } else {
                // Fallback if card counts don't match (shouldn't happen).
                let mut all: Vec<Card> = hole_cards.to_vec();
                all.extend_from_slice(community_cards);
                evaluator::best_hand(&all)
            }
        }
        EvalMode::ExactFive => {
            assert!(hole_cards.len() >= 5, "need 5 cards for draw poker");
            let five: [Card; 5] = [
                hole_cards[0],
                hole_cards[1],
                hole_cards[2],
                hole_cards[3],
                hole_cards[4],
            ];
            let rank = evaluator::evaluate_five(&five);
            (rank, five.to_vec())
        }
    }
}

/// Get all available rulesets.
pub fn all_rulesets() -> Vec<RuleSet> {
    vec![
        texas_holdem::ruleset(),
        omaha::ruleset(),
        omaha_hilo::ruleset(),
        five_card_draw::ruleset(),
        seven_card_stud::ruleset(),
    ]
}

/// Find a ruleset by name.
pub fn find_ruleset(name: &str) -> Option<RuleSet> {
    all_rulesets().into_iter().find(|r| r.name == name)
}
