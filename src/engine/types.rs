//! Core types for the poker engine.
//!
//! Defines cards, hand rankings, game phases, player actions, and errors.
//! These are leaf types with no dependencies on other engine modules.

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Card types
// ---------------------------------------------------------------------------

/// Card rank (2 through Ace).
///
/// Discriminant values enable natural ordering (Ace = 14 high).
/// For ace-low straights (A-2-3-4-5), the evaluator special-cases this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    /// All ranks in ascending order.
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// Numeric value of the rank (2–14).
    pub fn value(self) -> u8 {
        self as u8
    }

    /// Short symbol for display (e.g. "A", "K", "10", "2").
    pub fn symbol(self) -> &'static str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Card suit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    /// All suits.
    pub const ALL: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

    /// Lowercase name for JSON/API.
    pub fn name(self) -> &'static str {
        match self {
            Suit::Hearts => "hearts",
            Suit::Diamonds => "diamonds",
            Suit::Clubs => "clubs",
            Suit::Spades => "spades",
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sym = match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };
        write!(f, "{sym}")
    }
}

/// A playing card with a rank and suit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// Create a new card.
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.symbol(), self.suit)
    }
}

// ---------------------------------------------------------------------------
// Hand rankings
// ---------------------------------------------------------------------------

/// Poker hand ranking from weakest to strongest.
///
/// Each variant carries data for tie-breaking (kickers).
/// Derived `PartialOrd`/`Ord` gives natural poker ordering when
/// variants are listed from weakest (top) to strongest (bottom).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    /// No combination — ranked by highest cards.
    HighCard([Rank; 5]),
    /// Two cards of the same rank + 3 kickers.
    OnePair(Rank, [Rank; 3]),
    /// Two different pairs + 1 kicker.
    TwoPair(Rank, Rank, Rank),
    /// Three cards of the same rank + 2 kickers.
    ThreeOfAKind(Rank, [Rank; 2]),
    /// Five sequential cards (mixed suits). Ranked by highest card.
    Straight(Rank),
    /// Five cards of one suit (not sequential). Ranked by card values.
    Flush([Rank; 5]),
    /// Three of a kind + a pair. Ranked by trips then pair.
    FullHouse(Rank, Rank),
    /// Four cards of the same rank + 1 kicker.
    FourOfAKind(Rank, Rank),
    /// Five sequential cards of one suit. Ranked by highest card.
    StraightFlush(Rank),
    /// A-K-Q-J-10 of one suit. The best possible hand.
    RoyalFlush,
}

impl HandRank {
    /// Display name of this hand rank category.
    pub fn name(&self) -> &'static str {
        match self {
            HandRank::HighCard(_) => "High Card",
            HandRank::OnePair(_, _) => "One Pair",
            HandRank::TwoPair(_, _, _) => "Two Pair",
            HandRank::ThreeOfAKind(_, _) => "Three of a Kind",
            HandRank::Straight(_) => "Straight",
            HandRank::Flush(_) => "Flush",
            HandRank::FullHouse(_, _) => "Full House",
            HandRank::FourOfAKind(_, _) => "Four of a Kind",
            HandRank::StraightFlush(_) => "Straight Flush",
            HandRank::RoyalFlush => "Royal Flush",
        }
    }

    /// Numeric tier (1 = Royal Flush best, 10 = High Card worst).
    pub fn tier(&self) -> u8 {
        match self {
            HandRank::RoyalFlush => 1,
            HandRank::StraightFlush(_) => 2,
            HandRank::FourOfAKind(_, _) => 3,
            HandRank::FullHouse(_, _) => 4,
            HandRank::Flush(_) => 5,
            HandRank::Straight(_) => 6,
            HandRank::ThreeOfAKind(_, _) => 7,
            HandRank::TwoPair(_, _, _) => 8,
            HandRank::OnePair(_, _) => 9,
            HandRank::HighCard(_) => 10,
        }
    }
}

impl fmt::Display for HandRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ---------------------------------------------------------------------------
// Game phase & status
// ---------------------------------------------------------------------------

/// Phase of a poker hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    /// Table created, waiting for players to join.
    Waiting,
    /// Enough players have joined, ready to start a hand.
    Ready,
    /// Hole cards dealt, pre-flop betting round (Texas Hold'em / Omaha).
    PreFlop,
    /// First 3 community cards dealt.
    Flop,
    /// Fourth community card dealt.
    Turn,
    /// Fifth community card dealt.
    River,
    /// Draw phase — players discard and draw (Five-Card Draw only).
    Draw,
    /// All betting complete, reveal hands and determine winners.
    Showdown,
    /// Hand is complete, ready for next hand or game end.
    HandComplete,
}

impl GamePhase {
    /// Whether this phase is a terminal state for the hand.
    pub fn is_terminal(self) -> bool {
        matches!(self, GamePhase::HandComplete)
    }

    /// Whether this phase involves a betting round.
    pub fn is_betting_phase(self) -> bool {
        matches!(
            self,
            GamePhase::PreFlop | GamePhase::Flop | GamePhase::Turn | GamePhase::River
        )
    }
}

impl fmt::Display for GamePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GamePhase::Waiting => "Waiting for players",
            GamePhase::Ready => "Ready to deal",
            GamePhase::PreFlop => "Pre-Flop",
            GamePhase::Flop => "Flop",
            GamePhase::Turn => "Turn",
            GamePhase::River => "River",
            GamePhase::Draw => "Draw",
            GamePhase::Showdown => "Showdown",
            GamePhase::HandComplete => "Hand Complete",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Player status & actions
// ---------------------------------------------------------------------------

/// Status of a player at the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    /// Waiting for the hand to start.
    Waiting,
    /// Active in the current hand.
    Active,
    /// Has folded this hand.
    Folded,
    /// Has gone all-in this hand.
    AllIn,
    /// Eliminated (no chips remaining, for tournament-style).
    Eliminated,
}

impl fmt::Display for PlayerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PlayerStatus::Waiting => "Waiting",
            PlayerStatus::Active => "Active",
            PlayerStatus::Folded => "Folded",
            PlayerStatus::AllIn => "All-In",
            PlayerStatus::Eliminated => "Eliminated",
        };
        write!(f, "{s}")
    }
}

/// A player action during a betting round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerAction {
    /// Surrender the hand, forfeit chips in pot.
    Fold,
    /// Pass without betting (only when no outstanding bet).
    Check,
    /// Match the current highest bet.
    Call,
    /// Increase the current bet by the given amount.
    Raise(u32),
    /// Bet all remaining chips.
    AllIn,
    /// Discard and draw new cards (Five-Card Draw only).
    Discard(Vec<usize>),
}

impl fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerAction::Fold => write!(f, "Fold"),
            PlayerAction::Check => write!(f, "Check"),
            PlayerAction::Call => write!(f, "Call"),
            PlayerAction::Raise(amount) => write!(f, "Raise {amount}"),
            PlayerAction::AllIn => write!(f, "All-In"),
            PlayerAction::Discard(indices) => write!(f, "Discard {indices:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Table position
// ---------------------------------------------------------------------------

/// Named positions at the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Dealer,
    SmallBlind,
    BigBlind,
    UnderTheGun,
    /// Generic position for other seats.
    Seat(usize),
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Dealer => write!(f, "Dealer"),
            Position::SmallBlind => write!(f, "Small Blind"),
            Position::BigBlind => write!(f, "Big Blind"),
            Position::UnderTheGun => write!(f, "Under the Gun"),
            Position::Seat(n) => write!(f, "Seat {n}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Betting structure
// ---------------------------------------------------------------------------

/// Betting structure for a poker variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BettingStructure {
    /// Raise up to all of player's chips.
    NoLimit,
    /// Raise up to current pot size.
    PotLimit,
    /// Fixed increment raises.
    FixedLimit,
}

impl fmt::Display for BettingStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BettingStructure::NoLimit => "No Limit",
            BettingStructure::PotLimit => "Pot Limit",
            BettingStructure::FixedLimit => "Fixed Limit",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during game operations.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// Action not valid in the current state.
    #[error("invalid action: {0}")]
    InvalidAction(String),

    /// Deck has run out of cards.
    #[error("deck is empty")]
    DeckEmpty,

    /// Not enough players to start.
    #[error("not enough players: need {needed}, have {have}")]
    NotEnoughPlayers { needed: usize, have: usize },

    /// Table is full.
    #[error("table is full (max {max} players)")]
    TableFull { max: usize },

    /// Player not found at the table.
    #[error("player not found: {0}")]
    PlayerNotFound(String),

    /// Not this player's turn to act.
    #[error("not your turn")]
    NotYourTurn,

    /// Game/hand not in the expected phase.
    #[error("wrong phase: expected {expected}, got {actual}")]
    WrongPhase { expected: String, actual: String },

    /// Invalid bet amount.
    #[error("invalid bet: {0}")]
    InvalidBet(String),

    /// Invalid discard indices.
    #[error("invalid discard: {0}")]
    InvalidDiscard(String),
}

// ---------------------------------------------------------------------------
// Action log entry
// ---------------------------------------------------------------------------

/// A recorded action in the hand history.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    /// ID of the player who made the action.
    pub player_id: String,
    /// The action taken.
    pub action: PlayerAction,
    /// The game phase when the action occurred.
    pub phase: GamePhase,
}

impl fmt::Display for ActionRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} — {}", self.phase, self.player_id, self.action)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_values() {
        assert_eq!(Rank::Two.value(), 2);
        assert_eq!(Rank::Ten.value(), 10);
        assert_eq!(Rank::Ace.value(), 14);
        assert_eq!(Rank::King.value(), 13);
    }

    #[test]
    fn test_rank_ordering() {
        assert!(Rank::Ace > Rank::King);
        assert!(Rank::King > Rank::Queen);
        assert!(Rank::Two < Rank::Three);
    }

    #[test]
    fn test_rank_symbols() {
        assert_eq!(Rank::Ace.symbol(), "A");
        assert_eq!(Rank::Ten.symbol(), "10");
        assert_eq!(Rank::Two.symbol(), "2");
    }

    #[test]
    fn test_card_display() {
        let card = Card::new(Rank::Ace, Suit::Spades);
        assert_eq!(format!("{card}"), "A♠");
        let card = Card::new(Rank::Ten, Suit::Hearts);
        assert_eq!(format!("{card}"), "10♥");
    }

    #[test]
    fn test_suit_name() {
        assert_eq!(Suit::Hearts.name(), "hearts");
        assert_eq!(Suit::Spades.name(), "spades");
    }

    #[test]
    fn test_hand_rank_ordering() {
        let high_card =
            HandRank::HighCard([Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]);
        let pair = HandRank::OnePair(Rank::Ace, [Rank::King, Rank::Queen, Rank::Jack]);
        let two_pair = HandRank::TwoPair(Rank::Ace, Rank::King, Rank::Queen);
        let trips = HandRank::ThreeOfAKind(Rank::Ace, [Rank::King, Rank::Queen]);
        let straight = HandRank::Straight(Rank::Ace);
        let flush = HandRank::Flush([Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]);
        let full_house = HandRank::FullHouse(Rank::Ace, Rank::King);
        let quads = HandRank::FourOfAKind(Rank::Ace, Rank::King);
        let straight_flush = HandRank::StraightFlush(Rank::King);
        let royal = HandRank::RoyalFlush;

        assert!(high_card < pair);
        assert!(pair < two_pair);
        assert!(two_pair < trips);
        assert!(trips < straight);
        assert!(straight < flush);
        assert!(flush < full_house);
        assert!(full_house < quads);
        assert!(quads < straight_flush);
        assert!(straight_flush < royal);
    }

    #[test]
    fn test_hand_rank_tiebreak_pair() {
        let pair_aces = HandRank::OnePair(Rank::Ace, [Rank::King, Rank::Queen, Rank::Jack]);
        let pair_kings = HandRank::OnePair(Rank::King, [Rank::Ace, Rank::Queen, Rank::Jack]);
        assert!(pair_aces > pair_kings);
    }

    #[test]
    fn test_hand_rank_tiebreak_kicker() {
        let pair_k_high = HandRank::OnePair(Rank::Ace, [Rank::King, Rank::Queen, Rank::Jack]);
        let pair_q_high = HandRank::OnePair(Rank::Ace, [Rank::Queen, Rank::Jack, Rank::Ten]);
        assert!(pair_k_high > pair_q_high);
    }

    #[test]
    fn test_game_phase_terminal() {
        assert!(!GamePhase::PreFlop.is_terminal());
        assert!(!GamePhase::Showdown.is_terminal());
        assert!(GamePhase::HandComplete.is_terminal());
    }

    #[test]
    fn test_game_phase_betting() {
        assert!(GamePhase::PreFlop.is_betting_phase());
        assert!(GamePhase::Flop.is_betting_phase());
        assert!(GamePhase::Turn.is_betting_phase());
        assert!(GamePhase::River.is_betting_phase());
        assert!(!GamePhase::Waiting.is_betting_phase());
        assert!(!GamePhase::Showdown.is_betting_phase());
    }

    #[test]
    fn test_player_action_display() {
        assert_eq!(format!("{}", PlayerAction::Fold), "Fold");
        assert_eq!(format!("{}", PlayerAction::Raise(100)), "Raise 100");
        assert_eq!(format!("{}", PlayerAction::AllIn), "All-In");
    }

    #[test]
    fn test_betting_structure_display() {
        assert_eq!(format!("{}", BettingStructure::NoLimit), "No Limit");
        assert_eq!(format!("{}", BettingStructure::PotLimit), "Pot Limit");
        assert_eq!(format!("{}", BettingStructure::FixedLimit), "Fixed Limit");
    }

    #[test]
    fn test_hand_rank_name() {
        assert_eq!(HandRank::RoyalFlush.name(), "Royal Flush");
        assert_eq!(
            HandRank::HighCard([Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]).name(),
            "High Card"
        );
    }

    #[test]
    fn test_hand_rank_tier() {
        assert_eq!(HandRank::RoyalFlush.tier(), 1);
        assert_eq!(
            HandRank::HighCard([Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]).tier(),
            10
        );
    }

    #[test]
    fn test_all_ranks() {
        assert_eq!(Rank::ALL.len(), 13);
        assert_eq!(Rank::ALL[0], Rank::Two);
        assert_eq!(Rank::ALL[12], Rank::Ace);
    }

    #[test]
    fn test_all_suits() {
        assert_eq!(Suit::ALL.len(), 4);
    }
}
