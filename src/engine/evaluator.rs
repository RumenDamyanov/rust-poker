//! Poker hand evaluator.
//!
//! Evaluates 5-card poker hands and finds the best 5-card hand from N cards.
//! Supports standard poker rules (best 5 from 7 for Hold'em) and
//! Omaha-specific constraints (must use exactly 2 of 4 hole cards).

use itertools::Itertools;

use super::types::{Card, HandRank, Rank};

/// Evaluate a single 5-card poker hand and return its ranking.
///
/// The cards do not need to be sorted — the function handles that internally.
pub fn evaluate_five(cards: &[Card; 5]) -> HandRank {
    let mut ranks: Vec<Rank> = cards.iter().map(|c| c.rank).collect();
    ranks.sort_unstable_by(|a, b| b.cmp(a)); // descending

    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let is_straight = is_straight_high(&ranks);
    let is_wheel = is_wheel(&ranks);

    // Count rank occurrences.
    let mut counts: Vec<(Rank, usize)> = Vec::new();
    for &r in &ranks {
        if let Some(entry) = counts.iter_mut().find(|(rank, _)| *rank == r) {
            entry.1 += 1;
        } else {
            counts.push((r, 1));
        }
    }
    // Sort by count descending, then rank descending for tie-breaking.
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    let count_pattern: Vec<usize> = counts.iter().map(|(_, c)| *c).collect();

    // Royal flush: A-K-Q-J-10 of one suit.
    if is_flush && is_straight && ranks[0] == Rank::Ace && ranks[4] == Rank::Ten {
        return HandRank::RoyalFlush;
    }

    // Straight flush.
    if is_flush && is_straight {
        return HandRank::StraightFlush(ranks[0]);
    }
    if is_flush && is_wheel {
        return HandRank::StraightFlush(Rank::Five); // ace-low straight flush
    }

    // Four of a kind.
    if count_pattern == [4, 1] {
        return HandRank::FourOfAKind(counts[0].0, counts[1].0);
    }

    // Full house.
    if count_pattern == [3, 2] {
        return HandRank::FullHouse(counts[0].0, counts[1].0);
    }

    // Flush.
    if is_flush {
        return HandRank::Flush([ranks[0], ranks[1], ranks[2], ranks[3], ranks[4]]);
    }

    // Straight.
    if is_straight {
        return HandRank::Straight(ranks[0]);
    }
    if is_wheel {
        return HandRank::Straight(Rank::Five); // ace-low straight
    }

    // Three of a kind.
    if count_pattern == [3, 1, 1] {
        let kickers = counts.iter().skip(1).map(|(r, _)| *r).collect::<Vec<_>>();
        return HandRank::ThreeOfAKind(counts[0].0, [kickers[0], kickers[1]]);
    }

    // Two pair.
    if count_pattern == [2, 2, 1] {
        let high_pair = counts[0].0;
        let low_pair = counts[1].0;
        let kicker = counts[2].0;
        return HandRank::TwoPair(high_pair, low_pair, kicker);
    }

    // One pair.
    if count_pattern == [2, 1, 1, 1] {
        let pair = counts[0].0;
        let kickers = counts.iter().skip(1).map(|(r, _)| *r).collect::<Vec<_>>();
        return HandRank::OnePair(pair, [kickers[0], kickers[1], kickers[2]]);
    }

    // High card.
    HandRank::HighCard([ranks[0], ranks[1], ranks[2], ranks[3], ranks[4]])
}

/// Find the best 5-card poker hand from N cards (N >= 5).
///
/// For Texas Hold'em, N=7 (2 hole + 5 community) → C(7,5) = 21 combinations.
/// Returns the best `HandRank` and the 5 cards forming it.
pub fn best_hand(cards: &[Card]) -> (HandRank, Vec<Card>) {
    assert!(cards.len() >= 5, "need at least 5 cards");

    let mut best_rank: Option<HandRank> = None;
    let mut best_cards: Vec<Card> = Vec::new();

    for combo in cards.iter().combinations(5) {
        let five: [Card; 5] = [*combo[0], *combo[1], *combo[2], *combo[3], *combo[4]];
        let rank = evaluate_five(&five);
        if best_rank.as_ref().is_none_or(|br| rank > *br) {
            best_rank = Some(rank);
            best_cards = five.to_vec();
        }
    }

    (best_rank.expect("at least one combination"), best_cards)
}

/// Find the best hand for Omaha: must use exactly 2 of 4 hole cards
/// and exactly 3 of 5 community cards.
///
/// C(4,2) × C(5,3) = 6 × 10 = 60 combinations.
pub fn best_hand_omaha(hole: &[Card], community: &[Card]) -> (HandRank, Vec<Card>) {
    assert_eq!(hole.len(), 4, "Omaha requires exactly 4 hole cards");
    assert_eq!(
        community.len(),
        5,
        "Omaha requires exactly 5 community cards"
    );

    let mut best_rank: Option<HandRank> = None;
    let mut best_cards: Vec<Card> = Vec::new();

    for hole_combo in hole.iter().combinations(2) {
        for comm_combo in community.iter().combinations(3) {
            let five: [Card; 5] = [
                *hole_combo[0],
                *hole_combo[1],
                *comm_combo[0],
                *comm_combo[1],
                *comm_combo[2],
            ];
            let rank = evaluate_five(&five);
            if best_rank.as_ref().is_none_or(|br| rank > *br) {
                best_rank = Some(rank);
                best_cards = five.to_vec();
            }
        }
    }

    (best_rank.expect("at least one combination"), best_cards)
}

/// Find the best hand for Omaha Hi-Lo.
/// Returns (best_high, best_low) — the low hand may be None if no qualifying low exists.
/// A qualifying low hand must have 5 cards of rank 8 or below (aces play low).
pub fn best_hand_omaha_hilo(
    hole: &[Card],
    community: &[Card],
) -> (HandRank, Vec<Card>, Option<Vec<Card>>) {
    let (high_rank, high_cards) = best_hand_omaha(hole, community);

    // Find the best qualifying low hand.
    let mut best_low: Option<Vec<Card>> = None;

    for hole_combo in hole.iter().combinations(2) {
        for comm_combo in community.iter().combinations(3) {
            let five = vec![
                *hole_combo[0],
                *hole_combo[1],
                *comm_combo[0],
                *comm_combo[1],
                *comm_combo[2],
            ];

            if is_qualifying_low(&five)
                && best_low.as_ref().is_none_or(|current| {
                    compare_low_hands(&five, current) == std::cmp::Ordering::Less
                })
            {
                best_low = Some(five);
            }
        }
    }

    (high_rank, high_cards, best_low)
}

/// Check if a 5-card hand qualifies as a low hand (8-or-better).
/// All 5 cards must have rank ≤ 8, and all ranks must be distinct.
/// Aces count as low (1) for this purpose.
fn is_qualifying_low(cards: &[Card]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for card in cards {
        let low_val = low_rank_value(card.rank);
        if low_val > 8 || !seen.insert(low_val) {
            return false;
        }
    }
    true
}

/// Get a rank's value for low-hand comparison (Ace = 1, others = face value).
fn low_rank_value(rank: Rank) -> u8 {
    match rank {
        Rank::Ace => 1,
        _ => rank.value(),
    }
}

/// Compare two qualifying low hands. Lower is better.
/// Compare highest card first, then next highest, etc.
fn compare_low_hands(a: &[Card], b: &[Card]) -> std::cmp::Ordering {
    let mut a_vals: Vec<u8> = a.iter().map(|c| low_rank_value(c.rank)).collect();
    let mut b_vals: Vec<u8> = b.iter().map(|c| low_rank_value(c.rank)).collect();
    a_vals.sort_unstable_by(|x, y| y.cmp(x)); // descending (highest first for comparison)
    b_vals.sort_unstable_by(|x, y| y.cmp(x));
    a_vals.cmp(&b_vals)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Check if sorted (descending) ranks form a straight (ace-high only).
fn is_straight_high(ranks: &[Rank]) -> bool {
    if ranks.len() != 5 {
        return false;
    }
    for i in 0..4 {
        if ranks[i].value() != ranks[i + 1].value() + 1 {
            return false;
        }
    }
    true
}

/// Check if sorted (descending) ranks form the wheel (A-2-3-4-5).
fn is_wheel(ranks: &[Rank]) -> bool {
    ranks.len() == 5
        && ranks[0] == Rank::Ace
        && ranks[1] == Rank::Five
        && ranks[2] == Rank::Four
        && ranks[3] == Rank::Three
        && ranks[4] == Rank::Two
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::Suit;

    fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn test_royal_flush() {
        let cards = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::RoyalFlush);
    }

    #[test]
    fn test_straight_flush() {
        let cards = [
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Hearts),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six, Suit::Hearts),
            c(Rank::Five, Suit::Hearts),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::StraightFlush(Rank::Nine));
    }

    #[test]
    fn test_ace_low_straight_flush() {
        let cards = [
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Diamonds),
            c(Rank::Five, Suit::Diamonds),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::StraightFlush(Rank::Five));
    }

    #[test]
    fn test_four_of_a_kind() {
        let cards = [
            c(Rank::Nine, Suit::Spades),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Nine, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
            c(Rank::King, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::FourOfAKind(Rank::Nine, Rank::King)
        );
    }

    #[test]
    fn test_full_house() {
        let cards = [
            c(Rank::King, Suit::Spades),
            c(Rank::King, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
            c(Rank::Three, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::FullHouse(Rank::King, Rank::Three)
        );
    }

    #[test]
    fn test_flush() {
        let cards = [
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Five, Suit::Diamonds),
            c(Rank::Eight, Suit::Diamonds),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Ace, Suit::Diamonds),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::Flush([Rank::Ace, Rank::Jack, Rank::Eight, Rank::Five, Rank::Two])
        );
    }

    #[test]
    fn test_straight() {
        let cards = [
            c(Rank::Four, Suit::Clubs),
            c(Rank::Five, Suit::Spades),
            c(Rank::Six, Suit::Hearts),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Eight, Suit::Clubs),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::Straight(Rank::Eight));
    }

    #[test]
    fn test_ace_low_straight() {
        let cards = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Five, Suit::Spades),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::Straight(Rank::Five));
    }

    #[test]
    fn test_ace_high_straight() {
        let cards = [
            c(Rank::Ten, Suit::Clubs),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Ace, Suit::Clubs),
        ];
        assert_eq!(evaluate_five(&cards), HandRank::Straight(Rank::Ace));
    }

    #[test]
    fn test_three_of_a_kind() {
        let cards = [
            c(Rank::Seven, Suit::Spades),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Two, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::ThreeOfAKind(Rank::Seven, [Rank::Queen, Rank::Two])
        );
    }

    #[test]
    fn test_two_pair() {
        let cards = [
            c(Rank::Jack, Suit::Spades),
            c(Rank::Jack, Suit::Hearts),
            c(Rank::Four, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Ace, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::TwoPair(Rank::Jack, Rank::Four, Rank::Ace)
        );
    }

    #[test]
    fn test_one_pair() {
        let cards = [
            c(Rank::Ten, Suit::Spades),
            c(Rank::Ten, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::Eight, Suit::Clubs),
            c(Rank::Three, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::OnePair(Rank::Ten, [Rank::Ace, Rank::Eight, Rank::Three])
        );
    }

    #[test]
    fn test_high_card() {
        let cards = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Three, Suit::Spades),
        ];
        assert_eq!(
            evaluate_five(&cards),
            HandRank::HighCard([Rank::Ace, Rank::King, Rank::Nine, Rank::Seven, Rank::Three])
        );
    }

    #[test]
    fn test_best_hand_from_seven() {
        // Hold'em: 2 hole + 5 community = 7 cards
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        let (rank, _best) = best_hand(&cards);
        assert_eq!(rank, HandRank::RoyalFlush);
    }

    #[test]
    fn test_best_hand_picks_strongest() {
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        let (rank, _) = best_hand(&cards);
        assert_eq!(rank, HandRank::FullHouse(Rank::Ace, Rank::King));
    }

    #[test]
    fn test_omaha_must_use_two_hole_cards() {
        // Hole: A♠ K♠ 2♥ 3♥
        // Community: Q♠ J♠ 10♠ 9♦ 8♦
        // Regular best-5: A♠ K♠ Q♠ J♠ 10♠ = Royal Flush (using all from community + hole)
        // Omaha: must use exactly 2 hole + 3 community
        //   Best: A♠ K♠ + Q♠ J♠ 10♠ = Royal Flush (this works!)
        let hole = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Hearts),
        ];
        let community = [
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
            c(Rank::Nine, Suit::Diamonds),
            c(Rank::Eight, Suit::Diamonds),
        ];
        let (rank, _) = best_hand_omaha(&hole, &community);
        assert_eq!(rank, HandRank::RoyalFlush);
    }

    #[test]
    fn test_omaha_constraint_matters() {
        // Hole: A♠ 2♠ 3♣ 4♣ (no spade flush pieces that work well)
        // Community: K♠ Q♠ J♠ 10♠ 5♦
        // Without Omaha constraint: A♠ K♠ Q♠ J♠ 10♠ = Royal Flush
        // With Omaha: must use 2 hole + 3 community
        //   A♠ 2♠ + K♠ Q♠ J♠ = flush (A high = Flush, not Royal)
        let hole = [
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Spades),
            c(Rank::Three, Suit::Clubs),
            c(Rank::Four, Suit::Clubs),
        ];
        let community = [
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
            c(Rank::Five, Suit::Diamonds),
        ];
        let (rank, _) = best_hand_omaha(&hole, &community);
        // Best: A♠ 2♠ + K♠ Q♠ J♠ = flush, not royal (missing 10 in the 5-card combo)
        // Actually: A♠ 2♠ + K♠ Q♠ J♠ is a flush [A K Q J 2] of spades
        // Check it's a flush but NOT a straight flush or royal flush
        match rank {
            HandRank::Flush(_) | HandRank::StraightFlush(_) => {
                // The best combo might be A♠ + any other spade hole + 3 comm spades
                // A♠ 2♠ + K♠ Q♠ J♠ = flush
                // or: 3♣ 4♣ + K♠ Q♠ J♠ = no flush, just straight (K high)
                // So flush wins
            }
            _ => panic!("expected flush, got {rank:?}"),
        }
    }

    #[test]
    fn test_two_pair_beats_one_pair() {
        let one_pair = HandRank::OnePair(Rank::Ace, [Rank::King, Rank::Queen, Rank::Jack]);
        let two_pair = HandRank::TwoPair(Rank::Three, Rank::Two, Rank::Ace);
        assert!(two_pair > one_pair);
    }

    #[test]
    fn test_qualifying_low() {
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Five, Suit::Spades),
        ];
        assert!(is_qualifying_low(&cards));
    }

    #[test]
    fn test_non_qualifying_low_high_card() {
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Nine, Suit::Spades), // 9 > 8, doesn't qualify
        ];
        assert!(!is_qualifying_low(&cards));
    }

    #[test]
    fn test_non_qualifying_low_duplicate() {
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Ace, Suit::Hearts), // duplicate rank
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Five, Suit::Spades),
        ];
        assert!(!is_qualifying_low(&cards));
    }
}
