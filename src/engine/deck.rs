//! Standard 52-card deck with shuffle and draw operations.

use rand::seq::SliceRandom;

use super::types::{Card, GameError, Rank, Suit};

/// A deck of playing cards.
#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Create a new standard 52-card deck (unshuffled).
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &Suit::ALL {
            for &rank in &Rank::ALL {
                cards.push(Card::new(rank, suit));
            }
        }
        Self { cards }
    }

    /// Shuffle the deck using a cryptographically-secure RNG.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Create a new shuffled deck.
    pub fn new_shuffled() -> Self {
        let mut deck = Self::new();
        deck.shuffle();
        deck
    }

    /// Draw the top card from the deck.
    pub fn draw(&mut self) -> Result<Card, GameError> {
        self.cards.pop().ok_or(GameError::DeckEmpty)
    }

    /// Draw multiple cards from the deck.
    pub fn draw_many(&mut self, count: usize) -> Result<Vec<Card>, GameError> {
        let mut cards = Vec::with_capacity(count);
        for _ in 0..count {
            cards.push(self.draw()?);
        }
        Ok(cards)
    }

    /// Number of cards remaining in the deck.
    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Whether the deck is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_deck_has_52_cards() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 52);
    }

    #[test]
    fn test_all_cards_unique() {
        let deck = Deck::new();
        let unique: HashSet<_> = deck.cards.iter().collect();
        assert_eq!(unique.len(), 52);
    }

    #[test]
    fn test_draw_reduces_count() {
        let mut deck = Deck::new_shuffled();
        let _ = deck.draw().unwrap();
        assert_eq!(deck.remaining(), 51);
    }

    #[test]
    fn test_draw_many() {
        let mut deck = Deck::new_shuffled();
        let cards = deck.draw_many(5).unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(deck.remaining(), 47);
    }

    #[test]
    fn test_draw_empty_deck() {
        let mut deck = Deck { cards: vec![] };
        assert!(deck.draw().is_err());
    }

    #[test]
    fn test_shuffle_changes_order() {
        let deck1 = Deck::new();
        let deck2 = Deck::new_shuffled();
        // Extremely unlikely (1 / 52!) that order is preserved.
        assert_ne!(deck1.cards, deck2.cards);
    }

    #[test]
    fn test_is_empty() {
        let deck = Deck { cards: vec![] };
        assert!(deck.is_empty());
        let deck = Deck::new();
        assert!(!deck.is_empty());
    }
}
