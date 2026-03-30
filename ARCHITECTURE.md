# NodeChat - Secure Decentralized Chat Application

## Project Overview

**Topic:** Secure Decentralized Chat Messaging Application  
**Type:** School Final Year Project  
**Team:** 2 members

### Abstract

This project builds a peer-to-peer (P2P) chat application with end-to-end encryption (E2EE) supporting both 1:1 direct communication and Decentralized Group Chat. Unlike traditional chat apps that rely on central servers, NodeChat uses decentralized networking via Iroh (for transport) and Pkarr (for peer discovery), so users communicate directly without routing messages through any central authority. The system utilizes an Event-Driven Actor Model to decouple the immediate-mode UI from the asynchronous networking, cryptographic, and storage operations.

---

## 1. Technology Stack

### 1.1 Backend (Rust)

| Crate | Version | Purpose |
|-------|---------|---------|
| **iroh** | 0.97.0 | P2P networking, hole punching, DERP relay fallback, secure connections |
| **iroh-gossip** | 0.97.0 | Broadcast messaging across P2P swarms (Group Chats) |
| **pkarr** | 5.0 | DNS-based decentralized peer discovery (public key → node address) |
| **x25519-dalek** | 2.0 | X25519 key exchange for 1:1 E2EE |
| **chacha20poly1305** | 0.10 | Authenticated encryption (AEAD) for 1:1 and Groups |
| **sha2** | 0.10 | SHA-256 hash ratcheting for Forward Secrecy |
| **rusqlite** | 0.31 | Embedded SQLite for robust local storage (queue, history, contacts) |
| **tokio** | 1.x | Async runtime and MPSC channels for Actor Model |
| **uuid** | 1.x | Unique message IDs |
| **anyhow** | 1.x | Error handling |

> **Note on versions:** Always verify crate versions against `crates.io` before building. iroh in particular has an actively evolving API — pin exact versions in `Cargo.lock` and do not upgrade during active development.

### 1.2 Frontend (Rust + egui)

Based on **hello_android** project structure for cross-platform support:

| Crate | Version | Purpose |
|-------|---------|---------|
| **eframe** | 0.34 | GUI framework |
| **egui** | 0.34 | Immediate-mode UI toolkit |
| **egui_extras** | 0.34 | Extra UI components, images |

### 1.3 Platform Support

- **Primary:** Desktop — Windows, Linux, macOS (via eframe)
- **Stretch Goal:** Android (via cargo-apk, building off hello_android template)

> **Android risk note:** The cargo-apk + egui Android toolchain is complex to configure. A working hello_android build should be verified in Week 1. If Android proves unstable, the project is scoped to desktop with Android documented as a planned extension.

---

## 2. System Architecture (Actor Model)

To prevent the application UI from freezing during heavy network operations, cryptographic key exchanges, or database writes, the architecture utilizes a strictly decoupled **Actor Model**.

The UI (Frontend) never directly touches the Database or Network. It sends `Command` events down a channel to the Backend Worker, and dynamically updates its views based on `AppEvent` responses broadcast from the worker.

```
┌─────────────────────────────────────────────────────────────┐
│                       EGUI FRONTEND                         │
│   (Immediate Mode - Draws screen 60fps - Pure Synchronous)  │
│                                                             │
│   [Chat UI]   [Contact Book Engine]    [Group Management]   │
└────────────┬───────────────────────────────────────▲────────┘
             │                                       │
  (mpsc::Sender<Command>)                 (mpsc::Receiver<AppEvent>)
   - SendDirectMessage                     - MessageReceived
   - CreateGroup                           - PeerOnlineStatus
   - AddContact                            - QueueFlushed
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

---

## 3. Module Design

### 3.1 Network Layer (`src/p2p/`)

**Purpose:** Manages dual-mode communication. Iroh direct connections for 1:1 chats, and Iroh Gossip swarms for Groups.

```rust
// File: src/p2p/mod.rs

/// Main networking backend struct
pub struct NetworkManager {
    endpoint: Endpoint,
    gossip: GossipApi,
}

impl NetworkManager {
    /// 1. Direct 1:1 Message (Iroh Unicast)
    /// Also used for securely transmitting Group symmetric keys during invites.
    pub async fn send_direct(&self, target: NodeId, payload: Vec<u8>) -> Result<()>;
    
    /// 2. Group Broadcast (Iroh Gossip)
    /// Broadcasts encrypted payload to all peers subscribed to the group TopicId.
    pub async fn broadcast_group(&self, topic: TopicId, payload: Vec<u8>) -> Result<()>;
    
    /// Join a Gossip swarm for a newly invited or created group.
    pub async fn subscribe_group(&self, topic: TopicId) -> Result<()>;
    
    /// Look up a peer's current network address via Pkarr.
    /// Pkarr resolves a public key to a NodeAddr — it is not a message transport layer.
    pub async fn discover_peer(&self, pk: PublicKey) -> Result<Option<NodeAddr>>;
}
```

---

### 3.2 Crypto Layer (`src/crypto/`)

**Purpose:** End-to-end encryption. Distinct approaches for 1:1 vs Groups.

**1:1 Direct Chat:**
We rely on Iroh's native Noise protocol for encrypted transport. For verifiable application-level E2EE (to satisfy academic requirements), we perform a static X25519 Diffie-Hellman Key Exchange to derive an initial ChaCha20-Poly1305 symmetric key. After each message is sent or received, we apply a **Hash Ratchet** (hashing the active session key via SHA-256) to guarantee Forward Secrecy — if the device is later compromised, past messages remain mathematically secure.

> **Scope note:** This is a SHA-256 hash ratchet, not a full Double Ratchet (as used by Signal). The Double Ratchet provides additional properties such as break-in recovery (healing after key compromise) that are beyond this project's scope. The hash ratchet alone satisfies the forward secrecy requirement.

**Group Chat:**
Because Signal-style Sender Keys are highly complex to coordinate in a serverless environment, groups use **Shared Symmetric Keys**. When Alice creates a group, her client generates a random 32-byte ChaCha20 key. She invites Bob by sending him the Group `TopicId` and the `SymmetricKey` via a secure **1:1 Direct E2EE message**. Once Bob holds the key, he can decrypt all Gossip messages broadcast to that group's topic.

> **Known limitation — group key rotation:** When a member is removed from a group, they still possess the symmetric key. In the current design, a removed member could theoretically decrypt messages if they intercept Gossip traffic for that topic. Production solutions generate a new group key and re-distribute it to remaining members (key rotation). For this project's scope, the limitation is acknowledged and documented. The architecture does not prevent key rotation from being added as a future extension.

```rust
// File: src/crypto/mod.rs

/// The user's cryptographic identity — generated once on first launch, stored locally.
pub struct Identity {
    pub node_id: NodeId,
    pub x25519_static: StaticSecret,
}

impl CryptoManager {
    // ---- 1:1 CRYPTO ---- 
    /// Derive a shared secret using our private key and their public key (X25519 DH).
    pub fn derive_direct_key(our_secret: &StaticSecret, their_public: &PublicKey) -> [u8; 32];
    
    /// Advance the session key for Forward Secrecy: next_key = SHA-256(current_key)
    pub fn ratchet_key(current_key: &mut [u8; 32]);
    
    /// Encrypt a payload for a single peer using the current ratchet key.
    pub fn encrypt_direct(payload: &[u8], shared_key: &[u8; 32]) -> Vec<u8>;
    
    /// Decrypt a payload received from a direct peer.
    pub fn decrypt_direct(ciphertext: &[u8], shared_key: &[u8; 32]) -> Result<Vec<u8>>;

    // ---- GROUP CRYPTO ----
    /// Generate a new random symmetric key for a new Group.
    pub fn generate_group_key() -> [u8; 32];
    
    /// Encrypt a payload for broadcast via Gossip using the group key.
    pub fn encrypt_group(payload: &[u8], group_key: &[u8; 32]) -> Vec<u8>;

    /// Decrypt a Gossip payload using the group key.
    pub fn decrypt_group(ciphertext: &[u8], group_key: &[u8; 32]) -> Result<Vec<u8>>;
}
```

---

### 3.3 Storage Layer (`src/storage/`)

**Purpose:** A unified SQLite database using `rusqlite`. This eliminates the complexity of writing custom binary queues. If a peer is offline when a message is sent, the message is stored locally with `status = 'queued'` and flushed automatically when the peer is reachable.

```sql
-- SQLite Schema

CREATE TABLE peers (
    node_id      TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    x25519_pubkey TEXT NOT NULL
);

CREATE TABLE groups (
    topic_id      TEXT PRIMARY KEY,
    group_name    TEXT NOT NULL,
    symmetric_key BLOB NOT NULL  -- stored encrypted at rest with local password
);

CREATE TABLE messages (
    id        TEXT PRIMARY KEY,
    type      TEXT NOT NULL,       -- 'text', 'image', 'file', 'group_invite'
    target_id TEXT NOT NULL,       -- NodeId (1:1) or TopicId (group)
    sender_id TEXT NOT NULL,
    content   BLOB NOT NULL,       -- Decrypted plaintext stored locally, or file path on disk
    timestamp INTEGER NOT NULL,
    status    TEXT NOT NULL        -- 'queued', 'sent', 'delivered', 'read'
);
```

---

### 3.4 Offline Message Delivery

**Purpose:** Define clearly how messages are guaranteed to reach an offline peer.

The delivery model operates in three states, tried in order:

**State 1 — Direct P2P (both nodes online):**
Alice's node connects to Bob's node directly via Iroh (with NAT hole punching, falling back to DERP relay if UDP is blocked). The message transmits in real time. Status updates to `sent` → `delivered`.

**State 2 — Local queue (Bob's node is offline, Alice stays online):**
The message is written to SQLite with `status = 'queued'`. The backend worker's periodic flush task (`flush_offline_queue`, running every 10 seconds) retries delivery automatically each time it fires. When Bob's node comes back online and Pkarr resolves his address, the queued message sends and the status updates.

**State 3 — DERP relay transport (NAT traversal blocked):**
If direct UDP hole punching fails, Iroh transparently falls back to its DERP (Designated Encrypted Relay Point) relay infrastructure. DERP relays act as a transport proxy — they forward encrypted packets between nodes but never possess the decryption key and cannot read message content. This is a transport fallback only, not a store-and-forward system.

**Acknowledged limitation — extended mutual offline:**
The local queue delivers messages only while Alice's node remains running. If both Alice and Bob are offline simultaneously and Alice's device powers off, the queued messages are held in Alice's local SQLite and will flush the next time Alice's node runs and Bob is reachable. There is no server holding messages in the interim. This is a deliberate scope boundary. A production extension would implement a DHT-based store-and-forward mailbox (using Pkarr's BEP44 mutable item storage) to provide true asynchronous delivery. This is documented as a future extension, not a flaw.

---

### 3.5 Core Engine (The Actor Worker) (`src/core/`)

**Purpose:** Ties Storage, Crypto, and Network together in a single asynchronous event loop.

```rust
// File: src/core/mod.rs

/// Commands sent FROM egui TO the Backend Worker
pub enum Command {
    SendDirectMessage  { target: NodeId, plaintext: String },
    SendFile           { target: NodeId, file_path: PathBuf },
    NotifyReadReceipt  { target: NodeId, message_id: Uuid },
    CreateGroup        { name: String },
    SendGroupMessage   { topic: TopicId, plaintext: String },
    InviteToGroup      { target: NodeId, topic: TopicId },
}

/// Events broadcast FROM the Backend Worker TO egui
pub enum AppEvent {
    IncomingMessage       { sender: NodeId, plaintext: String },
    IncomingFile          { sender: NodeId, file_name: String, path: PathBuf },
    MessageStatusUpdate   { id: Uuid, status: MessageStatus },
    IncomingGroupMessage  { topic: TopicId, sender: NodeId, plaintext: String },
    GroupInviteReceived   { topic: TopicId, group_name: String },
    PeerOnlineStatus      { peer: NodeId, online: bool },
}

pub struct NodeChatWorker {
    db:          rusqlite::Connection,
    network:     NetworkManager,
    crypto:      CryptoManager,
    rx_commands: mpsc::Receiver<Command>,
    tx_events:   broadcast::Sender<AppEvent>,
}

impl NodeChatWorker {
    /// Main asynchronous loop running in the background thread.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // 1. Listen for UI Commands
                Some(cmd) = self.rx_commands.recv() => {
                    self.handle_command(cmd).await;
                }
                // 2. Listen for Network Events (Iroh Unicast or Gossip)
                event = self.network.next_event() => {
                    self.handle_network(event).await;
                }
                // 3. Periodic tasks — retry queued messages for peers now reachable
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    self.flush_offline_queue().await;
                }
            }
        }
    }
}
```

---

### 3.6 Frontend GUI (`src/ui/`)

**Purpose:** Synchronous egui renderer. Completely detached from network blocking. Maintains only visual state compiled from backend events.

```rust
// File: src/ui/mod.rs

pub struct NodeChatUI {
    /// Channel to send commands to the backend worker
    tx_cmd: mpsc::Sender<Command>,
    /// Channel to receive events from the backend worker
    rx_event: broadcast::Receiver<AppEvent>,
    
    // Local UI state — built entirely from received AppEvents
    active_chat:   ActiveChat,
    messages_view: Vec<UIChatMessage>,
    contact_book:  Vec<UIPeer>,
}

impl eframe::App for NodeChatUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drain any incoming backend events non-blockingly
        while let Ok(event) = self.rx_event.try_recv() {
            self.update_state_from_event(event);
        }
        
        // 2. Draw UI panels
        egui::SidePanel::left("contacts").show(ctx, |ui| {
            // Contact list and group list
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Message thread view
            if ui.button("Send").clicked() {
                let _ = self.tx_cmd.try_send(Command::SendDirectMessage { ... });
            }
        });
    }
}
```

---

### 3.7 User Identity & Onboarding

**Purpose:** Zero-server identity generation and local security.

**Zero-Server Registration:**
On first launch, the app automatically generates an X25519/Ed25519 keypair. The public key becomes the permanent `NodeId`. No cloud registration, username, or email is required or used. Identity is mathematically owned by the user.

**Local Display Name:**
The user enters a Display Name locally. This name is only exchanged during P2P handshakes to populate a contact's local phonebook entry — it is never stored on any server.

**Local Password Encryption:**
To protect chat history and the private key if the device is stolen, the SQLite database and private keystore are optionally encrypted at rest using a user-supplied local password. The password is never transmitted anywhere. Decryption happens locally on startup.

**Contact Model:**
There is no centralized "add friend" system. Contacts are added by sharing a `NodeId` (the public key fingerprint) via any out-of-band channel — in person, via QR code, or via another messenger. The Contacts feature is a purely local SQLite phonebook mapping a `NodeId` to a human-readable display name.

---

## 4. File Structure

```
nodechat/
├── ARCHITECTURE.md           # This document
├── README.md                 # Project description and setup guide
├── Cargo.toml                # Workspace and dependency definitions
│
├── src/
│   ├── main.rs               # Entry point — boots Tokio runtime and egui window
│   │
│   ├── core/                 # Backend Worker (Actor Model loop)
│   │   ├── mod.rs            # NodeChatWorker implementation
│   │   └── commands.rs       # Command and AppEvent enum definitions
│   │
│   ├── p2p/                  # Network layer
│   │   ├── mod.rs            # NetworkManager struct
│   │   ├── direct.rs         # Iroh unicast connection handling
│   │   └── group.rs          # Iroh Gossip swarm handling
│   │
│   ├── crypto/               # Encryption layer
│   │   └── mod.rs            # Identity, key exchange, encrypt/decrypt, ratchet
│   │
│   ├── storage/              # Database layer
│   │   ├── mod.rs            # SQLite initialization and connection management
│   │   └── queries.rs        # All SQL CRUD operations
│   │
│   └── ui/                   # Frontend
│       ├── mod.rs            # NodeChatUI main loop and event processing
│       └── views.rs          # Individual panel renderers (contacts, chat, groups)
│
└── hello_android/            # Cross-platform Android build reference
```

---

## 5. Implementation Order

### Phase 1 — Core Foundation (Week 1–2)
- Set up Cargo workspace and verify dependency versions build cleanly.
- Wire up Tokio Actor Model: MPSC channels between main thread and background worker.
- Initialize the SQLite schema via rusqlite.
- Verify hello_android builds on target Android device (early risk mitigation).

### Phase 2 — Peer-to-Peer 1:1 Networking (Week 3–4)
- Bind the Iroh Endpoint in the backend worker.
- Implement Pkarr peer discovery (resolving a NodeId to a NodeAddr).
- Enable basic unicast data streams between two pre-configured NodeIds on a LAN.

### Phase 3 — Identity & E2EE Crypto (Week 5–6)
- Implement X25519 keypair generation and persistence.
- Wire X25519 DH exchange into ChaCha20-Poly1305 encrypt/decrypt for 1:1 messages.
- Implement the SHA-256 hash ratchet for Forward Secrecy.
- Implement the offline queue: write to SQLite on send failure, flush on reconnect.

### Phase 4 — UI Integration (Week 7–8)
- Bind the egui frontend to the backend via channels.
- Build the Contacts phonebook panel.
- Build the 1:1 Direct Chat panel with delivery status display.
- Build the onboarding flow (first launch keypair generation, display name, local password).

### Phase 5 — Group Chat Expansion (Week 9–10)
- Integrate iroh-gossip into the Network layer.
- Implement symmetric group key generation and storage.
- Implement group invite flow (transmit TopicId + key via secure 1:1 message).
- Build Group Chat UI panels.

### Phase 6 — Polish & Defense Prep (Week 11–12)
- End-to-end integration testing across two separate machines.
- Unit tests for all crypto functions and storage queries.
- Final documentation, README, and presentation slides.
- Android demo build if toolchain is stable.

---

## 6. Testing Plan

### Unit Tests
- **Crypto:** Verify ChaCha20 decrypt fails on altered ciphertext bits (authentication tag check). Verify X25519 key derivation is deterministic. Verify hash ratchet advances correctly.
- **Storage:** Verify messages with `status = 'queued'` are reliably written and retrieved. Verify status updates are applied correctly.

### Integration Tests
- Two isolated `NodeChatWorker` instances exchanging a `Command::SendDirectMessage` on localhost, verifying the receiving worker emits `AppEvent::IncomingMessage` with correct plaintext.
- Verify peer discovery using a local Pkarr instance.
- Offline queue test: send a message while recipient worker is stopped, start recipient, verify message flushes and arrives.

---

## 7. Defense Points

When presenting this project, emphasize:

1. **True Decentralization:** No server needed for messaging. Pkarr handles peer discovery (resolving public keys to addresses). Iroh handles encrypted transport. The only optional infrastructure is Iroh's DERP relay for NAT traversal — which relays encrypted packets it cannot decrypt.

2. **Actor Model Architecture:** The UI thread is explicitly decoupled from all networking and database IO via Tokio MPSC channels. This is a standard pattern for professional desktop applications. The UI renders at 60fps regardless of network conditions.

3. **Data Sovereignty:** 100% of user data — messages, contacts, group keys, identity — lives exclusively on the user's local device in an encrypted SQLite database. No third party has any access to any of it.

4. **Honest Offline Queue Model:** The local SQLite queue guarantees delivery when Alice's node is running and Bob eventually comes online. The system transparently documents the extended mutual-offline limitation and identifies DHT-based store-and-forward as the production extension path.

5. **Decentralized Groups:** P2P group chat is solved by combining iroh-gossip (broadcast transport) with a symmetric key securely distributed via 1:1 E2EE messages. No group server. No group admin infrastructure. Any group member can broadcast.

6. **Robust NAT Traversal:** Even on aggressive networks (university Wi-Fi blocking UDP hole punching), NodeChat falls back transparently to Iroh's DERP relay servers. Connectivity is never broken. The relay only ever touches encrypted ciphertext.

7. **Forward Secrecy (Hash Ratchet):** A SHA-256 hash ratchet advances the 1:1 session key after every message. Past message payloads remain secure even if the current key is later compromised. The scope boundary relative to the full Signal Double Ratchet is explicitly documented.

8. **Transparent Architectural Trade-offs:** The project explicitly acknowledges where scope-appropriate simplifications are made — hash ratchet over Double Ratchet, symmetric group keys over Sender Keys, local queue over DHT mailbox — and can articulate the full production solution for each. This demonstrates understanding of the full problem space beyond the implementation itself.

---

## 8. Design Decisions

### The Contact Model
**Why no central "Add Friend" system?**
In a truly decentralized P2P system, there is no central authority to manage friend relationships. Identity is a cryptographic keypair, not a database entry. The Contacts feature is a local SQLite phonebook — it maps a `NodeId` to a human-readable display name purely for convenience. Adding a contact means exchanging `NodeId` values out-of-band (QR code, in person, etc.), which is both more secure and more private than a central social graph.

### The UI Freezing Problem
**Why the Actor Model?**
egui is an immediate-mode GUI — the entire UI redraws up to 60 times per second. If network I/O, key exchange, or database writes happen on the main thread, even a 50ms delay freezes the entire application visibly. The Actor Model isolates all async work in a Tokio background thread. The UI thread only sends commands and reads events — it never blocks.

### Group E2EE
**Why Symmetric Keys for Groups instead of Signal's Sender Keys?**
Signal's Sender Key Distribution protocol is highly complex: each group member generates their own chain key, distributes it to all other members, and handles member join/leave events with re-keying. In a serverless environment with no guaranteed message ordering, coordinating this correctly is an unsolved engineering problem at this scale. A shared ChaCha20 symmetric key — securely delivered to each member via individual 1:1 E2EE messages — fully guarantees end-to-end encryption for the group channel while avoiding this complexity. The trade-off (no per-sender forward secrecy, no automatic key rotation on member removal) is explicitly acknowledged in section 3.2.

### Offline Delivery
**Why not implement a DHT mailbox?**
A DHT store-and-forward mailbox (e.g. using Pkarr's BEP44 mutable item storage) would provide true asynchronous delivery even when both sender and recipient are simultaneously offline. However, implementing a correct DHT storage layer with appropriate TTLs, encryption at rest, and spam protection adds significant complexity beyond the project's 12-week scope. The local SQLite queue provides delivery guarantees for all realistic scenarios where Alice's node is running — which covers the large majority of real-world usage patterns. The DHT mailbox is documented as the production extension path.

### Decentralized Onboarding
**How do users log in without a server?**
The user mathematically owns their identity. On first launch, the application generates a cryptographic keypair locally. The public key *is* the identity. To protect against device theft, the private key and local database are encrypted with an optional local master password. No server is ever involved in authentication — there is no account to hack, no password database to breach, no login endpoint to attack.

---

## 9. Known Limitations & Future Extensions

| Limitation | Current Behavior | Production Extension |
|---|---|---|
| Extended mutual offline | Messages queued locally; lost if Alice's node is off | DHT BEP44 store-and-forward mailbox |
| Group key rotation | Removed members retain old key | Re-key all remaining members on removal |
| Full forward secrecy | SHA-256 hash ratchet (one direction) | Full Signal Double Ratchet |
| Group member forward secrecy | Shared symmetric key | Signal Sender Keys per member |
| Android support | Stretch goal / toolchain risk | Full cargo-apk pipeline |

---

*Last Updated: 2026-03-30*  
*Project: NodeChat — Secure Decentralized Chat*  
*Version: 1.1 — Architecture finalized, limitations documented*