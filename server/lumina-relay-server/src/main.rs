use axum::{routing::post, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{info, warn};

#[derive(Serialize)]
struct AllocateResponse {
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // The control plane for the Relay Server.
    // The Signal Server (or clients) can request a new dedicated UDP relay port.
    let app = Router::new().route("/allocate", post(allocate_relay));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("Relay control server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Dynamically allocates an ephemeral UDP port and spawns a lightweight task
/// to proxy UDP packets symmetrically between exactly two peers.
///
/// Because Lumina uses QUIC with Zero-Trust pre-shared keys, the relay doesn't 
/// need to decrypt or understand the traffic. It just blindly forwards bytes.
/// If an attacker intercepts the port, the QUIC handshake will reject them anyway.
async fn allocate_relay() -> Json<AllocateResponse> {
    // Bind to port 0 to let the OS assign an available ephemeral port
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind UDP socket");
    let port = socket.local_addr().unwrap().port();
    
    info!("Allocated new UDP relay on port {}", port);

    tokio::spawn(async move {
        let mut peer1: Option<SocketAddr> = None;
        let mut peer2: Option<SocketAddr> = None;
        
        // 64KB buffer for UDP packets
        let mut buf = [0u8; 65535]; 
        
        loop {
            // Close the relay port if no traffic is seen for 10 minutes
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(600),
                socket.recv_from(&mut buf)
            ).await;

            match result {
                Ok(Ok((len, src))) => {
                    // Lock onto the first two unique IP addresses that send packets
                    if peer1.is_none() {
                        peer1 = Some(src);
                        info!("Relay port {}: Peer 1 locked to {}", port, src);
                    } else if peer1 != Some(src) && peer2.is_none() {
                        peer2 = Some(src);
                        info!("Relay port {}: Peer 2 locked to {}", port, src);
                    }

                    // Symmetrically forward the UDP packet
                    if Some(src) == peer1 {
                        if let Some(p2) = peer2 {
                            let _ = socket.send_to(&buf[..len], p2).await;
                        }
                    } else if Some(src) == peer2 {
                        if let Some(p1) = peer1 {
                            let _ = socket.send_to(&buf[..len], p1).await;
                        }
                    } else {
                        warn!("Relay port {}: Dropping packet from unknown third peer {}", port, src);
                    }
                }
                _ => {
                    info!("Relay port {} shutting down (timeout or closed)", port);
                    break;
                }
            }
        }
    });

    Json(AllocateResponse { port })
}
