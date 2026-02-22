//! Pot management for poker games.
//!
//! Handles main pots and side pots created by all-in scenarios.

use serde::{Deserialize, Serialize};

/// A single pot (main or side) with a chip amount and eligible player IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidePot {
    /// Total chips in this pot.
    pub amount: u32,
    /// Player IDs eligible to win this pot.
    pub eligible: Vec<String>,
}

/// Manages the pot(s) for a poker hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotManager {
    /// All pots (first is the main pot, rest are side pots).
    pots: Vec<SidePot>,
}

impl PotManager {
    /// Create a new empty pot manager.
    pub fn new() -> Self {
        Self {
            pots: vec![SidePot {
                amount: 0,
                eligible: Vec::new(),
            }],
        }
    }

    /// Get all pots.
    pub fn pots(&self) -> &[SidePot] {
        &self.pots
    }

    /// Total chips across all pots.
    pub fn total(&self) -> u32 {
        self.pots.iter().map(|p| p.amount).sum()
    }

    /// Add chips to the main (current) pot.
    pub fn add(&mut self, amount: u32) {
        if let Some(pot) = self.pots.last_mut() {
            pot.amount += amount;
        }
    }

    /// Build pots from player bets at the end of a betting round.
    ///
    /// Takes a list of (player_id, total_bet_this_hand) sorted by bet size ascending.
    /// Creates main pot + any side pots needed for all-in players.
    pub fn build_from_bets(&mut self, mut player_bets: Vec<(String, u32)>) {
        // Sort by bet amount ascending.
        player_bets.sort_by_key(|(_, bet)| *bet);

        self.pots.clear();
        let mut prev_level = 0;

        for i in 0..player_bets.len() {
            let current_bet = player_bets[i].1;
            if current_bet > prev_level {
                let increment = current_bet - prev_level;
                // All players at or above this level contribute.
                let contributors: Vec<String> =
                    player_bets[i..].iter().map(|(id, _)| id.clone()).collect();
                let pot_amount = increment * contributors.len() as u32;
                self.pots.push(SidePot {
                    amount: pot_amount,
                    eligible: contributors,
                });
                prev_level = current_bet;
            }
        }

        // Ensure at least one pot exists.
        if self.pots.is_empty() {
            self.pots.push(SidePot {
                amount: 0,
                eligible: Vec::new(),
            });
        }
    }

    /// Reset the pot manager for a new hand.
    pub fn reset(&mut self) {
        self.pots = vec![SidePot {
            amount: 0,
            eligible: Vec::new(),
        }];
    }
}

impl Default for PotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pot() {
        let pot = PotManager::new();
        assert_eq!(pot.total(), 0);
        assert_eq!(pot.pots().len(), 1);
    }

    #[test]
    fn test_add_to_pot() {
        let mut pot = PotManager::new();
        pot.add(100);
        pot.add(200);
        assert_eq!(pot.total(), 300);
    }

    #[test]
    fn test_simple_pot_no_side_pots() {
        let mut pot = PotManager::new();
        // Three players all bet 100.
        pot.build_from_bets(vec![
            ("a".into(), 100),
            ("b".into(), 100),
            ("c".into(), 100),
        ]);
        assert_eq!(pot.pots().len(), 1);
        assert_eq!(pot.total(), 300);
        assert_eq!(pot.pots()[0].eligible.len(), 3);
    }

    #[test]
    fn test_single_all_in_side_pot() {
        let mut pot = PotManager::new();
        // Player A all-in for 50, B and C bet 100.
        pot.build_from_bets(vec![("a".into(), 50), ("b".into(), 100), ("c".into(), 100)]);
        assert_eq!(pot.pots().len(), 2);
        // Main pot: 50 × 3 = 150 (A, B, C)
        assert_eq!(pot.pots()[0].amount, 150);
        assert_eq!(pot.pots()[0].eligible.len(), 3);
        // Side pot: 50 × 2 = 100 (B, C only)
        assert_eq!(pot.pots()[1].amount, 100);
        assert_eq!(pot.pots()[1].eligible.len(), 2);
        assert_eq!(pot.total(), 250);
    }

    #[test]
    fn test_multiple_all_ins() {
        let mut pot = PotManager::new();
        // A: 30, B: 60, C: 100
        pot.build_from_bets(vec![("a".into(), 30), ("b".into(), 60), ("c".into(), 100)]);
        assert_eq!(pot.pots().len(), 3);
        // Main pot: 30 × 3 = 90
        assert_eq!(pot.pots()[0].amount, 90);
        // Side pot 1: 30 × 2 = 60
        assert_eq!(pot.pots()[1].amount, 60);
        // Side pot 2: 40 × 1 = 40
        assert_eq!(pot.pots()[2].amount, 40);
        assert_eq!(pot.total(), 190);
    }

    #[test]
    fn test_reset() {
        let mut pot = PotManager::new();
        pot.add(500);
        pot.reset();
        assert_eq!(pot.total(), 0);
        assert_eq!(pot.pots().len(), 1);
    }
}
