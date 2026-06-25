use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};

/// Generates a dummy self-signed certificate for QUIC encryption.
/// In LuminaRemote, actual authentication is performed inside the tunnel
/// using the shared secret derived from the 12-character seed.
pub fn generate_dummy_certificate() -> (Vec<Certificate>, PrivateKey) {
    let subject_alt_names = vec!["lumina.local".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    let key = PrivateKey(cert.serialize_private_key_der());
    let cert_der = Certificate(cert.serialize_der().unwrap());
    (vec![cert_der], key)
}
