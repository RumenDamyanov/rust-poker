//! Poker game state machine.
//!
//! Orchestrates the full lifecycle of a poker hand: dealing, betting rounds,
//! community cards, draw phase, showdown, and pot distribution.

use super::betting::BettingRound;
use super::deck::Deck;
use super::player::Player;
use super::pot::PotManager;
use super::types::{
    ActionRecord, Card, GameError, GamePhase, HandRank, PlayerAction, PlayerStatus,
};
use crate::rules::{self, RuleSet};

/// Result of a showdown — who won and what they had.
#[derive(Debug, Clone)]
pub struct ShowdownResult {
    /// Player ID of the winner.
    pub player_id: String,
    /// Player name.
    pub player_name: String,
    /// The winning hand rank.
    pub hand_rank: HandRank,
    /// The 5 cards forming the winning hand.
    pub best_cards: Vec<Card>,
    /// Chips won.
    pub chips_won: u32,
}

/// A poker game managing a table of players and hand lifecycle.
#[derive(Debug, Clone)]
pub struct Game {
    /// Unique game ID.
    id: String,
    /// Active ruleset.
    ruleset: RuleSet,
    /// Players at the table.
    players: Vec<Player>,
    /// Current deck.
    deck: Deck,
    /// Community cards on the table.
    community_cards: Vec<Card>,
    /// Current game phase.
    phase: GamePhase,
    /// Pot manager.
    pot: PotManager,
    /// Current betting round state.
    betting: BettingRound,
    /// Dealer button position (index into players).
    dealer_pos: usize,
    /// Index of current player to act.
    current_player: usize,
    /// Small blind amount.
    small_blind: u32,
    /// Big blind amount.
    big_blind: u32,
    /// Hand number (increments each deal).
    hand_number: u32,
    /// Action history for the current hand.
    history: Vec<ActionRecord>,
    /// Results from the last showdown.
    last_showdown: Vec<ShowdownResult>,
    /// Whether the draw phase was completed (Five-Card Draw).
    draw_complete: bool,
}

impl Game {
    /// Create a new poker game with the given ruleset and blind amounts.
    pub fn new(ruleset: RuleSet, small_blind: u32, big_blind: u32) -> Self {
        let betting = BettingRound::new(big_blind, ruleset.betting_structure, false);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ruleset,
            players: Vec::new(),
            deck: Deck::new(),
            community_cards: Vec::new(),
            phase: GamePhase::Waiting,
            pot: PotManager::new(),
            betting,
            dealer_pos: 0,
            current_player: 0,
            small_blind,
            big_blind,
            hand_number: 0,
            history: Vec::new(),
            last_showdown: Vec::new(),
            draw_complete: false,
        }
    }

    // -- Getters ---------------------------------------------------------

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn ruleset(&self) -> &RuleSet {
        &self.ruleset
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn community_cards(&self) -> &[Card] {
        &self.community_cards
    }

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn pot_total(&self) -> u32 {
        self.pot.total()
    }

    pub fn pot_manager(&self) -> &PotManager {
        &self.pot
    }

    pub fn dealer_position(&self) -> usize {
        self.dealer_pos
    }

    pub fn current_player_index(&self) -> usize {
        self.current_player
    }

    pub fn current_player_id(&self) -> Option<&str> {
        self.players.get(self.current_player).map(|p| p.id())
    }

    pub fn hand_number(&self) -> u32 {
        self.hand_number
    }

    pub fn history(&self) -> &[ActionRecord] {
        &self.history
    }

    pub fn last_showdown(&self) -> &[ShowdownResult] {
        &self.last_showdown
    }

    pub fn small_blind(&self) -> u32 {
        self.small_blind
    }

    pub fn big_blind(&self) -> u32 {
        self.big_blind
    }

    /// Get available actions for the current player.
    pub fn available_actions(&self) -> Vec<String> {
        if !self.phase.is_betting_phase() && self.phase != GamePhase::Draw {
            return Vec::new();
        }

        if self.phase == GamePhase::Draw {
            return vec!["discard".to_string()];
        }

        if let Some(player) = self.players.get(self.current_player) {
            self.betting.available_actions(player, self.pot.total())
        } else {
            Vec::new()
        }
    }

    /// Get the number of active (non-folded, non-eliminated) players.
    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|p| p.is_active()).count()
    }

    // -- Player management -----------------------------------------------

    /// Add a player to the table.
    pub fn join(&mut self, name: impl Into<String>, chips: u32) -> Result<String, GameError> {
        if self.players.len() >= self.ruleset.max_players {
            return Err(GameError::TableFull {
                max: self.ruleset.max_players,
            });
        }
        if self.phase != GamePhase::Waiting && self.phase != GamePhase::Ready {
            return Err(GameError::InvalidAction(
                "cannot join during an active hand".into(),
            ));
        }

        let player = Player::new(name, chips);
        let id = player.id().to_string();
        self.players.push(player);

        if self.players.len() >= self.ruleset.min_players {
            self.phase = GamePhase::Ready;
        }

        Ok(id)
    }

    /// Remove a player from the table.
    pub fn leave(&mut self, player_id: &str) -> Result<(), GameError> {
        let pos = self
            .players
            .iter()
            .position(|p| p.id() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound(player_id.to_string()))?;

        self.players.remove(pos);

        // Adjust dealer position if needed.
        if !self.players.is_empty() && self.dealer_pos >= self.players.len() {
            self.dealer_pos = 0;
        }

        if self.players.len() < self.ruleset.min_players {
            self.phase = GamePhase::Waiting;
        }

        Ok(())
    }

    /// Find a player by ID.
    pub fn find_player(&self, player_id: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.id() == player_id)
    }

    // -- Hand lifecycle --------------------------------------------------

    /// Start a new hand: shuffle, post blinds, deal hole cards.
    pub fn start_hand(&mut self) -> Result<(), GameError> {
        let player_count = self.players.len();
        if player_count < self.ruleset.min_players {
            return Err(GameError::NotEnoughPlayers {
                needed: self.ruleset.min_players,
                have: player_count,
            });
        }

        // Reset state for new hand.
        self.hand_number += 1;
        self.community_cards.clear();
        self.pot.reset();
        self.history.clear();
        self.last_showdown.clear();
        self.draw_complete = false;

        // Reset players.
        for player in &mut self.players {
            player.new_hand();
        }

        // Remove eliminated players.
        self.players
            .retain(|p| p.status() != PlayerStatus::Eliminated);
        if self.players.len() < self.ruleset.min_players {
            return Err(GameError::NotEnoughPlayers {
                needed: self.ruleset.min_players,
                have: self.players.len(),
            });
        }

        // Rotate dealer.
        if self.hand_number > 1 {
            self.dealer_pos = (self.dealer_pos + 1) % self.players.len();
        }

        // Shuffle deck.
        self.deck = Deck::new_shuffled();

        // Post blinds.
        if self.ruleset.uses_blinds {
            self.post_blinds();
        }

        // Deal hole cards.
        let hole_count = self.ruleset.hole_cards;
        for player in &mut self.players {
            let cards = self.deck.draw_many(hole_count)?;
            player.deal_cards(cards);
        }

        // Set phase to first betting round.
        self.phase = self.ruleset.betting_rounds[0];
        self.betting = BettingRound::new_preflop(self.big_blind, self.ruleset.betting_structure);

        // First to act: UTG (left of big blind) for pre-flop.
        self.current_player = self.first_to_act_preflop();

        Ok(())
    }

    /// Process a player action (fold, check, call, raise, all-in, discard).
    pub fn player_action(
        &mut self,
        player_id: &str,
        action: PlayerAction,
    ) -> Result<(), GameError> {
        // Verify it's this player's turn.
        let player_idx = self
            .players
            .iter()
            .position(|p| p.id() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound(player_id.to_string()))?;

        if player_idx != self.current_player {
            return Err(GameError::NotYourTurn);
        }

        // Handle draw phase separately.
        if self.phase == GamePhase::Draw {
            return self.handle_draw(player_idx, &action);
        }

        if !self.phase.is_betting_phase() {
            return Err(GameError::WrongPhase {
                expected: "betting phase".to_string(),
                actual: self.phase.to_string(),
            });
        }

        // Record the action.
        self.history.push(ActionRecord {
            player_id: player_id.to_string(),
            action: action.clone(),
            phase: self.phase,
        });

        // Process the betting action.
        let pot_total = self.pot.total();
        self.betting.process_action(
            &mut self.players[player_idx],
            &action,
            player_idx,
            pot_total,
        )?;

        // Check if only one player remains (everyone else folded).
        let active_count = self.players.iter().filter(|p| p.is_active()).count();

        if active_count <= 1 {
            self.collect_bets();
            self.award_last_standing();
            return Ok(());
        }

        // Check if all remaining players are all-in (no more actions possible).
        let can_act_count = self.players.iter().filter(|p| p.can_act()).count();

        if can_act_count <= 1 && self.betting.is_complete(&self.players) {
            // Deal remaining community cards and go to showdown.
            self.collect_bets();
            self.deal_remaining_community()?;
            self.showdown()?;
            return Ok(());
        }

        // Move to next player.
        self.advance_player();

        // Check if betting round is complete.
        if self.betting.is_complete(&self.players) {
            self.collect_bets();
            self.advance_phase()?;
        }

        Ok(())
    }

    // -- Internal methods ------------------------------------------------

    fn post_blinds(&mut self) {
        let player_count = self.players.len();
        let sb_pos = if player_count == 2 {
            self.dealer_pos // Heads-up: dealer posts SB
        } else {
            (self.dealer_pos + 1) % player_count
        };
        let bb_pos = (sb_pos + 1) % player_count;

        // Post small blind.
        let sb_actual = self.players[sb_pos].place_bet(self.small_blind);
        self.pot.add(sb_actual);

        // Post big blind.
        let bb_actual = self.players[bb_pos].place_bet(self.big_blind);
        self.pot.add(bb_actual);
    }

    fn first_to_act_preflop(&self) -> usize {
        let player_count = self.players.len();
        if player_count == 2 {
            // Heads-up: dealer (SB) acts first pre-flop.
            self.dealer_pos
        } else {
            // UTG: left of big blind.
            let bb_pos = (self.dealer_pos + 2) % player_count;
            (bb_pos + 1) % player_count
        }
    }

    fn first_to_act_postflop(&self) -> usize {
        let player_count = self.players.len();
        // First active player left of the dealer.
        for i in 1..=player_count {
            let pos = (self.dealer_pos + i) % player_count;
            if self.players[pos].can_act() || self.players[pos].status() == PlayerStatus::AllIn {
                return pos;
            }
        }
        0
    }

    fn advance_player(&mut self) {
        let player_count = self.players.len();
        for i in 1..=player_count {
            let next = (self.current_player + i) % player_count;
            if self.players[next].can_act() {
                self.current_player = next;
                return;
            }
        }
    }

    fn advance_phase(&mut self) -> Result<(), GameError> {
        // Reset round bets for all players.
        for player in &mut self.players {
            player.reset_round_bet();
        }

        let current_phase_idx = self
            .ruleset
            .betting_rounds
            .iter()
            .position(|p| *p == self.phase);

        let next_phase_idx = current_phase_idx.map(|i| i + 1);

        // Check if there's a draw phase next.
        if self.ruleset.has_draw && !self.draw_complete && self.phase == GamePhase::PreFlop {
            self.phase = GamePhase::Draw;
            self.current_player = self.first_to_act_postflop();
            return Ok(());
        }

        // Deal community cards for the next phase (if applicable).
        if let Some(idx) = next_phase_idx
            && idx < self.ruleset.betting_rounds.len()
        {
            let next_phase = self.ruleset.betting_rounds[idx];

            // Deal community cards scheduled for this phase.
            for (phase, count) in &self.ruleset.community_schedule {
                if *phase == next_phase {
                    let cards = self.deck.draw_many(*count)?;
                    self.community_cards.extend(cards);
                }
            }

            self.phase = next_phase;
            let is_late = matches!(next_phase, GamePhase::Turn | GamePhase::River);
            self.betting.reset(is_late);
            self.current_player = self.first_to_act_postflop();

            return Ok(());
        }

        // No more betting rounds — go to showdown.
        self.showdown()?;
        Ok(())
    }

    fn collect_bets(&mut self) {
        let player_bets: Vec<(String, u32)> = self
            .players
            .iter()
            .filter(|p| p.total_bet() > 0)
            .map(|p| (p.id().to_string(), p.total_bet()))
            .collect();

        if !player_bets.is_empty() {
            self.pot.build_from_bets(player_bets);
        }
    }

    fn award_last_standing(&mut self) {
        let winner = self.players.iter_mut().find(|p| p.is_active());

        if let Some(winner) = winner {
            let total = self.pot.total();
            winner.award_chips(total);
            self.last_showdown.push(ShowdownResult {
                player_id: winner.id().to_string(),
                player_name: winner.name().to_string(),
                hand_rank: HandRank::HighCard([
                    super::types::Rank::Ace,
                    super::types::Rank::King,
                    super::types::Rank::Queen,
                    super::types::Rank::Jack,
                    super::types::Rank::Ten,
                ]),
                best_cards: Vec::new(),
                chips_won: total,
            });
        }

        self.phase = GamePhase::HandComplete;
    }

    fn deal_remaining_community(&mut self) -> Result<(), GameError> {
        let dealt = self.community_cards.len();
        let total = self.ruleset.community_cards;
        if dealt < total {
            let remaining = total - dealt;
            let cards = self.deck.draw_many(remaining)?;
            self.community_cards.extend(cards);
        }
        Ok(())
    }

    fn showdown(&mut self) -> Result<(), GameError> {
        self.phase = GamePhase::Showdown;

        let active_players: Vec<(usize, String, String)> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_active())
            .map(|(i, p)| (i, p.id().to_string(), p.name().to_string()))
            .collect();

        // Evaluate hands for each active player.
        let mut player_hands: Vec<(usize, String, String, HandRank, Vec<Card>)> = Vec::new();

        for (idx, pid, pname) in &active_players {
            let hole = self.players[*idx].hole_cards().to_vec();
            let (rank, cards) =
                rules::evaluate_hand(self.ruleset.eval_mode, &hole, &self.community_cards);
            player_hands.push((*idx, pid.clone(), pname.clone(), rank, cards));
        }

        // Sort by hand rank (best first).
        player_hands.sort_by(|a, b| b.3.cmp(&a.3));

        // Distribute pots.
        let pots = self.pot.pots().to_vec();
        let mut results: Vec<ShowdownResult> = Vec::new();

        for pot in &pots {
            // Find the best hand among eligible players.
            let eligible_hands: Vec<_> = player_hands
                .iter()
                .filter(|(_, pid, _, _, _)| pot.eligible.contains(pid))
                .collect();

            if let Some(winner) = eligible_hands.first() {
                // Check for split (tied hands).
                let tied: Vec<_> = eligible_hands
                    .iter()
                    .filter(|(_, _, _, rank, _)| *rank == winner.3)
                    .collect();

                let share = pot.amount / tied.len() as u32;
                let remainder = pot.amount % tied.len() as u32;

                for (i, (idx, pid, pname, rank, cards)) in tied.iter().enumerate() {
                    let extra = if i == 0 { remainder } else { 0 };
                    let won = share + extra;
                    self.players[*idx].award_chips(won);
                    results.push(ShowdownResult {
                        player_id: pid.clone(),
                        player_name: pname.clone(),
                        hand_rank: rank.clone(),
                        best_cards: cards.clone(),
                        chips_won: won,
                    });
                }
            }
        }

        self.last_showdown = results;
        self.phase = GamePhase::HandComplete;

        Ok(())
    }

    fn handle_draw(&mut self, player_idx: usize, action: &PlayerAction) -> Result<(), GameError> {
        match action {
            PlayerAction::Discard(indices) => {
                // Validate discard count.
                if indices.len() > self.ruleset.max_discard {
                    return Err(GameError::InvalidDiscard(format!(
                        "can discard at most {} cards, tried {}",
                        self.ruleset.max_discard,
                        indices.len()
                    )));
                }

                // Validate indices.
                let hole_count = self.players[player_idx].hole_cards().len();
                for &idx in indices {
                    if idx >= hole_count {
                        return Err(GameError::InvalidDiscard(format!(
                            "card index {} out of range (have {} cards)",
                            idx, hole_count
                        )));
                    }
                }

                // Draw replacement cards.
                let new_cards = self.deck.draw_many(indices.len())?;
                self.players[player_idx].replace_cards(indices, &new_cards);

                // Record action.
                self.history.push(ActionRecord {
                    player_id: self.players[player_idx].id().to_string(),
                    action: action.clone(),
                    phase: self.phase,
                });

                // Check if all active players have drawn.
                self.advance_player();

                // If we've gone around all players, move to next betting round.
                let all_drawn = self.check_all_players_acted_draw();
                if all_drawn {
                    self.draw_complete = true;
                    // Move to the post-draw betting round.
                    self.phase = GamePhase::PreFlop; // reuse PreFlop as second betting round
                    self.betting.reset(true); // late street = doubled bets
                    self.current_player = self.first_to_act_postflop();
                    // advance_phase will handle the rest on next action
                    // Actually, set to the next phase in betting_rounds
                    if self.ruleset.betting_rounds.len() > 1 {
                        self.phase = self.ruleset.betting_rounds[1];
                    }
                    self.betting.reset(true);
                    self.current_player = self.first_to_act_postflop();
                }

                Ok(())
            }
            _ => Err(GameError::InvalidAction(
                "only discard is allowed during draw phase".into(),
            )),
        }
    }

    fn check_all_players_acted_draw(&self) -> bool {
        // Simple heuristic: count draw actions vs active players.
        let active_count = self.players.iter().filter(|p| p.can_act()).count();
        let draw_actions = self
            .history
            .iter()
            .filter(|a| a.phase == GamePhase::Draw)
            .count();
        draw_actions >= active_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::texas_holdem;

    fn setup_holdem_game() -> Game {
        let ruleset = texas_holdem::ruleset();
        let mut game = Game::new(ruleset, 10, 20);
        game.join("Alice", 1000).unwrap();
        game.join("Bob", 1000).unwrap();
        game
    }

    #[test]
    fn test_create_game() {
        let game = setup_holdem_game();
        assert_eq!(game.players().len(), 2);
        assert_eq!(game.phase(), GamePhase::Ready);
        assert_eq!(game.ruleset().name, "texas_holdem");
    }

    #[test]
    fn test_join_players() {
        let ruleset = texas_holdem::ruleset();
        let mut game = Game::new(ruleset, 10, 20);

        let id = game.join("Alice", 1000).unwrap();
        assert!(!id.is_empty());
        assert_eq!(game.phase(), GamePhase::Waiting); // need 2

        game.join("Bob", 1000).unwrap();
        assert_eq!(game.phase(), GamePhase::Ready); // 2 = min
    }

    #[test]
    fn test_table_full() {
        let ruleset = texas_holdem::ruleset();
        let mut game = Game::new(ruleset, 10, 20);
        for i in 0..10 {
            game.join(format!("P{i}"), 1000).unwrap();
        }
        let result = game.join("Extra", 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_hand() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        assert_eq!(game.phase(), GamePhase::PreFlop);
        assert_eq!(game.hand_number(), 1);
        // Each player should have 2 hole cards.
        for player in game.players() {
            assert_eq!(player.hole_cards().len(), 2);
        }
    }

    #[test]
    fn test_not_enough_players() {
        let ruleset = texas_holdem::ruleset();
        let mut game = Game::new(ruleset, 10, 20);
        game.join("Alice", 1000).unwrap();
        let result = game.start_hand();
        assert!(result.is_err());
    }

    #[test]
    fn test_fold_wins() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        let current_id = game.current_player_id().unwrap().to_string();
        game.player_action(&current_id, PlayerAction::Fold).unwrap();

        assert_eq!(game.phase(), GamePhase::HandComplete);
    }

    #[test]
    fn test_check_check_advances() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        // Pre-flop: first player calls (matches big blind).
        let p1_id = game.current_player_id().unwrap().to_string();
        game.player_action(&p1_id, PlayerAction::Call).unwrap();

        // Big blind checks.
        let p2_id = game.current_player_id().unwrap().to_string();
        game.player_action(&p2_id, PlayerAction::Check).unwrap();

        // Should advance to flop.
        assert_eq!(game.phase(), GamePhase::Flop);
        assert_eq!(game.community_cards().len(), 3);
    }

    #[test]
    fn test_full_hand_to_showdown() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        // Pre-flop.
        let p1 = game.current_player_id().unwrap().to_string();
        game.player_action(&p1, PlayerAction::Call).unwrap();
        let p2 = game.current_player_id().unwrap().to_string();
        game.player_action(&p2, PlayerAction::Check).unwrap();

        // Flop.
        assert_eq!(game.phase(), GamePhase::Flop);
        let p1 = game.current_player_id().unwrap().to_string();
        game.player_action(&p1, PlayerAction::Check).unwrap();
        let p2 = game.current_player_id().unwrap().to_string();
        game.player_action(&p2, PlayerAction::Check).unwrap();

        // Turn.
        assert_eq!(game.phase(), GamePhase::Turn);
        let p1 = game.current_player_id().unwrap().to_string();
        game.player_action(&p1, PlayerAction::Check).unwrap();
        let p2 = game.current_player_id().unwrap().to_string();
        game.player_action(&p2, PlayerAction::Check).unwrap();

        // River.
        assert_eq!(game.phase(), GamePhase::River);
        let p1 = game.current_player_id().unwrap().to_string();
        game.player_action(&p1, PlayerAction::Check).unwrap();
        let p2 = game.current_player_id().unwrap().to_string();
        game.player_action(&p2, PlayerAction::Check).unwrap();

        // Should reach hand complete (showdown resolves immediately).
        assert_eq!(game.phase(), GamePhase::HandComplete);
        assert!(!game.last_showdown().is_empty());
    }

    #[test]
    fn test_not_your_turn() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        // Find the player who is NOT current.
        let current = game.current_player_id().unwrap().to_string();
        let other = game
            .players()
            .iter()
            .find(|p| p.id() != current)
            .unwrap()
            .id()
            .to_string();

        let result = game.player_action(&other, PlayerAction::Fold);
        assert!(result.is_err());
    }

    #[test]
    fn test_leave_player() {
        let mut game = setup_holdem_game();
        let id = game.players()[0].id().to_string();
        game.leave(&id).unwrap();
        assert_eq!(game.players().len(), 1);
        assert_eq!(game.phase(), GamePhase::Waiting);
    }

    #[test]
    fn test_all_in_vs_all_in() {
        let mut game = setup_holdem_game();
        game.start_hand().unwrap();

        let p1 = game.current_player_id().unwrap().to_string();
        game.player_action(&p1, PlayerAction::AllIn).unwrap();

        let p2 = game.current_player_id().unwrap().to_string();
        game.player_action(&p2, PlayerAction::AllIn).unwrap();

        // Should complete immediately (deal remaining community + showdown).
        assert_eq!(game.phase(), GamePhase::HandComplete);
        assert_eq!(game.community_cards().len(), 5);
    }
}
