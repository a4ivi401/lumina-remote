# LuminaRemote Infrastructure Deployment

This directory contains the necessary components for the central backend infrastructure.

## 1. Signal Server (`lumina-signal-server`)
This is the WebSocket server used to route P2P NAT-traversal requests (SDP/ICE equivalents) between the Host and Client so they can establish a direct QUIC P2P connection.

### How to Deploy (VPS / DigitalOcean / AWS)
1. Install **Docker** and **Docker Compose** on your server.
2. Clone this repository (or copy the `server` folder).
3. Navigate to the server folder:
   ```bash
   cd server/lumina-signal-server
   docker-compose up -d --build
   ```
4. The server will start on port `3000`. 
5. *(Optional but Recommended)*: Set up an Nginx Reverse Proxy with Let's Encrypt (Certbot) to expose port 3000 as `wss://lumina.a4ivi4.dev/ws`.

## 2. Relay Server (TURN / TCP Fallback)
*Coming soon.* Used only when both Host and Client are behind Strict Symmetric NATs and UDP Hole Punching fails.
