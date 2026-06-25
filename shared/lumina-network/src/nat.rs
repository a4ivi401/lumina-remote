use rand::Rng;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{lookup_host, UdpSocket};
use tokio::time::timeout;

/// Connects to a public STUN server to discover the public WAN IP and Port
/// of our UDP socket. This allows us to perform UDP Hole Punching for P2P QUIC.
/// Returns the bound `std::net::UdpSocket` (ready for Quinn) and our Public `SocketAddr`.
pub async fn discover_public_endpoint(
    stun_server: &str,
) -> Result<(std::net::UdpSocket, SocketAddr), String> {
    // Bind to any local port
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Bind error: {}", e))?;

    // Resolve STUN server address
    let server_addr = lookup_host(stun_server)
        .await
        .map_err(|e| format!("DNS resolution failed for {}: {}", stun_server, e))?
        .next()
        .ok_or(format!("No IP found for STUN server: {}", stun_server))?;

    // Craft a simple raw STUN Binding Request (RFC 5389)
    // Header (20 bytes): Type (0x0001), Length (0x0000), Magic (0x2112A442), Transaction ID (12 bytes)
    let mut request = [0u8; 20];
    request[0] = 0x00;
    request[1] = 0x01;
    request[2] = 0x00;
    request[3] = 0x00;
    // Magic Cookie
    request[4] = 0x21;
    request[5] = 0x12;
    request[6] = 0xA4;
    request[7] = 0x42;
    
    // Generate Random Transaction ID
    let mut rng = rand::thread_rng();
    for i in 8..20 {
        request[i] = rng.gen();
    }

    socket
        .send_to(&request, server_addr)
        .await
        .map_err(|e| format!("STUN send error: {}", e))?;

    let mut buf = [0u8; 1024];
    
    // Wait up to 3 seconds for the STUN response
    let (len, _) = timeout(Duration::from_secs(3), socket.recv_from(&mut buf))
        .await
        .map_err(|_| "STUN request timed out (Blocked by firewall?)")?
        .map_err(|e| format!("STUN receive error: {}", e))?;

    if len < 20 {
        return Err("STUN response too short".to_string());
    }

    // Verify Magic Cookie
    if buf[4..8] != [0x21, 0x12, 0xA4, 0x42] {
        return Err("Invalid STUN magic cookie received".to_string());
    }

    // Parse STUN Attributes to find MAPPED-ADDRESS or XOR-MAPPED-ADDRESS
    let mut offset = 20;
    while offset + 4 <= len {
        let attr_type = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let attr_len = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]) as usize;
        offset += 4;

        if offset + attr_len > len {
            break;
        }

        // XOR-MAPPED-ADDRESS (0x0020)
        if attr_type == 0x0020 {
            if buf[offset + 1] == 0x01 {
                // IPv4
                let port = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]) ^ 0x2112;
                let ip_bytes = [
                    buf[offset + 4] ^ 0x21,
                    buf[offset + 5] ^ 0x12,
                    buf[offset + 6] ^ 0xA4,
                    buf[offset + 7] ^ 0x42,
                ];
                let ip = std::net::Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);

                // Convert Tokio socket back to STD socket for Quinn compatibility
                let std_socket = socket.into_std().map_err(|e| e.to_string())?;
                std_socket.set_nonblocking(true).unwrap();
                return Ok((std_socket, SocketAddr::new(std::net::IpAddr::V4(ip), port)));
            }
        }

        // MAPPED-ADDRESS (0x0001) Fallback
        if attr_type == 0x0001 {
            if buf[offset + 1] == 0x01 {
                // IPv4
                let port = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
                let ip = std::net::Ipv4Addr::new(
                    buf[offset + 4],
                    buf[offset + 5],
                    buf[offset + 6],
                    buf[offset + 7],
                );

                let std_socket = socket.into_std().map_err(|e| e.to_string())?;
                std_socket.set_nonblocking(true).unwrap();
                return Ok((std_socket, SocketAddr::new(std::net::IpAddr::V4(ip), port)));
            }
        }

        offset += attr_len;
        if offset % 4 != 0 {
            offset += 4 - (offset % 4); // STUN attributes are padded to 32-bit (4 bytes) boundaries
        }
    }

    Err("STUN response did not contain a valid public address".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stun_discovery() {
        // We use Google's public STUN server for testing
        let result = discover_public_endpoint("stun.l.google.com:19302").await;
        assert!(result.is_ok(), "STUN discovery failed: {:?}", result.err());
        
        let (_, public_addr) = result.unwrap();
        println!("Discovered Public Address: {}", public_addr);
        assert!(public_addr.port() > 0);
    }
}
