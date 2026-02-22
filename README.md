# rust-poker 🃏

A multi-variant poker game engine and REST API server written in Rust.

[![CI](https://github.com/RumenDamyanov/rust-poker/actions/workflows/ci.yml/badge.svg)](https://github.com/RumenDamyanov/rust-poker/actions/workflows/ci.yml)
[![CodeQL](https://github.com/RumenDamyanov/rust-poker/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/RumenDamyanov/rust-poker/actions/workflows/github-code-scanning/codeql)
[![Dependabot](https://github.com/RumenDamyanov/rust-poker/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/RumenDamyanov/rust-poker/actions/workflows/dependabot/dependabot-updates)
[![codecov](https://codecov.io/gh/RumenDamyanov/rust-poker/graph/badge.svg)](https://codecov.io/gh/RumenDamyanov/rust-poker)
[![crates.io](https://img.shields.io/crates/v/rumenx-poker.svg)](https://crates.io/crates/rumenx-poker)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

## Overview

**rust-poker** implements a multi-variant poker game engine as a backend with a RESTful API. It supports Texas Hold'em, Omaha, Omaha Hi-Lo, Five-Card Draw, and Seven-Card Stud — all in a single crate. Designed to be consumed by frontend applications (web, mobile, CLI) and serves as a demo/educational project showcasing Rust and its ecosystem.

Part of a family of poker implementations across languages (Rust, Go, TypeScript) sharing the same API contract.

### Features

- 🃏 **5 poker variants** — Texas Hold'em, Omaha, Omaha Hi-Lo, Five-Card Draw, Seven-Card Stud
- 🎯 **Pure game engine** — no I/O, fully testable
- 🌐 **REST API** with Axum
- 🏆 **Hand evaluator** — evaluates any combination of cards, supports Hi-Lo
- 💰 **Side pot management** — correct pot distribution for all-in scenarios
- 📜 **Full action history** tracking
- 🐳 **Docker support** (< 15 MB image)
- ✅ **Comprehensive tests** (unit + integration)

## Supported Variants

| Variant | Hole | Community | Betting | Players |
|---------|------|-----------|---------|---------|
| Texas Hold'em | 2 | 5 | No Limit | 2–10 |
| Omaha | 4 | 5 | Pot Limit | 2–9 |
| Omaha Hi-Lo | 4 | 5 | Pot Limit | 2–9 |
| Five-Card Draw | 5 | 0 | Fixed Limit | 2–6 |
| Seven-Card Stud | 7 | 0 | Fixed Limit | 2–8 |

See [docs/variants.md](docs/variants.md) for detailed rules.

## Quick Start

### Run the Server

```bash
cargo run
# Server starts on http://localhost:8084
```

### Play a Game

```bash
# Create a Texas Hold'em game
curl -s -X POST http://localhost:8084/api/games \
  -H 'Content-Type: application/json' \
  -d '{"variant": "texas_holdem"}' | jq

# Join players
curl -s -X POST http://localhost:8084/api/games/{id}/join \
  -H 'Content-Type: application/json' \
  -d '{"name": "Alice", "chips": 1000}' | jq

curl -s -X POST http://localhost:8084/api/games/{id}/join \
  -H 'Content-Type: application/json' \
  -d '{"name": "Bob", "chips": 1000}' | jq

# Start the hand
curl -s -X POST http://localhost:8084/api/games/{id}/start | jq

# Submit an action
curl -s -X POST http://localhost:8084/api/games/{id}/action \
  -H 'Content-Type: application/json' \
  -d '{"player_id": "...", "action": "call"}' | jq
```

### Use as a Library

```rust
use rumenx_poker::engine::game::Game;
use rumenx_poker::engine::types::PlayerAction;
use rumenx_poker::rules::texas_holdem;

let ruleset = texas_holdem::ruleset();
let mut game = Game::new(ruleset, 10, 20);

game.join("Alice", 1000).expect("should join");
game.join("Bob", 1000).expect("should join");
game.start_hand().expect("should deal");

println!("Phase: {}", game.phase());
```

## API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/rulesets` | List poker variants |
| `POST` | `/api/games` | Create new game |
| `GET` | `/api/games` | List active games |
| `GET` | `/api/games/{id}` | Get game state |
| `DELETE` | `/api/games/{id}` | Delete game |
| `POST` | `/api/games/{id}/join` | Join a game |
| `POST` | `/api/games/{id}/leave` | Leave a game |
| `POST` | `/api/games/{id}/start` | Start a hand |
| `POST` | `/api/games/{id}/action` | Player action (fold/check/call/raise/all-in/discard) |
| `GET` | `/api/games/{id}/player/{pid}` | Player-specific view |
| `GET` | `/api/games/{id}/history` | Action history |

See [docs/api-reference.md](docs/api-reference.md) for full details.

## Documentation

| Document | Description |
|----------|-------------|
| [Hand Rankings](docs/hand-rankings.md) | Complete hand ranking chart with tiebreakers |
| [Variants](docs/variants.md) | All 5 poker variants and their rules |
| [Betting](docs/betting.md) | Betting structures and round mechanics |
| [API Reference](docs/api-reference.md) | Full REST API documentation |
| [Glossary](docs/glossary.md) | Poker terminology reference |

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# All checks (format + lint + test)
make check

# Coverage report
make coverage

# Run example
cargo run --example basic_game
```

## Docker

```bash
# Build image
docker build -t rust-poker .

# Run container
docker run -p 8084:8084 --rm rust-poker

# Check image size (target: < 15 MB)
docker images rust-poker
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8084` | Server port |
| `HOST` | `0.0.0.0` | Bind address |
| `RUST_LOG` | `rumenx_poker=info` | Log level |

## Project Structure

```
src/
├── main.rs            # Server entry point
├── lib.rs             # Library root
├── config.rs          # Environment configuration
├── engine/            # Pure game logic (no I/O)
│   ├── types.rs       # Card, Rank, Suit, HandRank, GamePhase, etc.
│   ├── deck.rs        # 52-card deck with shuffle
│   ├── evaluator.rs   # Hand evaluation (5-card, best-of-N, Omaha, Hi-Lo)
│   ├── player.rs      # Player state and chip management
│   ├── pot.rs         # Main pot and side pot management
│   ├── betting.rs     # Betting round logic with structure validation
│   └── game.rs        # Game state machine
├── rules/             # Variant definitions
│   ├── mod.rs         # RuleSet struct, EvalMode, evaluate_hand
│   ├── texas_holdem.rs
│   ├── omaha.rs
│   ├── omaha_hilo.rs
│   ├── five_card_draw.rs
│   └── seven_card_stud.rs
└── api/               # REST API layer
    ├── router.rs      # Route definitions
    ├── handlers.rs    # Endpoint handlers
    ├── models.rs      # JSON DTOs
    ├── state.rs       # Shared state
    └── errors.rs      # Error types
```

## Other Poker Implementations

This project is part of a multi-language poker family. Other implementations are planned:

| Language | Repository | Status |
|----------|-----------|--------|
| **Rust** | [rust-poker](https://github.com/RumenDamyanov/rust-poker) | ✅ v0.1.0 |
| **Go** | go-poker | 🔜 Planned |
| **TypeScript** | npm-poker | 🔜 Planned |

All implementations share the same REST API contract for interoperability.

## Other Rust Game Projects

Check out the other Rust game engines in this family:

| Project | Description | Links |
|---------|-------------|-------|
| **rust-blackjack** | Blackjack engine & API | [GitHub](https://github.com/RumenDamyanov/rust-blackjack) · [crates.io](https://crates.io/crates/rumenx-blackjack) |
| **rust-chess** | Chess engine with AI | [GitHub](https://github.com/RumenDamyanov/rust-chess) · [crates.io](https://crates.io/crates/rumenx-chess) |
| **rust-sixtysix** | Sixty-Six card game engine | [GitHub](https://github.com/RumenDamyanov/rust-sixtysix) · [crates.io](https://crates.io/crates/rumenx-sixtysix) |

## License

[MIT](LICENSE.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.
