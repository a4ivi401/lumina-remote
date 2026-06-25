use hmac::{Hmac, Mac};
use quinn::Connection;
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::StaticSecret;

type HmacSha256 = Hmac<Sha256>;

/// Performs mutual authentication over an established QUIC connection.
/// This guarantees that both sides possess the same StaticSecret (derived from the 12-char seed).
pub async fn perform_handshake(
    conn: &Connection,
    is_server: bool,
    shared_secret: &StaticSecret,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut nonce);
    
    // We use a dedicated bidirectional stream for the handshake
    let (mut send, mut recv) = if is_server {
        let (send, recv) = conn.accept_bi().await?;
        (send, recv)
    } else {
        let (send, recv) = conn.open_bi().await?;
        (send, recv)
    };

    // 1. Send our nonce
    send.write_all(&nonce).await?;
    
    // 2. Receive peer's nonce
    let mut peer_nonce = [0u8; 32];
    recv.read_exact(&mut peer_nonce).await?;
    
    // 3. Calculate HMAC of peer's nonce using the shared secret
    let secret_bytes = shared_secret.to_bytes();
    let mut mac = HmacSha256::new_from_slice(&secret_bytes)?;
    mac.update(&peer_nonce);
    let my_proof = mac.finalize().into_bytes();
    
    // 4. Send our proof
    send.write_all(&my_proof).await?;
    
    // 5. Receive peer's proof
    let mut peer_proof = [0u8; 32];
    recv.read_exact(&mut peer_proof).await?;
    
    // 6. Verify peer's proof
    let mut verify_mac = HmacSha256::new_from_slice(&secret_bytes)?;
    verify_mac.update(&nonce);
    
    // If this fails, it returns an Error and the connection should be dropped
    verify_mac.verify_slice(&peer_proof)?;
    
    Ok(())
}
