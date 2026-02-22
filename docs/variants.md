# Poker Variants

rust-poker supports five major poker variants, all included from v0.1.0.

---

## Texas Hold'em

The most popular poker variant worldwide.

| Property | Value |
|----------|-------|
| Hole cards | 2 |
| Community cards | 5 (3+1+1) |
| Betting structure | No Limit |
| Players | 2–10 |
| Blinds | Yes |

### Flow

1. **Pre-flop**: Each player receives 2 hole cards, betting round
2. **Flop**: 3 community cards dealt, betting round
3. **Turn**: 1 community card dealt, betting round
4. **River**: 1 community card dealt, betting round
5. **Showdown**: Best 5-card hand from any combination of 7 available cards

---

## Omaha

Similar to Hold'em but with 4 hole cards and stricter hand rules.

| Property | Value |
|----------|-------|
| Hole cards | 4 |
| Community cards | 5 (3+1+1) |
| Betting structure | Pot Limit |
| Players | 2–9 |
| Blinds | Yes |

### Key Rule

Players **must** use exactly 2 of their 4 hole cards and exactly 3 community cards to form their hand. This differs from Hold'em where any combination is allowed.

### Flow

Same as Texas Hold'em (PreFlop → Flop → Turn → River → Showdown).

---

## Omaha Hi-Lo (8 or Better)

A split-pot variant where the pot is divided between the best high hand and the best qualifying low hand.

| Property | Value |
|----------|-------|
| Hole cards | 4 |
| Community cards | 5 (3+1+1) |
| Betting structure | Pot Limit |
| Players | 2–9 |
| Blinds | Yes |
| Split pot | Yes (Hi-Lo) |

### Key Rules

- Same "exactly 2 hole + 3 community" rule as Omaha
- Half the pot goes to the best high hand
- Half goes to the best qualifying low hand (5 cards, all 8 or lower, no pairs)
- If no qualifying low hand exists, the high hand wins the entire pot
- A player can win both halves ("scooping")

---

## Five-Card Draw

The classic draw poker variant.

| Property | Value |
|----------|-------|
| Hole cards | 5 |
| Community cards | 0 |
| Betting structure | Fixed Limit |
| Players | 2–6 |
| Blinds | Yes |
| Draw phase | Yes (discard up to 3) |

### Flow

1. **Deal**: Each player receives 5 hole cards
2. **First betting round**
3. **Draw**: Each player may discard up to 3 cards and receive replacements
4. **Second betting round**
5. **Showdown**: Best 5-card hand wins

---

## Seven-Card Stud

A stud poker variant with no community cards.

| Property | Value |
|----------|-------|
| Cards dealt | 7 per player |
| Community cards | 0 |
| Betting structure | Fixed Limit |
| Players | 2–8 |
| Blinds | No (uses antes/bring-in) |
| Mixed visibility | Yes (some cards face-up) |

### Flow

1. **Third Street**: 2 cards face-down + 1 face-up, betting round
2. **Fourth Street**: 1 card face-up, betting round
3. **Fifth Street**: 1 card face-up, betting round
4. **Sixth Street**: 1 card face-up, betting round
5. **Seventh Street**: 1 card face-down, betting round
6. **Showdown**: Best 5 cards from 7 wins

### Note

The current implementation simplifies stud by dealing all 7 cards at once. A future version will add per-street dealing with mixed visibility.
