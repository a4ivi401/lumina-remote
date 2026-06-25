FROM rust:slim AS builder

WORKDIR /usr/src/app
# Copy the entire workspace to satisfy Cargo.toml workspace members
COPY . .

# Build the signal server in release mode
RUN cargo build --release -p lumina-signal-server

# Production Image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/lumina-signal-server /app/lumina-signal-server

EXPOSE 3000

CMD ["./lumina-signal-server"]
