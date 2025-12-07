FROM rust:1.91-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin payment-engine-cli

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

COPY --from=builder /usr/src/app/target/release/payment-engine-cli /usr/local/bin/payment-engine

RUN mkdir -p /data && chown appuser:appuser /data

USER appuser

WORKDIR /data

ENTRYPOINT ["payment-engine"]
CMD ["--help"]