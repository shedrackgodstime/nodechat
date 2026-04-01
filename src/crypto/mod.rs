//! Cryptographic identity, session key management, encryption, and ratchet.
//!
//! ## 1:1 E2EE
//! X25519 DH derives a shared secret → ChaCha20-Poly1305 AEAD.
//! After every sent/received message, SHA-256 advances the session key
//! for forward secrecy (RULES.md C-04).
//!
//! ## Group E2EE
//! Random 32-byte symmetric key per group, distributed via 1:1 E2EE invite.
//!
//! ## Safety Numbers
//! Canonical key ordering ensures both peers compute the same number (RULES.md C-07).

use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand_08::rngs::OsRng;
use rand_08::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};

/// Size of a ChaCha20-Poly1305 nonce in bytes.
const NONCE_SIZE: usize = 12;
/// Size of a symmetric key in bytes (X25519 shared secret / ChaCha20 key).
const KEY_SIZE: usize = 32;
/// Number of digits per group in a rendered safety number.
const SAFETY_NUMBER_GROUP_SIZE: usize = 5;
/// Total number of digit groups in a rendered safety number.
const SAFETY_NUMBER_GROUPS: usize = 12;

// ── Identity ──────────────────────────────────────────────────────────────────

/// The user's persistent cryptographic identity, generated once on first launch.
///
/// The raw `x25519_static` secret never leaves this module (RULES.md C-01).
pub struct Identity {
    /// Raw public-key bytes used to derive the Iroh NodeId (set from the Iroh keypair in p2p).
    pub node_id_bytes: [u8; KEY_SIZE],
    /// X25519 static secret — never passed outside `src/crypto/`.
    x25519_static: StaticSecret,
    /// X25519 public key shared with peers during handshake.
    pub x25519_public: PublicKey,
}

impl Identity {
    /// Generate a fresh random identity. Call exactly once on first launch.
    pub fn generate() -> Self {
        let mut node_id_bytes = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut node_id_bytes);
        let x25519_static = StaticSecret::random_from_rng(OsRng);
        let x25519_public = PublicKey::from(&x25519_static);
        Self { node_id_bytes, x25519_static, x25519_public }
    }

    /// Restore a persisted identity from raw bytes.
    pub fn from_bytes(node_id_bytes: [u8; KEY_SIZE], x25519_secret_bytes: [u8; KEY_SIZE]) -> Self {
        let x25519_static = StaticSecret::from(x25519_secret_bytes);
        let x25519_public = PublicKey::from(&x25519_static);
        Self { node_id_bytes, x25519_static, x25519_public }
    }

    /// Export the X25519 secret key bytes for encrypted persistence.
    ///
    /// The caller MUST store this value encrypted at rest (RULES.md C-01, C-05).
    pub fn x25519_secret_bytes(&self) -> [u8; KEY_SIZE] {
        self.x25519_static.to_bytes()
    }
}

// ── CryptoManager ─────────────────────────────────────────────────────────────

/// Manages live session keys and provides the encryption/decryption API.
///
/// Session keys live only in memory (RULES.md C-04) — never written to SQLite.
/// A fresh DH exchange must occur on every app restart.
pub struct CryptoManager {
    identity: Identity,
    /// Per-peer 1:1 session keys, keyed by hex-encoded NodeId.
    session_keys: HashMap<String, [u8; KEY_SIZE]>,
    /// Per-group symmetric keys, keyed by hex-encoded TopicId.
    group_keys: HashMap<String, [u8; KEY_SIZE]>,
}

impl CryptoManager {
    /// Create a new manager wrapping the given identity.
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            session_keys: HashMap::new(),
            group_keys: HashMap::new(),
        }
    }

    /// Returns a reference to the node's persistent identity.
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    // ── 1:1 Session Key Lifecycle ─────────────────────────────────────────────

    /// Derive and store the initial 1:1 session key for `peer_id` via X25519 DH.
    ///
    /// Call once when a new contact is first contacted. Subsequent messages
    /// use the stored key, advanced by `ratchet_key_for` after each message.
    pub fn init_session(&mut self, peer_id: &str, their_x25519_public_bytes: &[u8; KEY_SIZE]) {
        let their_public = PublicKey::from(*their_x25519_public_bytes);
        let shared = self.identity.x25519_static.diffie_hellman(&their_public);
        self.session_keys.insert(peer_id.to_owned(), *shared.as_bytes());
    }

    /// Returns `true` if a live session key exists for `peer_id`.
    pub fn has_session(&self, peer_id: &str) -> bool {
        self.session_keys.contains_key(peer_id)
    }

    /// Advance the session key for `peer_id` one ratchet step (SHA-256).
    ///
    /// MUST be called exactly once after every successfully sent or received
    /// message (RULES.md C-04). No-op if no session exists yet.
    pub fn ratchet_key_for(&mut self, peer_id: &str) {
        if let Some(key) = self.session_keys.get_mut(peer_id) {
            Self::ratchet_key(key);
        }
    }

    // ── Group Key Lifecycle ───────────────────────────────────────────────────

    /// Generate a fresh random 32-byte symmetric key for a new group.
    pub fn generate_group_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Register a group symmetric key (received via invite or generated locally).
    pub fn register_group_key(&mut self, topic_id: &str, key: [u8; KEY_SIZE]) {
        self.group_keys.insert(topic_id.to_owned(), key);
    }

    /// Returns the raw group key bytes for a topic, used when inviting new members.
    ///
    /// # Errors
    /// Returns an error if no key is registered for the topic.
    pub fn group_key_bytes(&self, topic_id: &str) -> Result<[u8; KEY_SIZE]> {
        self.group_keys
            .get(topic_id)
            .copied()
            .with_context(|| format!("no group key for topic {:?}", topic_id))
    }

    // ── Safety Numbers ────────────────────────────────────────────────────────

    /// Compute the safety number for displaying to the user during key verification.
    ///
    /// Uses canonical (lexicographic) key ordering so both peers independently
    /// compute the same value (RULES.md C-07).
    /// Returns 12 groups of 5 digits, space-separated.
    pub fn compute_safety_number(our_public: &PublicKey, their_public: &PublicKey) -> String {
        let our_bytes = our_public.as_bytes();
        let their_bytes = their_public.as_bytes();

        // Canonical ordering: lower bytes first.
        let hash = if our_bytes < their_bytes {
            Sha256::digest([our_bytes.as_ref(), their_bytes.as_ref()].concat())
        } else {
            Sha256::digest([their_bytes.as_ref(), our_bytes.as_ref()].concat())
        };

        // Expand hash nibbles into decimal digits, take the first 60.
        let digits: Vec<char> = hash
            .iter()
            .flat_map(|b| [b >> 4, b & 0x0f])
            .take(SAFETY_NUMBER_GROUPS * SAFETY_NUMBER_GROUP_SIZE)
            .map(|d| char::from_digit(u32::from(d), 10).unwrap_or('0'))
            .collect();

        digits
            .chunks(SAFETY_NUMBER_GROUP_SIZE)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    // ── 1:1 Encryption / Decryption ───────────────────────────────────────────

    /// Encrypt `plaintext` for direct 1:1 delivery to `peer_id`.
    ///
    /// Caller MUST call `ratchet_key_for(peer_id)` after a confirmed send (RULES.md C-04).
    /// Output: `[nonce (12 bytes) || ciphertext]` (RULES.md C-02).
    ///
    /// # Errors
    /// Returns an error if no session key exists for `peer_id` or encryption fails.
    pub fn encrypt_direct(&self, peer_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .session_keys
            .get(peer_id)
            .with_context(|| format!("no session key for peer {:?} — call init_session first", peer_id))?;
        Self::encrypt_with_key(plaintext, key)
    }

    /// Decrypt a 1:1 ciphertext received from `peer_id`.
    ///
    /// Caller MUST call `ratchet_key_for(peer_id)` after successful decryption (RULES.md C-04).
    /// On authentication failure: log the error and drop — do NOT retry (RULES.md C-03).
    ///
    /// # Errors
    /// Returns an error if no session key exists, the payload is malformed, or AEAD auth fails.
    pub fn decrypt_direct(&self, peer_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .session_keys
            .get(peer_id)
            .with_context(|| format!("no session key for peer {:?}", peer_id))?;
        Self::decrypt_with_key(ciphertext, key)
    }

    // ── Group Encryption / Decryption ─────────────────────────────────────────

    /// Encrypt `plaintext` for gossip broadcast to a group.
    ///
    /// Output: `[nonce (12 bytes) || ciphertext]` (RULES.md C-02).
    ///
    /// # Errors
    /// Returns an error if no key is registered for the topic or encryption fails.
    pub fn encrypt_group(&self, topic_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .group_keys
            .get(topic_id)
            .with_context(|| format!("no group key for topic {:?}", topic_id))?;
        Self::encrypt_with_key(plaintext, key)
    }

    /// Decrypt a group gossip message for the given topic.
    ///
    /// On auth failure: log and drop — do NOT retry (RULES.md C-03).
    ///
    /// # Errors
    /// Returns an error if no key exists or AEAD authentication fails.
    pub fn decrypt_group(&self, topic_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .group_keys
            .get(topic_id)
            .with_context(|| format!("no group key for topic {:?}", topic_id))?;
        Self::decrypt_with_key(ciphertext, key)
    }

    // ── Internal Helpers ──────────────────────────────────────────────────────

    /// Advance a key one step: replace it with SHA-256(current key).
    fn ratchet_key(key: &mut [u8; KEY_SIZE]) {
        let hash = Sha256::digest(key.as_ref());
        key.copy_from_slice(&hash);
    }

    /// Encrypt `plaintext` with `key`: fresh nonce prepended to ciphertext (RULES.md C-02).
    fn encrypt_with_key(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| anyhow::anyhow!("ChaCha20-Poly1305 encryption failed"))?;

        let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Split nonce from payload, then authenticate and decrypt.
    fn decrypt_with_key(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        if data.len() < NONCE_SIZE {
            anyhow::bail!("ciphertext too short to contain nonce ({} bytes)", data.len());
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("ChaCha20-Poly1305 decryption / authentication failed"))
    }
}

// ── Unit Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity() -> Identity {
        Identity::generate()
    }

    /// T-06: X25519 DH — Alice and Bob derive the same shared secret.
    #[test]
    fn x25519_dh_alice_and_bob_share_same_secret() {
        let alice = make_identity();
        let bob = make_identity();
        let secret_a = alice.x25519_static.diffie_hellman(&bob.x25519_public);
        let secret_b = bob.x25519_static.diffie_hellman(&alice.x25519_public);
        assert_eq!(secret_a.as_bytes(), secret_b.as_bytes());
    }

    /// T-07: X25519 DH — different key pairs derive different secrets.
    #[test]
    fn x25519_dh_different_pairs_produce_different_secrets() {
        let alice = make_identity();
        let bob = make_identity();
        let carol = make_identity();
        let ab = alice.x25519_static.diffie_hellman(&bob.x25519_public);
        let ac = alice.x25519_static.diffie_hellman(&carol.x25519_public);
        assert_ne!(ab.as_bytes(), ac.as_bytes());
    }

    /// T-01: ChaCha20 encrypt → decrypt roundtrip.
    #[test]
    fn chacha20_roundtrip_produces_original_plaintext() {
        let key = CryptoManager::generate_group_key();
        let plaintext = b"hello from nodechat";
        let ct = CryptoManager::encrypt_with_key(plaintext, &key).unwrap();
        let recovered = CryptoManager::decrypt_with_key(&ct, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    /// T-02: Tampered ciphertext is rejected.
    #[test]
    fn chacha20_rejects_tampered_ciphertext() {
        let key = CryptoManager::generate_group_key();
        let mut ct = CryptoManager::encrypt_with_key(b"tamper me", &key).unwrap();
        let last = ct.len() - 1;
        ct[last] ^= 0xFF;
        assert!(CryptoManager::decrypt_with_key(&ct, &key).is_err());
    }

    /// T-03: Wrong key is rejected.
    #[test]
    fn chacha20_rejects_wrong_key() {
        let key = CryptoManager::generate_group_key();
        let wrong = CryptoManager::generate_group_key();
        let ct = CryptoManager::encrypt_with_key(b"wrong key", &key).unwrap();
        assert!(CryptoManager::decrypt_with_key(&ct, &wrong).is_err());
    }

    /// T-04: Hash ratchet advances the key.
    #[test]
    fn hash_ratchet_advances_key_each_step() {
        let original = [0xABu8; 32];
        let mut key = original;
        CryptoManager::ratchet_key(&mut key);
        assert_ne!(key, original);
    }

    /// T-05: Hash ratchet is deterministic.
    #[test]
    fn hash_ratchet_is_deterministic() {
        let mut a = [0x12u8; 32];
        let mut b = [0x12u8; 32];
        CryptoManager::ratchet_key(&mut a);
        CryptoManager::ratchet_key(&mut b);
        assert_eq!(a, b);
    }

    /// T-08: Safety number is symmetric.
    #[test]
    fn safety_number_is_symmetric() {
        let alice = make_identity();
        let bob = make_identity();
        let a = CryptoManager::compute_safety_number(&alice.x25519_public, &bob.x25519_public);
        let b = CryptoManager::compute_safety_number(&bob.x25519_public, &alice.x25519_public);
        assert_eq!(a, b);
    }

    /// T-09: Group key roundtrip.
    #[test]
    fn group_key_roundtrip_produces_original_plaintext() {
        let mut mgr = CryptoManager::new(make_identity());
        let key = CryptoManager::generate_group_key();
        mgr.register_group_key("topic-abc", key);
        let plaintext = b"group broadcast";
        let ct = mgr.encrypt_group("topic-abc", plaintext).unwrap();
        let recovered = mgr.decrypt_group("topic-abc", &ct).unwrap();
        assert_eq!(recovered, plaintext);
    }

    /// T-10: Tampered group ciphertext is rejected.
    #[test]
    fn group_key_rejects_tampered_ciphertext() {
        let mut mgr = CryptoManager::new(make_identity());
        let key = CryptoManager::generate_group_key();
        mgr.register_group_key("tamper-topic", key);
        let mut ct = mgr.encrypt_group("tamper-topic", b"tamper").unwrap();
        ct[ct.len() - 1] ^= 0xFF;
        assert!(mgr.decrypt_group("tamper-topic", &ct).is_err());
    }

    /// T-20: 1000 encryptions produce 1000 unique nonces.
    #[test]
    fn nonce_uniqueness_across_many_encryptions() {
        let key = CryptoManager::generate_group_key();
        let mut nonces: std::collections::HashSet<[u8; 12]> = std::collections::HashSet::new();
        for _ in 0..1000 {
            let ct = CryptoManager::encrypt_with_key(b"nonce check", &key).unwrap();
            let mut n = [0u8; 12];
            n.copy_from_slice(&ct[..12]);
            nonces.insert(n);
        }
        assert_eq!(nonces.len(), 1000, "all nonces must be unique");
    }

    /// T-SHA256-VECTOR: Known output for the all-zeros key.
    #[test]
    fn sha256_ratchet_matches_known_vector() {
        let mut key = [0u8; 32];
        CryptoManager::ratchet_key(&mut key);
        assert_eq!(
            hex::encode(key),
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
        );
    }
}
