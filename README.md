# LuminaRemote

LuminaRemote is an ultra-fast, lightweight, and secure remote access application optimized for unstable networks and local connections.

## Key Features
- **Zero-Trust Security**: No passwords required. Authentication is done via a 12-character seed phrase using X25519 and Argon2id.
- **P2P Architecture**: Direct connections using the QUIC protocol.
- **Adaptive QoS**: Dynamically adjusts bitrate, FPS, and resolution based on network conditions (RTT, packet loss).
- **LAN Optimization**: Instant discovery via mDNS and low-latency local streaming.
- **Cross-Platform**: Built with Rust and Tauri for Windows, macOS, and Linux support.

## Documentation
- [English Documentation](docs/en/)
- [Русская документация](docs/ru/)
- [Українська документація](docs/uk/)

## Quick Start (Development)
```bash
cargo build
cargo test
```
