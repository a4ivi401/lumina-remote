FROM rust:1.75-slim AS builder

WORKDIR /usr/src/app
# Copy the workspace manifests
COPY Cargo.toml ./
COPY server/lumina-signal-server/Cargo.toml server/lumina-signal-server/

# We only need the signal server, so we copy its source
COPY server/lumina-signal-server/src server/lumina-signal-server/src

# Build the signal server in release mode
RUN cargo build --release --manifest-path server/lumina-signal-server/Cargo.toml

# Production Image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/lumina-signal-server /app/lumina-signal-server

EXPOSE 3000

CMD ["./lumina-signal-server"]
