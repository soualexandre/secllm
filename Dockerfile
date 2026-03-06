# SecLLM – multi-stage build: Rust build + slim runtime
# Single full build (no dummy main) to avoid corrupt rmeta between lapin/amq-protocol-uri and url.

# ---- Build ----
FROM rust:latest AS builder

WORKDIR /app

# Copy all source; one build avoids metadata mismatch (url/amq-protocol-uri)
COPY Cargo.toml Cargo.lock ./
COPY config ./config
COPY src ./src
RUN cargo build --release

# ---- Runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/secllm /app/secllm
COPY config /app/config

EXPOSE 3000

ENTRYPOINT ["/app/secllm"]
