//! nodechat Cryptographic Engine (NC-Crypto)
//! ---------------------------------------------------------
//! Implements ChaCha20-Poly1305 AEAD for group and direct encryption.
//! Nonces are random 12-byte values prepended to ciphertext.

use anyhow::{Result, bail};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

/// Generate a fresh random 32-byte symmetric key for a new group.
pub fn generate_group_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Derive a 32-byte encryption key using X25519 Diffie-Hellman + SHA-256 KDF.
pub fn derive_shared_secret(my_secret_bytes: &[u8; 32], peer_public_bytes: &[u8; 32]) -> [u8; 32] {
    let my_secret = StaticSecret::from(*my_secret_bytes);
    let peer_public = PublicKey::from(*peer_public_bytes);
    
    // Compute Diffie-Hellman shared secret
    let shared_secret = my_secret.diffie_hellman(&peer_public);
    
    // Hash it using SHA-256 to ensure uniformly distributed key space and destroy structure
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.as_bytes());
    let result = hasher.finalize();
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Derive an X25519 keypair from an Ed25519 seed (Iroh SecretKey bytes).
/// This ensures we don't reuse the same raw bytes for different primitives.
pub fn derive_x25519_keypair(seed: &[u8; 32]) -> (StaticSecret, [u8; 32]) {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(b"nodechat-x25519-derivation"); // Context separator
    let result = hasher.finalize();
    
    let mut x_seed = [0u8; 32];
    x_seed.copy_from_slice(&result);
    
    let secret = StaticSecret::from(x_seed);
    let public = PublicKey::from(&secret);
    
    (secret, public.to_bytes())
}

/// Encrypt `plaintext` using a symmetric `key`.
/// Output: `[nonce (12 bytes) || ciphertext]`
pub fn encrypt(plaintext: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>> {
    if key_bytes.len() != KEY_SIZE {
        bail!("invalid key size: expected 32, got {}", key_bytes.len());
    }
    
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| anyhow::anyhow!("AEAD encryption failed"))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt a ciphertext `data` using a symmetric `key`.
/// Expects: `[nonce (12 bytes) || ciphertext]`
pub fn decrypt(data: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>> {
    if key_bytes.len() != KEY_SIZE {
        bail!("invalid key size: expected 32, got {}", key_bytes.len());
    }
    if data.len() < NONCE_SIZE {
        bail!("ciphertext too short to contain nonce");
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("AEAD decryption / authentication failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_group_key();
        let message = b"Confidential NodeChat Data";
        
        let ciphertext = encrypt(message, &key).expect("Encryption failed");
        assert_ne!(message.to_vec(), ciphertext, "Ciphertext must not match plaintext");
        assert!(ciphertext.len() > message.len(), "Ciphertext must include nonce and tag");

        let decrypted = decrypt(&ciphertext, &key).expect("Decryption failed");
        assert_eq!(message.to_vec(), decrypted, "Decrypted message must match original");
    }

    #[test]
    fn test_tamper_detection() {
        let key = generate_group_key();
        let mut ciphertext = encrypt(b"Immutable Data", &key).unwrap();
        
        // Tamper with the ciphertext (the last byte)
        if let Some(last) = ciphertext.last_mut() {
            *last ^= 0xFF;
        }

        let result = decrypt(&ciphertext, &key);
        assert!(result.is_err(), "Authentication must fail if ciphertext is tampered");
    }

    #[test]
    fn test_ecdh_key_agreement() {
        let alice_seed = [1u8; 32];
        let bob_seed = [2u8; 32];

        let (alice_secret, alice_public) = derive_x25519_keypair(&alice_seed);
        let (bob_secret, bob_public) = derive_x25519_keypair(&bob_seed);

        let alice_shared = derive_shared_secret(&alice_secret.to_bytes(), &bob_public);
        let bob_shared = derive_shared_secret(&bob_secret.to_bytes(), &alice_public);

        assert_eq!(alice_shared, bob_shared, "Diffie-Hellman shared secrets must match");
    }
}
