//! nodechat Cryptographic Engine (NC-Crypto)
//! ---------------------------------------------------------
//! Implements ChaCha20-Poly1305 AEAD for group and direct encryption.
//! Nonces are random 12-byte values prepended to ciphertext.

use anyhow::{Context, Result, bail};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

/// Generate a fresh random 32-byte symmetric key for a new group.
pub fn generate_group_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
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
