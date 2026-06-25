<div align="center">
  <img src="https://raw.githubusercontent.com/a4ivi401/lumina-remote/main/docs/assets/logo.png" alt="LuminaRemote Logo" width="120" />
  
  # LuminaRemote
  
  **Next-Generation, High-Performance Remote Desktop Software**
  
  [🇷🇺 Русский](README.ru.md) | [🇺🇦 Українська](README.uk.md) | [🇬🇧 English](README.md)

  <br />

  ![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
  ![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=FFFFFF)
  ![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)
  ![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)
  ![QUIC](https://img.shields.io/badge/Protocol-QUIC%20%2F%20WebRTC-blue?style=for-the-badge)
  ![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)
  ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
  ![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)

</div>

<br />

## 🌟 About The Project

**LuminaRemote** is a cutting-edge remote desktop application designed from the ground up to be ultra-fast, secure, and incredibly lightweight. 

Unlike traditional Electron-based applications like TeamViewer or AnyDesk, LuminaRemote is built on **Tauri** and **Rust**. It utilizes native OS-level screen capture APIs (DXGI for Windows, CoreGraphics for macOS, X11/Wayland for Linux) and transmits data via a modern **QUIC-based P2P tunnel**, bypassing the limitations of web browsers.

## ✨ Key Features

- ⚡ **Ultra-Low Latency:** Written purely in Rust for maximum performance. Hardware-accelerated VP9 video encoding ensures smooth 60FPS streaming.
- 🛡️ **Military-Grade Security:** True End-to-End Encryption using QUIC and TLS 1.3. Mutual HMAC-SHA256 authentication with Channel Binding prevents Man-In-The-Middle (MITM) attacks.
- 🌐 **Zero-Config Connectivity:** Effortlessly connects across the globe using STUN/TURN hole punching. If both devices are on the same Wi-Fi, it instantly switches to direct **mDNS** discovery for 0ms latency.
- 🎨 **Modern Cyberpunk UI:** A sleek, glassmorphism-inspired interface built with React. Minimal CPU/RAM footprint on the frontend.
- 🔌 **Unattended Access:** Securely save known machines locally for instant, one-click background access.

## 🏗️ Architecture overview

LuminaRemote consists of two main parts:
1. **The Client (Tauri):** The application installed on your device. It handles the UI, hardware screen capture, inputs, and P2P networking.
2. **The Signal Server (Axum):** A lightweight WebSocket server used *only* to exchange initial connection metadata (SDP/ICE equivalents) to establish a direct P2P tunnel between clients. **No video data ever passes through the server.**

## 🚀 Installation & Usage

1. Go to the [Releases](../../releases) tab.
2. Download the installer for your Operating System (`.exe` for Windows, `.dmg` for macOS, `.deb` for Linux).
3. Open the app. You will see your unique **Device ID**.
4. Share your ID and the generated **Session PIN** with your partner.
5. Your partner enters your ID and PIN to securely connect to your desktop!

## 🛠️ For Developers

Want to build LuminaRemote from source? 

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v20+)
- Build dependencies for your OS (e.g., `libwebkit2gtk-4.1-dev` on Linux).

### Running locally
```bash
# Clone the repository
git clone https://github.com/a4ivi401/lumina-remote.git
cd lumina-remote/client/lumina-ui

# Install frontend dependencies
npm install

# Start the Tauri development server
npm run tauri dev
```

## 📜 License
Distributed under the MIT License. See `LICENSE` for more information.
