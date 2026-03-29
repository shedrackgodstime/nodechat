# NodeChat - Secure Decentralized Chat Application

## Project Overview

**Topic:** Secure Decentralized Chat Messaging Application  
**Type:** School Final Year Project  
**Team:** 2 members

### Abstract

This project builds a peer-to-peer (P2P) chat application with end-to-end encryption (E2EE) supporting both 1:1 direct communication and Decentralized Group Chat. Unlike traditional chat apps that rely on central servers, NodeChat uses decentralized networking via Iroh and Pkarr so users communicate directly. The system utilizes an Event-Driven Actor Model to decouple the immediate-mode UI from the asynchronous networking and cryptographic and storage operations.

---

## 1. Technology Stack

### 1.1 Backend (Rust)

| Crate | Version | Purpose |
|-------|---------|---------|
| **iroh** | 0.97.0 | P2P networking, hole punching, secure connections |
| **iroh-gossip** | 0.97.0 | Broadcast messaging across P2P swarms (Group Chats) |
| **pkarr** | Latest | DNS-based decentralized peer discovery |
| **x25519-dalek** | 2.0 | X25519 key exchange for 1:1 E2EE |
| **chacha20poly1305** | 0.10 | Authenticated encryption (AEAD) for 1:1 and Groups |
| **sha2** | 0.10 | SHA-256 hash ratcheting for Forward Secrecy |
| **rusqlite** | 0.31 | Embedded SQLite for robust storage (Queue, History, Contacts) |
| **tokio** | 1.x | Async runtime and MPSC Channels for Actor Model |
| **uuid** | 1.x | Unique message IDs |
| **anyhow** | 1.x | Error handling |

### 1.2 Frontend (Rust + egui)

Based on **hello_android** project structure for cross-platform support:

| Crate | Version | Purpose |
|-------|---------|---------|
| **eframe** | 0.34 | GUI framework |
| **egui** | 0.34 | Immediate-mode UI toolkit |
| **egui_extras** | 0.34 | Extra UI components, images |

### 1.3 Platform Support

- **Desktop:** Windows, Linux, macOS (via eframe)
- **Mobile:** Android (via cargo-apk, building off hello_android)

---

## 2. System Architecture (Actor Model)

To prevent the application UI from freezing during heavy network operations, cryptographic key exchanges, or database writes, the architecture utilizes a strictly decoupled **Actor Model**. 

The UI (Frontend) never directly touches the Database or Network. It sends `Command` events down a channel to the Backend Worker, and dynamically updates its views based on `AppEvent` responses broadcasted from the worker.

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
    relay_client: reqwest::Client,
}

impl NetworkManager {
    /// 1. Direct 1:1 Message (Iroh Unicast)
    /// Also used for securely transmitting Group asymmetric keys.
    pub async fn send_direct(&self, target: NodeId, payload: Vec<u8>) -> Result<()>;
    
    /// 2. Group Broadcast (Iroh Gossip)
    /// Broadcasts encrypted payload to all peers subscribed to the group TopicId.
    pub async fn broadcast_group(&self, topic: TopicId, payload: Vec<u8>) -> Result<()>;
    
    /// Join a Gossip swarm for a newly invited/created group
    pub async fn subscribe_group(&self, topic: TopicId) -> Result<()>;
    
    /// Look up a peer via Pkarr
    pub async fn discover_peer(&self, pk: PublicKey) -> Result<Option<NodeAddr>>;
}
```

---

### 3.2 Crypto Layer (`src/crypto/`)

**Purpose:** End-to-end encryption. Distinct approaches for 1:1 vs Groups.

*   **1:1 Direct Chat:** We rely on Iroh's native Noise protocol for encrypted transport. For verifiable application-level E2EE (to satisfy academic requirements), we perform a static X25519 Diffie-Hellman Key Exchange to derive an initial ChaCha20-Poly1305 symmetric key. Crucially, after each message is sent or received, we implement a **Hash Ratchet** (hashing the active session key via SHA-256) to guarantee Forward Secrecy—if the device is later compromised, past messages remain mathematically secure.
*   **Group Chat:** Because Signal-style Sender Keys are highly complex, groups use **Shared Symmetric Keys**. When Alice creates a group, her client generates a random 32-byte key. She invites Bob by sending him the Group `TopicId` and the `Symmetric Key` via a secure **1:1 Direct E2EE message**. Once Bob has the key, he can decrypt Gossip messages heavily distributed across the network.

```rust
// File: src/crypto/mod.rs

/// The user's cryptographic identity
pub struct Identity {
    pub node_id: NodeId,
    pub x25519_static: StaticSecret,
}

impl CryptoManager {
    // ---- 1:1 CRYPTO ---- 
    /// Generate a shared secret for 1:1 E2EE using our private key and their public key
    pub fn derive_direct_key(our_secret: &StaticSecret, their_public: &PublicKey) -> [u8; 32];
    
    /// Advance the session key for Forward Secrecy: next_key = SHA-256(current_key)
    pub fn ratchet_key(current_key: &mut [u8; 32]);
    
    /// Encrypt a payload intended for a single peer
    pub fn encrypt_direct(payload: &[u8], shared_key: &[u8; 32]) -> Vec<u8>;
    
    // ---- GROUP CRYPTO ----
    /// Generate a new symmetric key for a new Group
    pub fn generate_group_key() -> [u8; 32];
    
    /// Encrypt a payload intended for broadcast via Gossip
    pub fn encrypt_group(payload: &[u8], group_key: &[u8; 32]) -> Vec<u8>;
}
```

---

### 3.3 Storage Layer (`src/storage/`)

**Purpose:** A unified SQLite database using `rusqlite`. This completely eliminates the complexity of writing custom JSON queues. If a peer is offline, the message is stored with `status = 'queued'`.

```sql
-- SQLite Schema

CREATE TABLE peers (
    node_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    x25519_pubkey TEXT NOT NULL
);

CREATE TABLE groups (
    topic_id TEXT PRIMARY KEY,
    group_name TEXT NOT NULL,
    symmetric_key BLOB NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,           -- 'text', 'image', 'file', 'group_invite'
    target_id TEXT NOT NULL,      -- NodeId OR TopicId
    sender_id TEXT NOT NULL, 
    content BLOB NOT NULL,        -- Decrypted plaintext locally OR file path on disk
    timestamp INTEGER NOT NULL,
    status TEXT NOT NULL          -- 'queued', 'sent', 'delivered', 'read'
);
```

---

### 3.4 Core Engine (The Actor Worker) (`src/core/`)

**Purpose:** Ties Storage, Crypto, and Network together in a single asynchronous event loop.

```rust
// File: src/core/mod.rs

/// Commands sent FROM egui TO the Backend Worker
pub enum Command {
    SendDirectMessage { target: NodeId, plaintext: String },
    SendFile { target: NodeId, file_path: PathBuf },
    NotifyReadReceipt { target: NodeId, message_id: Uuid },
    CreateGroup { name: String },
    SendGroupMessage { topic: TopicId, plaintext: String },
    InviteToGroup { target: NodeId, topic: TopicId },
}

/// Events broadcast FROM the Backend Worker TO egui
pub enum AppEvent {
    IncomingMessage { sender: NodeId, plaintext: String },
    IncomingFile { sender: NodeId, file_name: String, path: PathBuf },
    MessageStatusUpdate { id: Uuid, status: MessageStatus }, // Read/Delivered
    IncomingGroupMessage { topic: TopicId, sender: NodeId, plaintext: String },
    GroupInviteReceived { topic: TopicId, group_name: String },
}

pub struct NodeChatWorker {
    db: rusqlite::Connection,
    network: NetworkManager,
    crypto: CryptoManager,
    rx_commands: mpsc::Receiver<Command>,
    tx_events: broadcast::Sender<AppEvent>,
}

impl NodeChatWorker {
    /// Main asynchronous loop running in the background
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
                // 3. Periodic tasks (e.g. queue flushing)
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    self.flush_offline_queue().await;
                }
            }
        }
    }
}
```

---

### 3.5 Frontend GUI (`src/ui/`)

**Purpose:** Synchronous egui renderer. Completely detached from network blocking. Maintained strictly for visual state compilation.

```rust
// File: src/ui/mod.rs

pub struct NodeChatUI {
    /// Channel to tell the backend to do things
    tx_cmd: mpsc::Sender<Command>,
    /// Channel to hear what the backend did
    rx_event: broadcast::Receiver<AppEvent>,
    
    // UI State compiled from events
    active_chat: ActiveChat,
    messages_view: Vec<UIChatMessage>,
    contact_book: Vec<UIPeer>,
}

impl eframe::App for NodeChatUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process any incoming events from backend non-blockingly
        while let Ok(event) = self.rx_event.try_recv() {
            self.update_state_from_event(event);
        }
        
        // 2. Draw UI
        egui::SidePanel::left("contacts").show(ctx, |ui| {
             // Draw contacts/groups
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
             // Draw messages
             if ui.button("Send").clicked() {
                 let _ = self.tx_cmd.try_send(Command::SendDirectMessage { ... });
             }
        });
    }
}
```

---

### 3.6 User Identity & Onboarding

**Purpose:** Handles zero-knowledge identity generation and local security.

*   **Zero-Server Registration:** Upon first launch, the app automatically generates an X25519/Ed25519 keypair. The public key forms the permanent `NodeId`. There is no cloud registration, username, or email required.
*   **Local Display Name:** The user enters a Display Name locally, which is only used to populate friends' local phonebooks during P2P handshakes.
*   **Local Password Encryption:** To secure the chat history and identity on the physical device, the SQLite database and Private Key are optionally encrypted at rest. The user decrypts them locally via password upon app startup.

---

## 4. File Structure

```
nodechat/
├── ARCHITECTURE.md           # This file
├── README.md                 # Project description
├── Cargo.toml                # Rust dependencies
│
├── src/
│   ├── main.rs               # Entry point (boots Tokio + egui)
│   │
│   ├── core/                 # Backend Worker (Actor Model loop)
│   │   ├── mod.rs            
│   │   └── commands.rs       # Channels interface definitions
│   │
│   ├── p2p/                  # Network layer 
│   │   ├── mod.rs            # NetworkManager
│   │   ├── direct.rs         # Iroh Unicast handling
│   │   └── group.rs          # Iroh Gossip handling
│   │
│   ├── crypto/               # Encryption layer
│   │   ├── mod.rs            # Keys and Encryption logic
│   │
│   ├── storage/              # Database Layer
│   │   ├── mod.rs            # SQLite initialization
│   │   └── queries.rs        # SQL CRUD ops
│   │
│   └── ui/                   # Frontend
│       ├── mod.rs            # NodeChatUI main loop
│       └── views.rs          # Specific panel renderers
│
└── hello_android/            # Reference for cross-platform
```

---

## 5. Implementation Order

### Phase 1: Core Foundation (Week 1-2)
- Set up Cargo workspace and Actor Model Channels (Tokio <-> Main Thread).
- Initialize the embedded SQLite Database schema.
- Implement the basic Tokio background worker scaffolding.

### Phase 2: Peer-to-Peer 1:1 Networking (Week 3-4)
- Wire up Iroh Endpoint binding on the backend.
- Implement Pkarr Discovery.
- Enable basic Unicast data-streams between two pre-configured NodeIds.

### Phase 3: Identity & E2EE Crypto (Week 5-6)
- Implement static X25519 identity generation.
- Combine the X25519 DH-Exchange with ChaCha20 to encrypt 1:1 payloads.
- Implement Offline Queue execution flow via SQLite (`status = 'queued'`).

### Phase 4: UI Integration (Week 7-8)
- Bind the egui frontend to via the `rx`/`tx` channels.
- Build the Local Phonebook view.
- Build the 1:1 Direct Chat view showing queue statuses.

### Phase 5: Group Chat Expansion (Week 9-10)
- Add entirely new `iroh-gossip` router to the Network Layer.
- Implement Symmetric Group Key Generation.
- Create UI for creating groups and sending secure "Invites" via 1:1 E2EE.

### Phase 6: Polish & Testing (Week 11-12)
- Mobile execution tests (via `cargo-apk`).
- Final documentation and Presentation/Defense prep.

---

## 6. Testing Plan

### Unit Tests
- Crypto functions (Ensure ChaCha20 payloads fail on altered bits).
- SQLite storage tests (Ensuring offline queued messages are reliably written).

### Integration Tests
- Two isolated Backend Workers exchanging a simulated `Command` event without a UI.
- Verify node discovery using an isolated Pkarr instance.

---

## 7. Defense Points

When presenting this project, emphasize:

1. **True Decentralization:** No server needed. Pkarr for discovery, Iroh for transport.
2. **Actor Model Architecture:** Explicitly decoupled the UI thread from the intense networking and database IO, standard for professional desktop applications.
3. **Data Sovereignty:** The use of SQLite embedded locally means 100% of user data and contact directories live exclusively on the user's local disk.
4. **Smart Offline Queueing:** Messages are stored locally via SQL and flush to the network autonomously when peers are discovered online.
5. **Decentralized Groups:** Solved the complex problem of P2P Group chatting by utilizing Gossip protocols combined with securely distributed symmetric keys.
6. **Robust NAT Traversal:** Even on highly aggressive university Wi-Fi networks where peer-to-peer UDP hole-punching is blocked, NodeChat falls back to Iroh's native DERP relay servers, ensuring messages always deliver globally.
7. **Forward Secrecy (Hash Ratcheting):** While the full Signal Protocol is out-of-scope, implementing a SHA-256 key ratchet after every 1:1 message proves cryptographic maturity, ensuring past message payloads remain secure even if current keys are compromised.

---

## 8. Design Decisions (Answered)

### The Contact Model
**Why no central "Add Friend" system?**
In a truly decentralized P2P system, there is no central authority to manage friends. Much like Bitcoin, you communicate strictly using a cryptographic identifier (`NodeId`). The "Contacts" feature is intentionally designed as a purely local "Phonebook" mapped via SQLite, simply assigning a human-readable display name to a `NodeId` for user convenience.

### The UI Freezing Problem
**Why the complex Actor Model?**
If the application uses an Immediate-Mode GUI (`egui`), the UI must redraw entirely up to 60 times a second. If network packets drop, or cryptographic keys take 50ms to generate, executing this on the main thread would freeze the application completely. The Actor Model separates concerns perfectly.

### Group E2EE
**Why Symmetric Keys for Groups instead of Signal's Sender Keys?**
While 1:1 chat utilizes full Asymmetric Key encryption, Group Chat via a public Gossip swarm poses immense cryptographic coordination challenges in a serverless environment. Sharing a Symmetric AES/ChaCha20 key via a secure 1:1 backend message to group participants avoids immense P2P architectural complexity while fully guaranteeing End-to-End Encryption for the group swarm itself.

### Decentralized Onboarding
**How do users login without a server?**
True decentralization means the user mathematically owns their identity. Instead of a server assigning a user ID, the application generates a cryptographic Keypair on first launch. The public component *is* their identity. To protect the user if their device is stolen, the local database and private keystore are encrypted with an optional local master password, meaning no server is ever required to verify access.

---

*Last Updated: 2026-03-29*
*Project: NodeChat - Secure Decentralized Chat*