# NodeChat 💬 🔒

**Secure Decentralized Chat Application**

NodeChat is a purely peer-to-peer (P2P), serverless chat application built natively in Rust. Designed from the ground up for total data sovereignty and privacy, it requires no central servers, no cloud databases, and no user registration whatsoever.

*This project is built as a Polytechnic Final Year Project, evolving protocol concepts from the Artemis P2P research framework into a legitimate, secure consumer application.*

---

## ✨ Key Features

- **True Decentralization:** Uses `iroh` and `pkarr` for direct peer-to-peer connections and NAT hole-punching. Falls back silently to DERP relay servers on aggressive networks — the relay only ever touches ciphertext it cannot decrypt.

- **Zero-Server Identity:** Users mathematically own their identity via locally generated X25519 cryptographic keypairs. No emails, no usernames, no registration APIs.

- **End-to-End Encryption (E2EE):**
  - *1:1 Direct Chat:* Secured via Iroh's Noise protocol transport, augmented with an application-layer X25519 Diffie-Hellman exchange + SHA-256 Hash Ratcheting for verifiable **Forward Secrecy**.
  - *Group Chat:* `iroh-gossip` broadcast swarms secured via distributed ChaCha20 symmetric group keys.

- **Smart Offline Queueing:** Messages sent to offline peers are stored securely in a local embedded SQLite database and flushed to the network the moment the peer is discovered online.

- **Robust Actor Model Architecture:** An asynchronous Tokio background worker handles all network and crypto IO, communicating via channels to a smooth, responsive Slint UI frontend. The UI never blocks.

- **Polished Cross-Platform UI:** Built with Slint — a declarative `.slint` markup UI framework that produces a native, production-quality interface on both desktop and Android from a single codebase.

---

## 🛠️ Technology Stack

| Layer | Technology |
|---|---|
| P2P Networking | `iroh` 0.97.0, `iroh-gossip` 0.97.0, `pkarr` 5.0.4 |
| Cryptography | `x25519-dalek` 2.0.1, `chacha20poly1305` 0.10.1, `sha2` 0.10.9, `rand` 0.10.0 |
| Async Runtime | `tokio` 1.50.0 |
| Local Storage | `rusqlite` 0.39.0 |
| UI Framework | `slint` 1.15.1 (declarative markup + Rust bindings) |
| Utilities | `uuid` 1.23.0, `anyhow` 1.0.102 |
| Platform | Desktop (Windows, Linux, macOS) + Android (stretch goal) |

> **Note:** See [ARCHITECTURE.md](./ARCHITECTURE.md) for the complete dependency list with versions.

---

## 📖 Architecture & Documentation

| Document | Purpose |
|---|---|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Full system design, module hierarchy, cryptographic decisions, implementation phases |
| [UX_FLOW.md](./UX_FLOW.md) | Complete UX flow, screen designs, component specifications |
| [RULES.md](./RULES.md) | Strict engineering and testing standards — binding for all contributors |
| [AGENT.md](./AGENT.md) | Rules for AI agent interactions with this codebase |

---

## 🗂️ Project Structure

```
nodechat/
├── src/               ← Rust backend + UI wiring
│   ├── core/          ← Actor Model worker (Command/AppEvent loop)
│   ├── p2p/           ← Iroh networking (unicast + gossip)
│   ├── crypto/        ← X25519 DH, ChaCha20, hash ratchet
│   ├── storage/       ← SQLite schema and queries
│   └── ui/            ← Slint window loader + callback wiring
├── ui/                ← .slint UI markup files (zero Rust logic)
│   ├── components/    ← Reusable components (bubble, contact row, etc.)
│   └── screens/       ← Full screen layouts
├── assets/            ← Fonts, icons
└── tests/             ← Integration tests
```

---

## 🚀 Getting Started

*(Note: Project is currently entering Phase 1 implementation)*

```bash
# Clone the repository
git clone https://github.com/yourusername/nodechat.git
cd nodechat

# Build and run the desktop client
cargo run --release
```

---

## 📋 Implementation Status

| Phase | Description | Status |
|---|---|---|
| Phase 1 | Cargo workspace, Slint window, SQLite schema, Actor Model channels | 🔲 Not started |
| Phase 2 | Iroh endpoint, Pkarr discovery, LAN unicast | 🔲 Not started |
| Phase 3 | X25519 identity, ChaCha20 E2EE, hash ratchet, offline queue | 🔲 Not started |
| Phase 4 | Full Slint UI, all screens wired | 🔲 Not started |
| Phase 5 | iroh-gossip group chat, symmetric key distribution | 🔲 Not started |
| Phase 6 | Polish, testing, defense prep | 🔲 Not started |

---

*NodeChat — Final Year Project*
*Built with Slint 1.15.1 · Powered by Iroh 0.97.0 · Secured by X25519 + ChaCha20*
