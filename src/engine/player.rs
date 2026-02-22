//! Player state management for a poker table.

use serde::{Deserialize, Serialize};

use super::types::{Card, PlayerStatus};

/// A player at the poker table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player ID.
    id: String,
    /// Display name.
    name: String,
    /// Current chip count.
    chips: u32,
    /// Hole cards dealt to this player.
    hole_cards: Vec<Card>,
    /// Current status in the hand.
    status: PlayerStatus,
    /// Amount bet in the current betting round.
    current_bet: u32,
    /// Total amount invested in the current hand (across all rounds).
    total_bet: u32,
}

impl Player {
    /// Create a new player with a name and starting chips.
    pub fn new(name: impl Into<String>, chips: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            chips,
            hole_cards: Vec::new(),
            status: PlayerStatus::Waiting,
            current_bet: 0,
            total_bet: 0,
        }
    }

    /// Create a new player with a specific ID.
    pub fn with_id(id: impl Into<String>, name: impl Into<String>, chips: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            chips,
            hole_cards: Vec::new(),
            status: PlayerStatus::Waiting,
            current_bet: 0,
            total_bet: 0,
        }
    }

    // -- Getters ---------------------------------------------------------

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn chips(&self) -> u32 {
        self.chips
    }

    pub fn hole_cards(&self) -> &[Card] {
        &self.hole_cards
    }

    pub fn status(&self) -> PlayerStatus {
        self.status
    }

    pub fn current_bet(&self) -> u32 {
        self.current_bet
    }

    pub fn total_bet(&self) -> u32 {
        self.total_bet
    }

    /// Whether this player can still act (active or waiting).
    pub fn is_active(&self) -> bool {
        matches!(self.status, PlayerStatus::Active | PlayerStatus::AllIn)
    }

    /// Whether this player can make betting decisions.
    pub fn can_act(&self) -> bool {
        self.status == PlayerStatus::Active
    }

    // -- Mutators --------------------------------------------------------

    /// Set the player's status.
    pub fn set_status(&mut self, status: PlayerStatus) {
        self.status = status;
    }

    /// Deal hole cards to this player.
    pub fn deal_cards(&mut self, cards: Vec<Card>) {
        self.hole_cards = cards;
        self.status = PlayerStatus::Active;
    }

    /// Clear the player's hole cards (end of hand).
    pub fn clear_cards(&mut self) {
        self.hole_cards.clear();
    }

    /// Place a bet, deducting from chips.
    /// Returns the actual amount bet (may be less if going all-in).
    pub fn place_bet(&mut self, amount: u32) -> u32 {
        let actual = amount.min(self.chips);
        self.chips -= actual;
        self.current_bet += actual;
        self.total_bet += actual;
        if self.chips == 0 {
            self.status = PlayerStatus::AllIn;
        }
        actual
    }

    /// Reset the current-round bet (called between betting rounds).
    pub fn reset_round_bet(&mut self) {
        self.current_bet = 0;
    }

    /// Award chips to this player (pot winnings).
    pub fn award_chips(&mut self, amount: u32) {
        self.chips += amount;
    }

    /// Reset player state for a new hand.
    pub fn new_hand(&mut self) {
        self.hole_cards.clear();
        self.current_bet = 0;
        self.total_bet = 0;
        if self.chips > 0 {
            self.status = PlayerStatus::Waiting;
        } else {
            self.status = PlayerStatus::Eliminated;
        }
    }

    /// Replace hole cards at the given indices with new cards (for Five-Card Draw).
    pub fn replace_cards(&mut self, indices: &[usize], new_cards: &[Card]) {
        for (i, &idx) in indices.iter().enumerate() {
            if idx < self.hole_cards.len() && i < new_cards.len() {
                self.hole_cards[idx] = new_cards[i];
            }
        }
    }

    /// Mark the player as folded.
    pub fn fold(&mut self) {
        self.status = PlayerStatus::Folded;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_player() {
        let p = Player::new("Alice", 1000);
        assert_eq!(p.name(), "Alice");
        assert_eq!(p.chips(), 1000);
        assert_eq!(p.status(), PlayerStatus::Waiting);
        assert!(p.hole_cards().is_empty());
    }

    #[test]
    fn test_place_bet() {
        let mut p = Player::new("Alice", 1000);
        p.set_status(PlayerStatus::Active);
        let actual = p.place_bet(200);
        assert_eq!(actual, 200);
        assert_eq!(p.chips(), 800);
        assert_eq!(p.current_bet(), 200);
        assert_eq!(p.total_bet(), 200);
    }

    #[test]
    fn test_place_bet_all_in() {
        let mut p = Player::new("Alice", 100);
        p.set_status(PlayerStatus::Active);
        let actual = p.place_bet(500); // More than chips
        assert_eq!(actual, 100);
        assert_eq!(p.chips(), 0);
        assert_eq!(p.status(), PlayerStatus::AllIn);
    }

    #[test]
    fn test_award_chips() {
        let mut p = Player::new("Alice", 1000);
        p.award_chips(500);
        assert_eq!(p.chips(), 1500);
    }

    #[test]
    fn test_fold() {
        let mut p = Player::new("Alice", 1000);
        p.set_status(PlayerStatus::Active);
        p.fold();
        assert_eq!(p.status(), PlayerStatus::Folded);
    }

    #[test]
    fn test_new_hand_reset() {
        let mut p = Player::new("Alice", 1000);
        p.set_status(PlayerStatus::Active);
        p.place_bet(200);
        p.new_hand();
        assert_eq!(p.current_bet(), 0);
        assert_eq!(p.total_bet(), 0);
        assert!(p.hole_cards().is_empty());
        assert_eq!(p.status(), PlayerStatus::Waiting);
    }

    #[test]
    fn test_new_hand_eliminated() {
        let mut p = Player::new("Alice", 100);
        p.set_status(PlayerStatus::Active);
        p.place_bet(100); // all-in
        p.new_hand();
        assert_eq!(p.status(), PlayerStatus::Eliminated);
    }

    #[test]
    fn test_reset_round_bet() {
        let mut p = Player::new("Alice", 1000);
        p.set_status(PlayerStatus::Active);
        p.place_bet(200);
        p.reset_round_bet();
        assert_eq!(p.current_bet(), 0);
        assert_eq!(p.total_bet(), 200); // total not reset
    }
}
