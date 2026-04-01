# NodeChat 💬 🔒

**Secure Decentralized Chat Application**

NodeChat is a purely peer-to-peer (P2P), serverless chat application built natively in Rust. Designed from the ground up for total data sovereignty and privacy, it requires no central servers, no cloud databases, and no user registration whatsoever.

*This project is built as a Polytechnic Final Year Project, evolving protocol concepts from the Artemis P2P research framework into a legitimate, secure consumer application.*

---

## ✨ Key Features

* **True Decentralization:** Uses `iroh` and `pkarr` (BitTorrent DHT) to enable direct peer-to-peer connection and NAT hole-punching. If strict networks block P2P, it silently falls back to DERP relays.
* **Zero-Server Identity:** Users mathematically own their identity via locally generated X25519 cryptographic keypairs. No emails, no usernames, no registration APIs.
* **End-to-End Encryption (E2EE):** 
  * *1:1 Direct Chat:* Secured natively via Iroh's Noise protocol transport, augmented with an application-layer X25519 Diffie-Hellman exchange + SHA-256 Hash Ratcheting for verifiable **Forward Secrecy**.
  * *Group Chat:* Achieved by broadcasting over `iroh-gossip` swarms, heavily secured via distributed ChaCha20 symmetric group keys.
* **Smart Offline Queueing:** Because there is no server to hold messages, payloads sent to offline peers are securely queued in a local embedded SQLite database and instantly flushed to the network the moment the peer is discovered online.
* **Robust Architecture:** Built using a strictly decoupled **Event-Driven Actor Model**. An asynchronous `tokio` background worker handles intense network/crypto IO, communicating via channels to a buttery-smooth synchronous `egui` graphical frontend.

## 🛠️ Technology Stack

* **Networking:** `iroh`, `iroh-gossip`, `pkarr`
* **Cryptography:** `x25519-dalek`, `chacha20poly1305`, `sha2`
* **Core Systems:** `tokio` (Async Actor Model), `rusqlite` (Embedded Storage)
* **GUI / Frontend:** `egui`, `eframe` (Cross-platform Desktop & Android)

## 📖 Architecture & Documentation

For a comprehensive technical breakdown of the Actor Model, the SQLite schema, cryptographic handshakes, and our academic design decisions, please refer to the primary **[ARCHITECTURE.md](./ARCHITECTURE.md)** document.

## 🚀 Getting Started

*(Note: Project is currently entering Phase 1 implementation)*

```bash
# Clone the repository
git clone https://github.com/yourusername/nodechat.git
cd nodechat

# Build and run the desktop client natively
cargo run --release
```

---
*NodeChat Final Year Project*