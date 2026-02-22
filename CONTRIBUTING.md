# Contributing to rust-poker

Thank you for your interest in contributing to rust-poker! We welcome contributions from the community.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. **Fork the repository** and clone it locally
2. **Install Rust**: [rustup.rs](https://rustup.rs/) (stable, edition 2024)
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes** following our coding standards
5. **Test your changes**: `cargo test`
6. **Submit a pull request**

### Development Setup

```bash
# Clone
git clone https://github.com/RumenDamyanov/rust-poker.git
cd rust-poker

# Build
cargo build

# Run tests
cargo test

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Run all checks
make check

# Run the server
cargo run
```

## Pull Request Process

1. Ensure all tests pass: `cargo test`
2. Ensure code is formatted: `cargo fmt --all -- --check`
3. Ensure no lint warnings: `cargo clippy --all-targets --all-features -- -D warnings`
4. Update documentation if you changed public APIs
5. Add tests for new functionality
6. Update CHANGELOG.md with your changes

## Coding Standards

- Follow `cargo fmt` formatting
- No clippy warnings (`-D warnings`)
- All public items must have `///` doc comments
- Use `thiserror` for error types
- No `unsafe` code
- No `unwrap()` in library code — use `?` or `expect("reason")`
- Keep engine logic pure (no I/O, no async)
- Write descriptive test names: `test_full_house_beats_flush`

## Architecture

The codebase follows a layered architecture:

```
Types ← Rules ← Engine ← API ← main
```

- **engine/types.rs** — All core types (Card, Rank, Suit, HandRank, etc.)
- **engine/** — Pure game logic (deck, evaluator, player, pot, betting, game)
- **rules/** — Variant definitions (RuleSet structs, evaluation dispatch)
- **api/** — REST layer (handlers, router, models, state, errors)

No reverse imports — lower layers never import from higher ones.

## Commit Message Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Omaha Hi-Lo variant
fix: correct straight flush detection
docs: update API reference
test: add integration tests for player actions
chore(deps): bump axum to 0.8
```

## Questions?

Open an issue or reach out at **contact@rumenx.com**.
