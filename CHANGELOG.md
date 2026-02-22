# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-21

### Added
- Core poker engine with 5 supported variants:
  - **Texas Hold'em** — 2 hole cards, 5 community, no-limit
  - **Omaha** — 4 hole cards, pot-limit, must use exactly 2+3
  - **Omaha Hi-Lo** — split pot variant with qualifying low hands
  - **Five-Card Draw** — 5 hole cards, draw phase, fixed-limit
  - **Seven-Card Stud** — 7 dealt cards, no community, fixed-limit
- Card types: Rank (2–A), Suit (♥♦♣♠), Card struct
- Deck: standard 52-card deck with shuffle and draw
- Hand evaluator: evaluate any 5-card hand, find best from N cards
  - Omaha-specific evaluation (exactly 2 hole + 3 community)
  - Hi-Lo evaluation with qualifying low hand detection
- Player state management with chip tracking
- Pot management with side pot calculation for all-in scenarios
- Betting round logic with validation for No Limit, Pot Limit, and Fixed Limit
- Game state machine: deal → betting → community → showdown
- Draw phase support for Five-Card Draw
- RuleSet data structs for each variant (serializable, not traits)
- REST API with Axum:
  - `GET /health` — health check
  - `GET /api/rulesets` — list all poker variants
  - `POST /api/games` — create new game
  - `GET /api/games` — list active games
  - `GET /api/games/{id}` — get game state
  - `DELETE /api/games/{id}` — delete game
  - `POST /api/games/{id}/join` — join a game
  - `POST /api/games/{id}/leave` — leave a game
  - `POST /api/games/{id}/start` — start a hand
  - `POST /api/games/{id}/action` — submit player action
  - `GET /api/games/{id}/player/{player_id}` — player-specific view
  - `GET /api/games/{id}/history` — action history
- CORS support
- Structured JSON error responses
- Comprehensive unit tests (engine, evaluator, betting) and integration tests (API)
- Docker support (multi-stage build, minimal image)
- CI pipeline (GitHub Actions): format, lint, test, coverage, Docker
- Publish workflow for crates.io
- Documentation: hand rankings, variant rules, betting structures, API reference, glossary
