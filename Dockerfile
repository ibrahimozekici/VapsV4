# Build stage
FROM rust:1.86-slim AS builder

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    cmake \
    pkg-config \
    libssl-dev \
    librdkafka-dev \
    gcc \
    g++ \
    make

WORKDIR /app
COPY . .

RUN cargo build --release

# Minimal runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && apt-get clean

COPY --from=builder /app/target/release/chirpstack /usr/bin/chirpstack

USER nobody

ENTRYPOINT ["/usr/bin/chirpstack"]

