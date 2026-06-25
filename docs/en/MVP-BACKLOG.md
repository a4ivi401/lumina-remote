# LuminaRemote: MVP Backlog & Roadmap

This document outlines the development plan for LuminaRemote, divided into phases, with detailed tasks for developers.

## Phase 1: Proof of Concept (PoC) — Transport & Video Foundation (Estimated: 4 weeks)
**Goal:** Prove the architecture, set up the network and video foundation, measure latency (< 50 ms in LAN).

- [x] **TASK-1.1:** Initialize project and base structure (cargo workspace).
- [x] **TASK-1.2:** Write `lumina-core` (Cryptography) module.
  - [x] Implement `derive_key_pair(seed: &str) -> (StaticSecret, PublicKey)`.
  - [x] Use `argon2` for key derivation from seed phrase.
- [ ] **TASK-1.3:** Setup `quinn` (QUIC) in `lumina-network`.
  - [ ] Integrate key generation as `rustls::CustomCertVerifier` / PSK.
  - [ ] Develop test data exchange (P2P echo client/server).
- [x] **TASK-1.4:** Screen capture module `lumina-capture`.
  - [x] Use cross-platform crate (`xcap`) for quick start (PoC).
  - [ ] *Note:* In the final stage (Phase 4), rewrite to native APIs (DirectX Desktop Duplication, ScreenCaptureKit) for maximum performance.
- [ ] **TASK-1.5:** Video encoding module `lumina-encoder`.
  - [ ] Integrate FFmpeg (`ffmpeg-sys-next`) for hardware H.264 encoding.
- [ ] **TASK-1.6:** Decoding and rendering module.
  - [ ] Window for video stream testing (`winit` + `softbuffer` or `egui`).
- [ ] **TASK-1.7:** Input transmission (`lumina-input`).
  - [ ] Mouse and keyboard event injection on the host.

## Phase 2: Network Layer & Signaling (Estimated: 4 weeks)
**Goal:** Full P2P connection over WAN with NAT traversal and authorization.

- [ ] **TASK-2.1:** Signal Server (`lumina-signal-server`).
  - [ ] Actor model on `tokio`.
  - [ ] In-memory session store.
  - [ ] REST/WebSocket for IP exchange.
- [ ] **TASK-2.2:** Connect Client to Signal Server.
- [ ] **TASK-2.3:** NAT Traversal (STUN).
- [ ] **TASK-2.4:** Basic Relay Server.

## Phase 3: LAN & Adaptivity (Estimated: 3 weeks)
**Goal:** Work in any network, instant LAN connection, dynamic QoS.

- [ ] **TASK-3.1:** mDNS discovery.
- [ ] **TASK-3.2:** Direct LAN connection algorithms.
- [ ] **TASK-3.3:** Adaptive Bitrate (QoS).
- [ ] **TASK-3.4:** Screen capture optimization (Dirty Rects).
- [ ] **TASK-3.5:** Focus loss handling (pause capture).

## Phase 4: GUI & Cross-Platform (Estimated: 5 weeks)
**Goal:** Commercial look, UI client, macOS/Linux support.

- [ ] **TASK-4.1:** Build base Tauri app (`src-tauri` / `ui`).
- [ ] **TASK-4.2:** UI layout (LAN/Online list, connect screen, control panel).
- [ ] **TASK-4.3:** macOS capture port (ScreenCaptureKit).
- [ ] **TASK-4.4:** Linux capture port (PipeWire).
- [ ] **TASK-4.5:** System integration (tray, autostart).

## Phase 5: Release (Estimated: 2 weeks)
**Goal:** Prepare stable version.

- [ ] **TASK-5.1:** Binary obfuscation and optimization.
- [ ] **TASK-5.2:** CI/CD (GitHub Actions) for cross-compilation.
- [ ] **TASK-5.3:** Security audit.
- [ ] **TASK-5.4:** Prepare installers.
