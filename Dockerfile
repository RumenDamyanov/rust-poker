# Stage 1: Build
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Cache dependencies by building a dummy project first
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'pub mod engine; pub mod api; pub mod config; pub mod rules;' > src/lib.rs && \
    mkdir -p src/engine src/api src/rules && \
    touch src/engine/mod.rs src/api/mod.rs src/rules/mod.rs src/config.rs && \
    cargo build --release 2>/dev/null || true && \
    rm -rf src

# Copy real source and build
COPY src/ src/
COPY tests/ tests/
# Touch main.rs to force rebuild (not just deps)
RUN touch src/main.rs src/lib.rs && cargo build --release

# Stage 2: Runtime (scratch — minimal image)
FROM scratch

COPY --from=builder /app/target/release/rust-poker /rust-poker

ENV PORT=8084
EXPOSE 8084

# Health check using the built-in --health-check flag (no curl needed in scratch)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/rust-poker", "--health-check"]

ENTRYPOINT ["/rust-poker"]
