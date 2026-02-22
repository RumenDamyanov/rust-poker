//! Basic programmatic usage of the Poker engine.
//!
//! Run with: `cargo run --example basic_game`

use rumenx_poker::engine::game::Game;
use rumenx_poker::engine::types::PlayerAction;
use rumenx_poker::rules::texas_holdem;

fn main() {
    println!("=== Poker Engine Demo (Texas Hold'em) ===\n");

    let ruleset = texas_holdem::ruleset();
    let mut game = Game::new(ruleset, 10, 20);

    // Add players.
    let alice_id = game.join("Alice", 1000).expect("Alice should join");
    let bob_id = game.join("Bob", 1000).expect("Bob should join");

    println!("Game ID: {}", game.id());
    println!("Players: Alice ({}), Bob ({})", alice_id, bob_id);
    println!("Blinds: {}/{}", game.small_blind(), game.big_blind());
    println!();

    // Deal.
    game.start_hand().expect("should deal");
    println!("--- Hand #{} ---", game.hand_number());
    println!("Phase: {}", game.phase());

    for player in game.players() {
        println!(
            "  {} ({}) has {} cards, {} chips",
            player.name(),
            player.id(),
            player.hole_cards().len(),
            player.chips(),
        );
    }
    println!();

    // Pre-flop: first player calls, second player checks.
    let current = game.current_player_id().unwrap().to_string();
    println!("{} calls", player_name(&game, &current));
    game.player_action(&current, PlayerAction::Call)
        .expect("should call");

    let current = game.current_player_id().unwrap().to_string();
    println!("{} checks", player_name(&game, &current));
    game.player_action(&current, PlayerAction::Check)
        .expect("should check");

    // Flop.
    println!("\n--- Flop ---");
    println!("Community: {}", format_cards(game.community_cards()));
    println!("Phase: {}", game.phase());
    println!("Pot: {}\n", game.pot_total());

    // Both check through remaining streets.
    for street in &["Flop", "Turn", "River"] {
        println!("--- {street} betting ---");
        for _ in 0..2 {
            if game.phase().to_string() == "HandComplete" {
                break;
            }
            let current = game.current_player_id().unwrap().to_string();
            println!("  {} checks", player_name(&game, &current));
            game.player_action(&current, PlayerAction::Check)
                .expect("should check");
        }
        println!("Community: {}", format_cards(game.community_cards()));
        println!("Phase: {}", game.phase());
        println!();
    }

    // Show results.
    println!("=== Showdown ===");
    for result in game.last_showdown() {
        println!(
            "  {} won {} chips with {}",
            result.player_name, result.chips_won, result.hand_rank,
        );
    }

    println!("\nFinal chip counts:");
    for player in game.players() {
        println!("  {}: {} chips", player.name(), player.chips());
    }
}

fn player_name(game: &Game, id: &str) -> String {
    game.find_player(id)
        .map(|p| p.name().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_cards(cards: &[rumenx_poker::engine::types::Card]) -> String {
    if cards.is_empty() {
        return "(none)".to_string();
    }
    cards
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
