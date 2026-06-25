use argon2::{Algorithm, Argon2, Params, Version};
use x25519_dalek::{PublicKey, StaticSecret};

/// Derives an X25519 static secret and public key pair from a given 12-character seed.
pub fn derive_key_pair(seed: &str) -> (StaticSecret, PublicKey) {
    // We use a fixed salt for now since the seed itself is the source of entropy
    // and both sides need to derive the same key without communicating the salt beforehand.
    let fixed_salt = b"LuminaRemote_Fixed_Salt!";
    
    // Argon2id parameters. m=64MB, t=3, p=2 as per tech spec.
    let params = Params::new(65536, 3, 2, Some(32)).unwrap();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key_material = [0u8; 32];
    
    // Derive the key
    let result = argon2.hash_password_into(
        seed.as_bytes(),
        fixed_salt,
        &mut key_material,
    );
    
    assert!(result.is_ok(), "Failed to derive key from seed");
    
    let secret = StaticSecret::from(key_material);
    let public = PublicKey::from(&secret);
    
    (secret, public)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_pair() {
        let seed = "23456789ABCD";
        let (secret1, public1) = derive_key_pair(seed);
        let (secret2, public2) = derive_key_pair(seed);

        // Same seed must produce same keys
        assert_eq!(public1.as_bytes(), public2.as_bytes());
        
        // Different seed must produce different keys
        let (_, public3) = derive_key_pair("23456789ABCE");
        assert_ne!(public1.as_bytes(), public3.as_bytes());
    }
}
