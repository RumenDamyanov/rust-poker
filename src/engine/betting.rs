//! Betting round logic for poker.
//!
//! Validates player actions, tracks bets, and determines when a round completes.

use super::player::Player;
use super::types::{BettingStructure, GameError, PlayerAction, PlayerStatus};

/// State for a single betting round.
#[derive(Debug, Clone)]
pub struct BettingRound {
    /// The highest bet placed this round.
    current_bet: u32,
    /// Minimum raise increment.
    min_raise: u32,
    /// The big blind amount (base unit for betting).
    big_blind: u32,
    /// Betting structure (no-limit, pot-limit, fixed-limit).
    structure: BettingStructure,
    /// Index of the last player who raised (or opened).
    last_raiser: Option<usize>,
    /// Count of players who have acted this round.
    actions_taken: usize,
    /// Whether the round is for a "late street" (turn/river in fixed-limit).
    is_late_street: bool,
}

impl BettingRound {
    /// Create a new betting round.
    pub fn new(big_blind: u32, structure: BettingStructure, is_late_street: bool) -> Self {
        Self {
            current_bet: 0,
            min_raise: big_blind,
            big_blind,
            structure,
            last_raiser: None,
            actions_taken: 0,
            is_late_street,
        }
    }

    /// Create a pre-flop betting round (big blind already posted).
    pub fn new_preflop(big_blind: u32, structure: BettingStructure) -> Self {
        Self {
            current_bet: big_blind,
            min_raise: big_blind,
            big_blind,
            structure,
            last_raiser: None,
            actions_taken: 0,
            is_late_street: false,
        }
    }

    /// Current highest bet.
    pub fn current_bet(&self) -> u32 {
        self.current_bet
    }

    /// Minimum raise amount.
    pub fn min_raise(&self) -> u32 {
        self.min_raise
    }

    /// Validate and process a player action.
    ///
    /// Returns the chip amount the player needs to put in (0 for fold/check).
    pub fn process_action(
        &mut self,
        player: &mut Player,
        action: &PlayerAction,
        player_index: usize,
        pot_total: u32,
    ) -> Result<u32, GameError> {
        if !player.can_act() {
            return Err(GameError::InvalidAction(format!(
                "player {} cannot act (status: {})",
                player.id(),
                player.status()
            )));
        }

        let chips_to_call = self.current_bet.saturating_sub(player.current_bet());

        let amount = match action {
            PlayerAction::Fold => {
                player.fold();
                self.actions_taken += 1;
                return Ok(0);
            }

            PlayerAction::Check => {
                if chips_to_call > 0 {
                    return Err(GameError::InvalidAction(format!(
                        "cannot check: must call {} or fold",
                        chips_to_call
                    )));
                }
                self.actions_taken += 1;
                return Ok(0);
            }

            PlayerAction::Call => {
                if chips_to_call == 0 {
                    return Err(GameError::InvalidAction(
                        "nothing to call — use check instead".into(),
                    ));
                }
                let actual = player.place_bet(chips_to_call);
                self.actions_taken += 1;
                actual
            }

            PlayerAction::Raise(raise_amount) => {
                let total_bet = self.current_bet + raise_amount;
                self.validate_raise(*raise_amount, player.chips(), chips_to_call, pot_total)?;

                let to_put_in = total_bet - player.current_bet();
                let actual = player.place_bet(to_put_in);

                self.min_raise = *raise_amount;
                self.current_bet = player.current_bet();
                self.last_raiser = Some(player_index);
                self.actions_taken += 1;
                actual
            }

            PlayerAction::AllIn => {
                let actual = player.place_bet(player.chips());
                if player.current_bet() > self.current_bet {
                    // This all-in counts as a raise.
                    let raise_by = player.current_bet() - self.current_bet;
                    if raise_by >= self.min_raise {
                        self.min_raise = raise_by;
                    }
                    self.current_bet = player.current_bet();
                    self.last_raiser = Some(player_index);
                }
                self.actions_taken += 1;
                actual
            }

            PlayerAction::Discard(_) => {
                return Err(GameError::InvalidAction(
                    "discard is not a betting action".into(),
                ));
            }
        };

        Ok(amount)
    }

    /// Check if the betting round is complete.
    ///
    /// A round is complete when all active players have acted at least once
    /// and all have matched the current bet (or are all-in/folded).
    pub fn is_complete(&self, players: &[Player]) -> bool {
        let active_players: Vec<_> = players
            .iter()
            .filter(|p| p.status() == PlayerStatus::Active)
            .collect();

        // If no active players (all folded or all-in), round is done.
        if active_players.is_empty() {
            return true;
        }

        // All active players must have acted.
        if self.actions_taken < active_players.len() {
            return false;
        }

        // All active players must have matched the current bet.
        active_players
            .iter()
            .all(|p| p.current_bet() == self.current_bet)
    }

    /// Get available actions for a player.
    pub fn available_actions(&self, player: &Player, pot_total: u32) -> Vec<String> {
        if !player.can_act() {
            return Vec::new();
        }

        let chips_to_call = self.current_bet.saturating_sub(player.current_bet());
        let mut actions = Vec::new();

        // Fold is always available.
        actions.push("fold".to_string());

        if chips_to_call == 0 {
            actions.push("check".to_string());
        } else {
            actions.push("call".to_string());
        }

        // Can raise if they have enough chips.
        if player.chips() > chips_to_call {
            let max_raise = self.max_raise(player.chips(), chips_to_call, pot_total);
            if max_raise >= self.min_raise {
                actions.push("raise".to_string());
            }
        }

        // All-in is always available if player has chips.
        if player.chips() > 0 {
            actions.push("all_in".to_string());
        }

        actions
    }

    /// Reset for a new betting round within the same hand.
    pub fn reset(&mut self, is_late_street: bool) {
        self.current_bet = 0;
        self.min_raise = if is_late_street && self.structure == BettingStructure::FixedLimit {
            self.big_blind * 2
        } else {
            self.big_blind
        };
        self.last_raiser = None;
        self.actions_taken = 0;
        self.is_late_street = is_late_street;
    }

    // -- Internal validation ---------------------------------------------

    fn validate_raise(
        &self,
        raise_amount: u32,
        player_chips: u32,
        chips_to_call: u32,
        pot_total: u32,
    ) -> Result<(), GameError> {
        // Must have enough chips for call + raise.
        if chips_to_call + raise_amount > player_chips {
            return Err(GameError::InvalidBet(format!(
                "not enough chips: need {}, have {}",
                chips_to_call + raise_amount,
                player_chips
            )));
        }

        match self.structure {
            BettingStructure::NoLimit => {
                if raise_amount < self.min_raise {
                    return Err(GameError::InvalidBet(format!(
                        "minimum raise is {}, got {}",
                        self.min_raise, raise_amount
                    )));
                }
            }
            BettingStructure::PotLimit => {
                let max_raise = pot_total + chips_to_call + self.current_bet;
                if raise_amount < self.min_raise {
                    return Err(GameError::InvalidBet(format!(
                        "minimum raise is {}, got {}",
                        self.min_raise, raise_amount
                    )));
                }
                if raise_amount > max_raise {
                    return Err(GameError::InvalidBet(format!(
                        "pot-limit max raise is {}, got {}",
                        max_raise, raise_amount
                    )));
                }
            }
            BettingStructure::FixedLimit => {
                let fixed_amount = if self.is_late_street {
                    self.big_blind * 2
                } else {
                    self.big_blind
                };
                if raise_amount != fixed_amount {
                    return Err(GameError::InvalidBet(format!(
                        "fixed-limit raise must be {}, got {}",
                        fixed_amount, raise_amount
                    )));
                }
            }
        }

        Ok(())
    }

    fn max_raise(&self, player_chips: u32, chips_to_call: u32, pot_total: u32) -> u32 {
        let available = player_chips.saturating_sub(chips_to_call);
        match self.structure {
            BettingStructure::NoLimit => available,
            BettingStructure::PotLimit => {
                let pot_max = pot_total + chips_to_call + self.current_bet;
                available.min(pot_max)
            }
            BettingStructure::FixedLimit => {
                if self.is_late_street {
                    self.big_blind * 2
                } else {
                    self.big_blind
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player(name: &str, chips: u32) -> Player {
        let mut p = Player::with_id(name, name, chips);
        p.set_status(PlayerStatus::Active);
        p
    }

    #[test]
    fn test_fold() {
        let mut round = BettingRound::new(20, BettingStructure::NoLimit, false);
        let mut player = make_player("a", 1000);
        let result = round.process_action(&mut player, &PlayerAction::Fold, 0, 0);
        assert!(result.is_ok());
        assert_eq!(player.status(), PlayerStatus::Folded);
    }

    #[test]
    fn test_check_when_no_bet() {
        let mut round = BettingRound::new(20, BettingStructure::NoLimit, false);
        let mut player = make_player("a", 1000);
        let result = round.process_action(&mut player, &PlayerAction::Check, 0, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_fails_when_bet_outstanding() {
        let mut round = BettingRound::new_preflop(20, BettingStructure::NoLimit);
        let mut player = make_player("a", 1000);
        let result = round.process_action(&mut player, &PlayerAction::Check, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_call() {
        let mut round = BettingRound::new_preflop(20, BettingStructure::NoLimit);
        let mut player = make_player("a", 1000);
        let result = round.process_action(&mut player, &PlayerAction::Call, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20);
        assert_eq!(player.chips(), 980);
    }

    #[test]
    fn test_raise_no_limit() {
        let mut round = BettingRound::new_preflop(20, BettingStructure::NoLimit);
        let mut player = make_player("a", 1000);
        // Raise by 40 (total bet = 20 + 40 = 60)
        let result = round.process_action(&mut player, &PlayerAction::Raise(40), 0, 0);
        assert!(result.is_ok());
        assert_eq!(player.current_bet(), 60);
        assert_eq!(player.chips(), 940);
        assert_eq!(round.current_bet(), 60);
    }

    #[test]
    fn test_raise_below_minimum() {
        let mut round = BettingRound::new_preflop(20, BettingStructure::NoLimit);
        let mut player = make_player("a", 1000);
        // Min raise is 20 (big blind), try 10
        let result = round.process_action(&mut player, &PlayerAction::Raise(10), 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_in() {
        let mut round = BettingRound::new(20, BettingStructure::NoLimit, false);
        let mut player = make_player("a", 100);
        let result = round.process_action(&mut player, &PlayerAction::AllIn, 0, 0);
        assert!(result.is_ok());
        assert_eq!(player.chips(), 0);
        assert_eq!(player.status(), PlayerStatus::AllIn);
    }

    #[test]
    fn test_round_complete() {
        let round = BettingRound::new(20, BettingStructure::NoLimit, false);
        let p1 = make_player("a", 1000);
        let p2 = make_player("b", 1000);
        // No actions yet — not complete.
        assert!(!round.is_complete(&[p1, p2]));
    }

    #[test]
    fn test_fixed_limit_raise() {
        let mut round = BettingRound::new(20, BettingStructure::FixedLimit, false);
        let mut player = make_player("a", 1000);
        // Fixed limit: must raise exactly big_blind (20).
        let result = round.process_action(&mut player, &PlayerAction::Raise(20), 0, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_limit_wrong_amount() {
        let mut round = BettingRound::new(20, BettingStructure::FixedLimit, false);
        let mut player = make_player("a", 1000);
        let result = round.process_action(&mut player, &PlayerAction::Raise(30), 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_available_actions_no_bet() {
        let round = BettingRound::new(20, BettingStructure::NoLimit, false);
        let player = make_player("a", 1000);
        let actions = round.available_actions(&player, 0);
        assert!(actions.contains(&"fold".to_string()));
        assert!(actions.contains(&"check".to_string()));
        assert!(actions.contains(&"raise".to_string()));
        assert!(!actions.contains(&"call".to_string()));
    }

    #[test]
    fn test_available_actions_with_bet() {
        let round = BettingRound::new_preflop(20, BettingStructure::NoLimit);
        let player = make_player("a", 1000);
        let actions = round.available_actions(&player, 20);
        assert!(actions.contains(&"fold".to_string()));
        assert!(actions.contains(&"call".to_string()));
        assert!(actions.contains(&"raise".to_string()));
        assert!(!actions.contains(&"check".to_string()));
    }
}
