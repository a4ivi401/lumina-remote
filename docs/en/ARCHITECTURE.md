# LuminaRemote: Architecture

This document describes the modular architecture of the LuminaRemote project, the purpose of each component, and how they interact.

## 1. Components Overview (C4 Model)

LuminaRemote operates as a hybrid P2P application. All Rust components are unified in a single Cargo workspace.

```mermaid
graph TD
    subgraph Client UI ["UI Layer (React / Vue)"]
        A(Graphical User Interface)
    end
    
    subgraph Tauri Backend ["Tauri Wrapper (src-tauri)"]
        B(IPC Layer)
    end
    
    subgraph Rust Crates ["Core Application (Rust)"]
        Core[lumina-core]
        Proto[lumina-protocol]
        Net[lumina-network]
        Capture[lumina-capture]
        Encode[lumina-encoder]
        Input[lumina-input]
    end
    
    subgraph External Infra ["Infrastructure"]
        SigServer(Lumina Signal Server)
        Relay(Lumina Relay Server)
    end
    
    A <-->|Tauri IPC| B
    B --> Core
    B --> Net
    B --> Capture
    B --> Encode
    B --> Input
    
    Net <-->|WebSocket| SigServer
    Net <-->|QUIC| Relay
    Net <-->|QUIC (P2P)| Net
```

## 2. Crates Description

* **`lumina-core`**: Core types, configurations, cryptography (Argon2id, X25519).
* **`lumina-protocol`**: Serialization of video frames and input commands.
* **`lumina-network`**: QUIC-based P2P connectivity, mDNS LAN discovery, STUN NAT traversal.
* **`lumina-capture`**: Cross-platform screen/audio capture.
* **`lumina-encoder`**: Hardware-accelerated video encoding via FFmpeg.
* **`lumina-input`**: Keyboard and mouse emulation injection.
* **`lumina-signal-server`**: Cloud server for assisting in handshakes and public IP exchange.

## 3. Zero-Trust Security Model

```mermaid
sequenceDiagram
    participant Host
    participant Signal as Signal Server
    participant Client
    
    Host->>Host: Generate 12-char Seed
    Host->>Host: Argon2(Seed) -> MasterKey -> X25519 Keys
    Host->>Signal: Register (PubKey_H, WAN_IP)
    
    Note over Client: User enters the 12-char seed
    
    Client->>Client: Argon2(Seed) -> MasterKey -> X25519 Keys
    
    Client->>Signal: Request access to PubKey_H
    Signal-->>Client: Returns Host IP Candidates
    
    Client->>Host: QUIC Handshake
    
    Host->>Host: ECDH(PrivKey_H, PubKey_C) -> SharedSecret
    Client->>Client: ECDH(PrivKey_C, PubKey_H) -> SharedSecret
    
    Host<-->>Client: TLS 1.3 PSK Authenticated Connection
```
