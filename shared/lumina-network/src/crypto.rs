use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};

/// Generates a dummy self-signed certificate for QUIC encryption.
/// In LuminaRemote, actual authentication is performed inside the tunnel
/// using the shared secret derived from the 12-character seed.
pub fn generate_dummy_certificate() -> (Vec<Certificate>, PrivateKey) {
    let subject_alt_names = vec!["lumina.a4ivi4.dev".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    let key = PrivateKey(cert.serialize_private_key_der());
    let cert_der = Certificate(cert.serialize_der().unwrap());
    (vec![cert_der], key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dummy_certificate() {
        let (certs, key) = generate_dummy_certificate();
        assert!(!certs.is_empty(), "Certificate chain should not be empty");
        assert!(!certs[0].0.is_empty(), "First certificate should have data");
        assert!(!key.0.is_empty(), "Private key should have data");
    }
}
