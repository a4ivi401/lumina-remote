use crate::mdns_discovery;
use crate::nat;
use std::net::SocketAddr;
use tracing::{info, warn};

/// Represents the established network path to the target device.
pub enum ConnectionPath {
    /// Device was found directly on the local network (LAN).
    /// Ultra-low latency, no internet required.
    DirectLan(SocketAddr),
    
    /// Device is on the Internet. We have discovered our public IP via STUN.
    /// The caller must now exchange this IP with the peer via the Signal Server.
    P2pWan {
        our_socket: std::net::UdpSocket,
        our_public_addr: SocketAddr,
    },
    
    /// Device is behind a strict Symmetric NAT.
    /// We must use the Relay Server.
    Relay(SocketAddr),
}

/// The intelligent Connection Manager that orchestrates LAN vs WAN routing.
pub struct ConnectionManager {
    stun_server: String,
    _signal_server: String,
}

impl ConnectionManager {
    /// Creates a new Connection Manager.
    pub fn new(stun_server: String, signal_server: String) -> Self {
        Self {
            stun_server,
            _signal_server: signal_server,
        }
    }

    /// Determines the best and fastest path to connect to the target Host.
    /// 1. Prioritizes local network (mDNS) with a fast 2-second timeout.
    /// 2. Falls back to global Internet (STUN Hole Punching).
    /// 3. Falls back to Relay (if STUN fails).
    pub async fn establish_path(&self, device_id: &str) -> Result<ConnectionPath, String> {
        info!("Attempting LAN discovery for Device ID: {}", device_id);
        
        // 1. Try Local Area Network (mDNS)
        // We give it 2 seconds. If the device is on the same Wi-Fi, it will respond in < 100ms.
        match mdns_discovery::discover_local_host(device_id, 2).await {
            Ok(lan_addr) => {
                info!("⚡ Success! Found device directly on LAN: {}", lan_addr);
                return Ok(ConnectionPath::DirectLan(lan_addr));
            }
            Err(e) => {
                warn!("LAN discovery failed or timed out: {}. Falling back to WAN.", e);
            }
        }

        // 2. Try Wide Area Network (STUN Hole Punching)
        info!("Attempting WAN STUN discovery...");
        match nat::discover_public_endpoint(&self.stun_server).await {
            Ok((socket, public_addr)) => {
                info!("🌍 STUN Success! Our Public IP:PORT is {}", public_addr);
                info!("Ready to exchange addresses via Signal Server...");
                return Ok(ConnectionPath::P2pWan {
                    our_socket: socket,
                    our_public_addr: public_addr,
                });
            }
            Err(e) => {
                warn!("STUN discovery failed (UDP blocked?): {}. Must fallback to Relay.", e);
            }
        }

        // 3. Fallback to Relay Server
        // Here we would call the Relay Server API to get a fallback port.
        Err("All connection paths exhausted. Relay fallback not fully connected yet.".to_string())
    }
}
