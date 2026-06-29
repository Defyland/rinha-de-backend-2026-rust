FROM rust:1.95-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY resources ./resources
COPY openapi.yaml ./openapi.yaml

RUN cargo build --release

FROM debian:bookworm-slim

RUN useradd --system --uid 10001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/rinha-de-backend-2026-rust /usr/local/bin/rinha-de-backend-2026-rust
COPY resources ./resources

ENV BIND_ADDR=0.0.0.0:9999

EXPOSE 9999

USER appuser

CMD ["rinha-de-backend-2026-rust"]
