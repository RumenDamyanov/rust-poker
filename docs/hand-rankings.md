# Poker Hand Rankings

Hands are ranked from strongest to weakest. In the engine, `HandRank` uses Rust's derived `Ord` trait with variants listed weakest-to-strongest, so higher variants win.

## Rankings (Best to Worst)

| Rank | Hand | Description | Example |
|------|------|-------------|---------|
| 1 | **Royal Flush** | A, K, Q, J, 10 of the same suit | A♠ K♠ Q♠ J♠ 10♠ |
| 2 | **Straight Flush** | Five consecutive cards of the same suit | 9♥ 8♥ 7♥ 6♥ 5♥ |
| 3 | **Four of a Kind** | Four cards of the same rank | K♣ K♠ K♥ K♦ 3♠ |
| 4 | **Full House** | Three of a kind + a pair | Q♣ Q♠ Q♥ 8♦ 8♣ |
| 5 | **Flush** | Five cards of the same suit (not sequential) | A♦ J♦ 8♦ 6♦ 3♦ |
| 6 | **Straight** | Five consecutive cards (mixed suits) | 10♣ 9♦ 8♥ 7♠ 6♣ |
| 7 | **Three of a Kind** | Three cards of the same rank | 7♣ 7♠ 7♥ K♦ 2♣ |
| 8 | **Two Pair** | Two different pairs | J♣ J♠ 4♥ 4♦ A♣ |
| 9 | **One Pair** | Two cards of the same rank | 10♣ 10♠ A♥ 8♦ 5♣ |
| 10 | **High Card** | No combination — highest card wins | A♣ J♦ 8♥ 5♠ 3♣ |

## Tiebreaking

When two or more players have the same hand type, ties are broken by:

1. **Primary rank**: The rank that defines the hand (e.g., the pair, the three-of-a-kind)
2. **Kickers**: Remaining cards compared from highest to lowest

### Examples

- **Pair of Kings** vs **Pair of Queens**: Kings win (higher pair)
- **Pair of Kings (A kicker)** vs **Pair of Kings (J kicker)**: Ace kicker wins
- **Two Pair (K-K, 8-8)** vs **Two Pair (K-K, 7-7)**: Eights win (higher second pair)

## The Wheel (A-2-3-4-5)

The Ace can be used as a low card to form the lowest straight (A-2-3-4-5), known as "the wheel." This is the lowest possible straight.

## Low Hands (Omaha Hi-Lo)

In Omaha Hi-Lo, a qualifying low hand must have:
- Five cards ranked **8 or lower**
- **No pairs** among those five cards
- Aces count as low (1)

The best possible low hand is A-2-3-4-5 ("the nut low").

Low hands are compared from the highest card down. For example:
- 8-6-4-3-A beats 8-7-3-2-A (6 < 7 in the second position)
