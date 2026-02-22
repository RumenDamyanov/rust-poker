# Betting Structures

rust-poker supports three betting structures used across the different variants.

---

## No Limit

Used by: **Texas Hold'em**

- A player may bet or raise **any amount** up to their entire chip stack
- Minimum raise must be at least the size of the previous raise
- No maximum bet (other than the player's stack)

### Example

Blinds: 10/20. Player A raises to 60 (a raise of 40 over the big blind). Player B must raise at least 40 more (to 100 minimum).

---

## Pot Limit

Used by: **Omaha**, **Omaha Hi-Lo**

- Maximum raise is the current pot size
- Pot is calculated as: current pot + all bets on the table + the amount needed to call
- Minimum raise follows the same rules as No Limit

### Example

Pot is 100. Player A bets 50. Player B wants to raise: pot is now 100 + 50 = 150, plus the 50 to call = 200. So max raise is 200 total (a raise of 150 over Player A's bet).

---

## Fixed Limit

Used by: **Five-Card Draw**, **Seven-Card Stud**

- Bets and raises are in fixed increments
- Early streets (pre-flop, flop): small bet = big blind
- Late streets (turn, river): big bet = 2× big blind
- Typically limited to 4 raises per round (not strictly enforced in v1)

### Example

Blinds: 10/20 (fixed limit). On the flop, bets must be exactly 20. On the turn and river, bets must be exactly 40.

---

## Betting Round Flow

Each betting round proceeds clockwise from the first player to act:

1. Each player chooses one action: **Fold**, **Check**, **Call**, **Raise**, or **All-In**
2. A player may **check** only if no bet is outstanding
3. A player may **call** to match the current bet
4. A **raise** increases the current bet (must meet minimum raise requirements)
5. **All-In** puts all remaining chips in the pot
6. The round ends when all active players have matched the highest bet

### Blinds

- **Small blind**: Forced bet by the player to the left of the dealer
- **Big blind**: Forced bet by the player two seats left of the dealer (2× small blind)
- In heads-up play: the dealer posts the small blind and acts first pre-flop

### Heads-Up Rules

In a 2-player game:
- The dealer posts the small blind and acts first pre-flop
- Post-flop, the non-dealer acts first
