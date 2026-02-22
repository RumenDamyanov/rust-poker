.PHONY: build run test lint fmt check clean docker docker-run coverage audit help

# Default target
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build in debug mode
	cargo build

release: ## Build in release mode (optimized)
	cargo build --release

run: ## Run the server (debug mode)
	cargo run

run-release: ## Run the server (release mode)
	cargo run --release

test: ## Run all tests
	cargo test

test-verbose: ## Run all tests with output
	cargo test -- --nocapture

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

check: fmt-check lint test ## Run all checks (format, lint, test)

coverage: ## Generate coverage report
	cargo tarpaulin --out html --output-dir coverage/
	@echo "Coverage report: coverage/tarpaulin-report.html"

audit: ## Audit dependencies for vulnerabilities
	cargo audit

clean: ## Clean build artifacts
	cargo clean
	rm -rf coverage/

example: ## Run the basic game example
	cargo run --example basic_game

docker: ## Build Docker image
	docker build -t rust-poker .

docker-run: ## Run Docker container
	docker run -p 8084:8084 --rm rust-poker

docker-size: ## Show Docker image size
	docker images rust-poker --format "{{.Repository}}:{{.Tag}} — {{.Size}}"
