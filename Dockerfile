# ── Stage 1: Build ──────────────────────────────────────────────────────
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

RUN cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openalgo-mcp /usr/local/bin/openalgo-mcp

# Environment variables (set at runtime)
ENV OPENALGO_API_KEY=""
ENV OPENALGO_URL="http://host.docker.internal:5000"

# MCP servers use stdio transport
ENTRYPOINT ["openalgo-mcp"]
