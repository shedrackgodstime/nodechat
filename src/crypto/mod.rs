use anyhow::{Result, anyhow};
use x25519_dalek::{StaticSecret, PublicKey};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::{Aead, KeyInit}};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use rand::RngCore;

/// The user's cryptographic identity, deterministic based on the provided secret seed.
pub struct Identity {
    pub x25519_secret: StaticSecret,
    pub x25519_public: PublicKey,
}

impl Identity {
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let x25519_secret = StaticSecret::from(*seed);
        let x25519_public = PublicKey::from(&x25519_secret);
        Self { x25519_secret, x25519_public }
    }
}

/// Encryption and key management logic for NodeChat.
pub struct CryptoManager {
    pub identity: Option<Identity>,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self { identity: None }
    }

    pub fn set_identity(&mut self, identity: Identity) {
        self.identity = Some(identity);
    }

    // ---- 1:1 DIRECT CRYPTO (X25519 + ChaCha20) ----

    /// Derive a 32-byte shared secret using X25519 Diffie-Hellman.
    pub fn derive_shared_secret(our_secret: &StaticSecret, their_public: &PublicKey) -> [u8; 32] {
        let shared_secret = our_secret.diffie_hellman(their_public);
        *shared_secret.as_bytes()
    }

    /// Advance the session key via SHA-256 hash ratchet for Forward Secrecy.
    pub fn ratchet_key(current_key: &mut [u8; 32]) {
        let mut hasher = Sha256::new();
        hasher.update(*current_key);
        let result = hasher.finalize();
        current_key.copy_from_slice(&result);
    }

    /// Encrypt a payload for a single peer using the current session key.
    /// Prepends a fresh 12-byte nonce to the ciphertext.
    pub fn encrypt_direct(payload: &[u8], session_key: &[u8; 32]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(session_key));
        let ciphertext = cipher.encrypt(nonce, payload)
            .map_err(|e| anyhow!("Encryption failure: {}", e))?;

        // Format: [12 bytes nonce | ciphertext + 16 bytes tag]
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt a payload received from a direct peer.
    pub fn decrypt_direct(data: &[u8], session_key: &[u8; 32]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow!("Ciphertext too short (missing nonce)"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(session_key));
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failure (potential tampering): {}", e))
    }

    // ---- GROUP CRYPTO (Shared Symmetric Keys) ----

    /// Generate a new random 32-byte key for a new Group chat.
    pub fn generate_group_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt a group payload. Same logic as direct but uses group key.
    pub fn encrypt_group(payload: &[u8], group_key: &[u8; 32]) -> Result<Vec<u8>> {
        Self::encrypt_direct(payload, group_key)
    }

    /// Decrypt a group payload.
    pub fn decrypt_group(data: &[u8], group_key: &[u8; 32]) -> Result<Vec<u8>> {
        Self::decrypt_direct(data, group_key)
    }
}
