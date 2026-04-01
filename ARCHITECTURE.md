# NodeChat - Secure Decentralized Chat Application

## Project Overview

**Topic:** Secure Decentralized Chat Messaging Application
**Type:** School Final Year Project
**Team:** 2 members

### Abstract

This project builds a peer-to-peer (P2P) chat application with end-to-end encryption (E2EE) supporting both 1:1 direct communication and Decentralized Group Chat. Unlike traditional chat apps that rely on central servers, NodeChat uses decentralized networking via Iroh (for transport) and Pkarr (for peer discovery), so users communicate directly without routing messages through any central authority. The system utilizes an Event-Driven Actor Model to decouple the declarative Slint UI from the asynchronous networking, cryptographic, and storage operations.

---

## 1. Technology Stack

### 1.1 Backend (Rust)

| Crate | Version | Purpose |
|-------|---------|---------|
| **iroh** | 0.97.0 | P2P networking, hole punching, DERP relay fallback, secure connections |
| **iroh-gossip** | 0.97.0 | Broadcast messaging across P2P swarms (Group Chats) |
| **pkarr** | 5.0.4 | DNS-based decentralized peer discovery (public key → node address) |
| **x25519-dalek** | 2.0.1 | X25519 key exchange for 1:1 E2EE |
| **chacha20poly1305** | 0.10.1 | Authenticated encryption (AEAD) for 1:1 and Groups |
| **sha2** | 0.10.9 | SHA-256 hash ratcheting for Forward Secrecy |
| **rusqlite** | 0.39.0 | Embedded SQLite for robust local storage (queue, history, contacts) |
| **tokio** | 1.50.0 | Async runtime and MPSC channels for Actor Model |
| **uuid** | 1.23.0 | Unique message IDs |
| **anyhow** | 1.0.102 | Error handling |
| **rand** | 0.10.0 | Cryptographic random number generation |

> **Note on versions:** Always verify crate versions against `crates.io` before building. iroh in particular has an actively evolving API — pin exact versions in `Cargo.lock` and do not upgrade during active development.

### 1.2 Frontend (Rust + Slint)

| Crate | Version | Purpose |
|-------|---------|---------|
| **slint** | 1.15.1 | Declarative UI framework — `.slint` markup files + Rust property bindings |
| **slint-build** | 1.15.1 | Build script integration — compiles `.slint` files at build time |

> **Slint licence:** NodeChat qualifies for the **Community (GPL) licence** — free for open-source, non-commercial school projects. The only obligation is displaying "Made with Slint" somewhere in the app (About screen).

### 1.3 Platform Support

- **Primary:** Desktop — Windows, Linux, macOS (via Slint's native backends)
- **Stretch Goal:** Android (via Slint's official Android backend — significantly more stable than the previous egui/cargo-apk path)

> **Android note:** Slint has documented, maintained Android support. Verify a working Slint Android build on a physical device in Week 1. If it proves unstable, scope to desktop with Android documented as a planned extension.

---

## 2. System Architecture (Actor Model)

To prevent the UI from freezing during heavy network operations, cryptographic key exchanges, or database writes, the architecture uses a strictly decoupled **Actor Model**.

The Slint UI never directly touches the Database or Network. It sends `Command` events to the Backend Worker via a Tokio MPSC channel, and receives state updates back via `slint::invoke_from_event_loop`.

```
┌─────────────────────────────────────────────────────────────┐
│                      SLINT FRONTEND                         │
│  (Declarative .slint UI — driven by Rust property bindings) │
│                                                             │
│   [Chat View]   [Contact Book]   [Group Management]         │
│                                                             │
│   Zero business logic in .slint files.                      │
│   All state comes from ModelRc<T> and @property bindings.   │
└────────────┬───────────────────────────────────────▲────────┘
             │                                       │
  (mpsc::Sender<Command>)         (slint::invoke_from_event_loop)
   - SendDirectMessage              - AppEvent::IncomingMessage
   - CreateGroup                    - AppEvent::PeerOnlineStatus
   - AddContact                     - AppEvent::MessageStatusUpdate
             │                                       │
┌────────────▼───────────────────────────────────────┴────────┐
│                  TOKIO ASYNC BACKEND CORE                   │
│   (Asynchronous Event Loop - Manages State and Work IO)     │
│                                                             │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │     STORAGE     │ │     CRYPTO      │ │    NETWORK    │  │
│  │ (rusqlite DB)   │ │  (E2EE X25519)  │ │ (Iroh & Pkarr)│  │
│  │ - Messages      │ │  - 1:1 Keying   │ │ - Unicast     │  │
│  │ - Peers / Queues│ │  - Group Keys   │ │ - Gossip      │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### How Slint Communicates with the Backend

Unlike polling-based UIs, Slint uses a **push model**. The Tokio backend thread pushes state changes into the UI via `slint::invoke_from_event_loop`, which queues a closure to run safely on the Slint event thread:

```rust
// Pattern used in src/ui/mod.rs — backend pushes events into UI
let handle = app_window.as_weak();

tokio::spawn(async move {
    while let Ok(event) = rx_events.recv().await {
        let h = handle.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = h.upgrade() {
                crate::ui::models::apply_event(&ui, event);
            }
        });
    }
});
```

The Slint UI fires `Command`s back to the backend via callbacks wired once at startup:

```rust
// Pattern used in src/ui/mod.rs — wiring a send button callback
ui.on_send_message({
    let tx = tx_commands.clone();
    move |text| {
        let _ = tx.try_send(Command::SendDirectMessage {
            target: current_peer_id(),
            plaintext: text.to_string(),
        });
    }
});
```

---

## 3. Module Design

### 3.1 Network Layer (`src/p2p/`)

**Purpose:** Manages dual-mode communication. Iroh direct connections for 1:1 chats, and Iroh Gossip swarms for Groups.

```rust
// File: src/p2p/mod.rs

pub struct NetworkManager {
    endpoint:    Endpoint,
    gossip:      GossipApi,
    connections: HashMap<NodeId, Connection>, // reused per peer — not reopened per message
}

impl NetworkManager {
    /// Direct 1:1 message (Iroh unicast).
    /// Also used to transmit group symmetric keys during invites.
    pub async fn send_direct(&self, target: NodeId, payload: Vec<u8>) -> Result<()>;

    /// Group broadcast (Iroh Gossip).
    /// Broadcasts encrypted payload to all subscribers of the group TopicId.
    pub async fn broadcast_group(&self, topic: TopicId, payload: Vec<u8>) -> Result<()>;

    /// Join a Gossip swarm for a group. Idempotent — no-op if already subscribed.
    pub async fn subscribe_group(&self, topic: TopicId) -> Result<()>;

    /// Resolve a peer's current network address via Pkarr.
    /// Pkarr is a discovery layer only — not a transport layer.
    pub async fn discover_peer(&self, pk: PublicKey) -> Result<Option<NodeAddr>>;
}
```

---

### 3.2 Crypto Layer (`src/crypto/`)

**Purpose:** End-to-end encryption. Distinct approaches for 1:1 vs Groups.

**1:1 Direct Chat:**
We rely on Iroh's native Noise protocol for encrypted transport. For verifiable application-level E2EE, we perform a static X25519 Diffie-Hellman Key Exchange to derive an initial ChaCha20-Poly1305 symmetric key. After each message is sent or received, we apply a **Hash Ratchet** (SHA-256 of the current key) for Forward Secrecy.

> **Scope note:** This is a SHA-256 hash ratchet, not a full Double Ratchet. The Double Ratchet provides break-in recovery properties that are beyond this project's scope. The hash ratchet satisfies the forward secrecy requirement.

**Group Chat:**
Groups use **Shared Symmetric Keys**. When Alice creates a group, her client generates a random 32-byte ChaCha20 key. She invites Bob by sending him the Group `TopicId` and `SymmetricKey` via a secure 1:1 Direct E2EE message.

> **Known limitation — group key rotation:** Removed members retain the old key. Production systems re-generate and re-distribute the group key on every removal. Documented as a future extension.

```rust
// File: src/crypto/mod.rs

/// The user's cryptographic identity — generated once on first launch.
pub struct Identity {
    pub node_id:        NodeId,
    pub x25519_static:  StaticSecret,
}

impl CryptoManager {
    // ---- 1:1 CRYPTO ----
    pub fn derive_direct_key(our_secret: &StaticSecret, their_public: &PublicKey) -> [u8; 32];
    pub fn ratchet_key(current_key: &mut [u8; 32]);
    pub fn encrypt_direct(payload: &[u8], shared_key: &[u8; 32]) -> Vec<u8>;
    pub fn decrypt_direct(ciphertext: &[u8], shared_key: &[u8; 32]) -> Result<Vec<u8>>;

    // ---- GROUP CRYPTO ----
    pub fn generate_group_key() -> [u8; 32];
    pub fn encrypt_group(payload: &[u8], group_key: &[u8; 32]) -> Vec<u8>;
    pub fn decrypt_group(ciphertext: &[u8], group_key: &[u8; 32]) -> Result<Vec<u8>>;
}
```

---

### 3.3 Storage Layer (`src/storage/`)

**Purpose:** A unified SQLite database. If a peer is offline, the message is stored with `status = 'queued'` and flushed when the peer is reachable.

```sql
CREATE TABLE IF NOT EXISTS peers (
    node_id       TEXT PRIMARY KEY,
    display_name  TEXT NOT NULL,
    x25519_pubkey TEXT NOT NULL,
    verified      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS groups (
    topic_id      TEXT PRIMARY KEY,
    group_name    TEXT NOT NULL,
    symmetric_key BLOB NOT NULL  -- encrypted at rest under local password key
);

CREATE TABLE IF NOT EXISTS messages (
    id        TEXT PRIMARY KEY,
    type      TEXT NOT NULL,      -- 'text', 'file', 'group_invite'
    target_id TEXT NOT NULL,      -- NodeId (1:1) or TopicId (group)
    sender_id TEXT NOT NULL,
    content   BLOB NOT NULL,      -- decrypted plaintext stored locally
    timestamp INTEGER NOT NULL,   -- UTC Unix seconds
    status    TEXT NOT NULL       -- 'queued' → 'sent' → 'delivered' → 'read'
);
```

---

### 3.4 Offline Message Delivery

**State 1 — Direct P2P (both online):** Iroh direct connection, NAT hole punching, DERP relay fallback. Status: `sent` → `delivered`.

**State 2 — Local queue (Bob offline, Alice online):** Written to SQLite as `queued`. `flush_offline_queue()` retries every 10 seconds. Delivers when Pkarr resolves Bob's address.

**State 3 — DERP relay (NAT blocked):** Iroh transparently uses DERP. Relay forwards ciphertext only — never holds the decryption key.

**Acknowledged limitation:** The local queue only works while Alice's node is running. Extended mutual-offline delivery requires a DHT mailbox (Pkarr BEP44) — documented as a future extension.

---

### 3.5 Core Engine — The Actor Worker (`src/core/`)

```rust
// File: src/core/commands.rs

/// Commands sent FROM the Slint UI TO the Backend Worker.
pub enum Command {
    SendDirectMessage  { target: NodeId, plaintext: String },
    SendFile           { target: NodeId, file_path: PathBuf },
    NotifyReadReceipt  { target: NodeId, message_id: Uuid },
    CreateGroup        { name: String },
    SendGroupMessage   { topic: TopicId, plaintext: String },
    InviteToGroup      { target: NodeId, topic: TopicId },
}

/// Events pushed FROM the Backend Worker TO the Slint UI
/// via slint::invoke_from_event_loop.
pub enum AppEvent {
    IncomingMessage      { sender: NodeId, plaintext: String },
    IncomingFile         { sender: NodeId, file_name: String, path: PathBuf },
    MessageStatusUpdate  { id: Uuid, status: MessageStatus },
    IncomingGroupMessage { topic: TopicId, sender: NodeId, plaintext: String },
    GroupInviteReceived  { topic: TopicId, group_name: String },
    PeerOnlineStatus     { peer: NodeId, online: bool, via_relay: bool },
}
```

```rust
// File: src/core/mod.rs

pub struct NodeChatWorker {
    db:          rusqlite::Connection,
    network:     NetworkManager,
    crypto:      CryptoManager,
    rx_commands: mpsc::Receiver<Command>,
    tx_events:   broadcast::Sender<AppEvent>,
}

impl NodeChatWorker {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(cmd) = self.rx_commands.recv() => {
                    self.handle_command(cmd).await;
                }
                event = self.network.next_event() => {
                    self.handle_network(event).await;
                }
                _ = tokio::time::sleep(Duration::from_secs(QUEUE_FLUSH_INTERVAL_SECS)) => {
                    self.flush_offline_queue().await;
                }
            }
        }
    }
}
```

---

### 3.6 Frontend — Slint UI (`src/ui/` + `ui/`)

**`ui/` — `.slint` files:** Pure declarative markup. Layouts, components, colours, animations. Zero business logic. Emits named callbacks only.

**`src/ui/mod.rs` — Rust wiring:** Loads the compiled Slint window, wires callbacks to `Command` sends, spawns the event listener task that pushes `AppEvent`s into Slint models and properties.

**`src/ui/models.rs` — Data bridge:** Implements Slint's `ModelRc<T>` for message lists and contact lists. Contains the `apply_event()` dispatcher that translates `AppEvent` variants into Slint property updates.

---

### 3.7 User Identity & Onboarding

- **Zero-Server Registration:** X25519/Ed25519 keypair generated locally on first launch. Public key = `NodeId`. No cloud registration.
- **Local Display Name:** Shared only during P2P handshakes. Never sent to any server.
- **Local Password Encryption:** SQLite and private key optionally encrypted at rest. Password never transmitted.
- **Contact Model:** Local SQLite phonebook only. Contacts added by exchanging `NodeId` out-of-band (QR code, in person).

---

## 4. File Structure

```
nodechat/
├── AGENT.md
├── ARCHITECTURE.md
├── RULES.md
├── UX_FLOW.md
├── README.md
├── Cargo.toml
├── Cargo.lock
├── build.rs                      ← slint_build::compile("ui/app.slint")
│
├── src/
│   ├── main.rs                   ← boots Tokio + runs Slint event loop
│   │
│   ├── core/
│   │   ├── mod.rs                ← NodeChatWorker async select loop
│   │   └── commands.rs           ← Command + AppEvent definitions
│   │
│   ├── p2p/
│   │   ├── mod.rs                ← NetworkManager
│   │   ├── direct.rs             ← Iroh unicast
│   │   └── group.rs              ← Iroh Gossip
│   │
│   ├── crypto/
│   │   └── mod.rs                ← Identity, DH, encrypt/decrypt, ratchet
│   │
│   ├── storage/
│   │   ├── mod.rs                ← SQLite init + schema
│   │   └── queries.rs            ← All SQL CRUD
│   │
│   └── ui/
│       ├── mod.rs                ← NodeChatApp: loads window, wires callbacks, runs
│       └── models.rs             ← ModelRc impls + apply_event() dispatcher
│
├── ui/                           ← Pure .slint markup — zero Rust logic here
│   ├── app.slint                 ← Root AppWindow
│   ├── components/
│   │   ├── message_bubble.slint
│   │   ├── contact_row.slint
│   │   ├── chat_input.slint
│   │   └── status_dot.slint
│   └── screens/
│       ├── welcome.slint
│       ├── setup_name.slint
│       ├── identity_card.slint
│       ├── chat_list.slint
│       ├── chat_view.slint
│       ├── group_view.slint
│       ├── contacts.slint
│       ├── add_contact.slint
│       ├── verify_key.slint
│       └── settings.slint
│
├── assets/
│   ├── fonts/
│   │   └── inter.ttf
│   └── icons/
│       └── app_icon.png
│
└── tests/
    └── integration_tests.rs
```

---

## 5. Implementation Order

### Phase 1 — Core Foundation (Week 1–2)
- Set up Cargo workspace, add all dependencies, confirm `cargo build` is clean.
- Add `build.rs` with `slint_build::compile` and confirm a blank Slint window renders.
- Wire Tokio Actor Model channels between `main.rs` and `NodeChatWorker`.
- Initialize SQLite schema with WAL mode.
- Verify Slint Android build on a physical Android device (early risk check).

### Phase 2 — Peer-to-Peer 1:1 Networking (Week 3–4)
- Bind Iroh Endpoint in `NodeChatWorker`.
- Implement Pkarr peer discovery.
- Enable raw unicast data streams between two NodeIds on a LAN.

### Phase 3 — Identity & E2EE Crypto (Week 5–6)
- Implement X25519 keypair generation and persistence.
- Wire X25519 DH + ChaCha20-Poly1305 for 1:1 message encryption.
- Implement SHA-256 hash ratchet.
- Implement offline queue: write to SQLite on failure, flush on reconnect.

### Phase 4 — UI Integration (Week 7–8)
- Build all `.slint` components and screens per `UX_FLOW.md`.
- Wire all Slint callbacks to `Command` sends in `src/ui/mod.rs`.
- Wire all `AppEvent` variants to model updates via `invoke_from_event_loop`.
- Build full onboarding flow.

### Phase 5 — Group Chat Expansion (Week 9–10)
- Integrate `iroh-gossip` into the Network layer.
- Implement symmetric group key generation and encrypted SQLite storage.
- Implement group invite flow via secure 1:1 message.
- Build Group Chat screens and wire them.

### Phase 6 — Polish & Defense Prep (Week 11–12)
- End-to-end integration testing on two separate machines.
- All required unit and integration tests passing (per RULES.md section 10).
- Final documentation, README, and presentation slides.
- Android demo build if Slint Android toolchain is confirmed stable.

---

## 6. Testing Plan

### Unit Tests
- Crypto: ChaCha20 roundtrip, tampered ciphertext rejection, ratchet advancement, X25519 shared secret agreement.
- Storage: Queued message write/read, status transition enforcement, group key round-trip.

### Integration Tests
- Two `NodeChatWorker` instances on localhost exchanging a direct message end-to-end.
- Offline queue: send while recipient is stopped, restart recipient, verify flush within 15 seconds.
- Pkarr discovery: verify `discover_peer` resolves a locally bound test node.

---

## 7. Defense Points

1. **True Decentralization:** No server needed. Pkarr for discovery. Iroh for transport. DERP relay (optional fallback) touches only ciphertext it cannot decrypt.
2. **Actor Model Architecture:** Slint UI is fully decoupled from all networking and DB IO. Responsive regardless of network conditions.
3. **Data Sovereignty:** 100% of user data lives exclusively on the user's local device in an encrypted SQLite database.
4. **Honest Offline Queue Model:** Local queue guarantees delivery while Alice's node is running. Extended mutual-offline limitation explicitly documented with the DHT mailbox production path.
5. **Decentralized Groups:** iroh-gossip + symmetric key distributed via 1:1 E2EE. No group server.
6. **Robust NAT Traversal:** DERP relay fallback ensures connectivity even on aggressive networks.
7. **Forward Secrecy:** SHA-256 hash ratchet advances the session key after every message.
8. **Transparent Trade-offs:** Every simplification (hash ratchet, symmetric group keys, local queue) is documented with its production solution. Demonstrates understanding of the full problem space.

---

## 8. Design Decisions

### Why Slint over egui?
Slint's declarative `.slint` markup cleanly separates visual design from Rust logic — enforcing the Actor Model separation by design. It produces a polished, consumer-grade UI (critical for defense impression), has first-class documented Android support, supports hot-reload during development, and builds beautiful list views (chat bubbles, contact rows) with far less code than egui's immediate-mode API.

### The Contact Model
No centralized friend system. Identity is a cryptographic keypair. Contacts are a local SQLite phonebook only — `NodeId` → display name. Added by exchanging NodeIds out-of-band.

### Group E2EE
Shared ChaCha20 symmetric key delivered per-member via 1:1 E2EE avoids Signal's Sender Key coordination complexity in a serverless environment. Trade-off is explicitly documented.

### Offline Delivery
Local SQLite queue covers all realistic scenarios where Alice's node is running. DHT mailbox documented as production extension.

### Decentralized Onboarding
Keypair generated locally. Public key is the identity. Private key + DB encrypted with optional local password. No server ever involved.

---

## 9. Known Limitations & Future Extensions

| Limitation | Current Behavior | Production Extension |
|---|---|---|
| Extended mutual offline | Messages queued locally; held if Alice's node is off | DHT BEP44 store-and-forward mailbox |
| Group key rotation | Removed members retain old key | Re-key all remaining members on removal |
| Full forward secrecy | SHA-256 hash ratchet (one direction) | Full Signal Double Ratchet |
| Group member forward secrecy | Shared symmetric key | Signal Sender Keys per member |
| Android support | Stretch goal — Slint Android backend | Full Slint Android pipeline |

---

*Last Updated: 2026-03-30*
*Project: NodeChat — Secure Decentralized Chat*
*Version: 1.3 — Latest Crate Versions*
