pub mod crypto;
pub mod handshake;
pub mod signaling;

use crate::crypto::generate_dummy_certificate;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::{client::ServerCertVerified, client::ServerCertVerifier, Certificate, Error as RustlsError, ServerName};
use std::{net::SocketAddr, sync::Arc, time::SystemTime};

struct SkipServerVerification;

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        // Accept any certificate (authentication is done post-QUIC via HMAC)
        Ok(ServerCertVerified::assertion())
    }
}

pub fn configure_server() -> ServerConfig {
    let (certs, key) = generate_dummy_certificate();
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    server_crypto.alpn_protocols = vec![b"lumina-hq".to_vec()];
    
    ServerConfig::with_crypto(Arc::new(server_crypto))
}

pub fn configure_client() -> ClientConfig {
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();
    client_crypto.alpn_protocols = vec![b"lumina-hq".to_vec()];
    
    ClientConfig::new(Arc::new(client_crypto))
}

pub fn create_server_endpoint(bind_addr: SocketAddr) -> Result<Endpoint, std::io::Error> {
    let server_config = configure_server();
    Endpoint::server(server_config, bind_addr)
}

pub fn create_client_endpoint(bind_addr: SocketAddr) -> Result<Endpoint, std::io::Error> {
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(configure_client());
    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handshake::perform_handshake;
    use lumina_core::derive_key_pair;
    use tokio::runtime::Runtime;

    #[test]
    fn test_quic_p2p_with_auth() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let seed = "23456789ABCD";
            // Client and Server have the same seed (shared offline)
            let (secret_s, _pub_s) = derive_key_pair(seed);
            let (secret_c, _pub_c) = derive_key_pair(seed);

            let server_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let server_endpoint = create_server_endpoint(server_addr).unwrap();
            let actual_server_addr = server_endpoint.local_addr().unwrap();

            let client_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let client_endpoint = create_client_endpoint(client_addr).unwrap();

            // Client connects
            let connect_task = client_endpoint.connect(actual_server_addr, "lumina.local").unwrap();
            
            // Server accepts
            let accept_task = server_endpoint.accept();

            let (client_conn_res, server_conn_opt) = tokio::join!(connect_task, accept_task);
            
            let client_conn = client_conn_res.unwrap();
            let server_incoming = server_conn_opt.unwrap();
            let server_conn = server_incoming.await.unwrap();

            // Perform application-layer mutual auth
            let hs_server = perform_handshake(&server_conn, true, &secret_s);
            let hs_client = perform_handshake(&client_conn, false, &secret_c);

            let (res_s, res_c) = tokio::join!(hs_server, hs_client);
            
            assert!(res_s.is_ok(), "Server handshake failed");
            assert!(res_c.is_ok(), "Client handshake failed");
        });
    }
}
