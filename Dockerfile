# SecLLM – multi-stage build: Rust build + slim runtime

# ---- Build ----
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache de dependências: copiar manifestos primeiro
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn dummy() {}" > src/lib.rs
RUN cargo build --release && rm -rf src

# Código fonte e build real
COPY config ./config
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

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
