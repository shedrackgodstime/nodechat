# NodeChat - Secure Decentralized Chat Application

## Project Overview

**Topic:** Secure Decentralized Chat Messaging Application  
**Type:** School Final Year Project  
**Team:** 2 members

### Abstract

This project builds a peer-to-peer (P2P) chat application with end-to-end encryption (E2EE). Unlike traditional chat apps that rely on central servers, NodeChat uses decentralized networking so users communicate directly without a trusted server. The project adapts research from the Artemis P2P framework for secure human communication.

---

## 1. Technology Stack

### 1.1 Backend (Rust)

| Crate | Version | Purpose |
|-------|---------|---------|
| **iroh** | 0.97.0 | P2P networking, hole punching, encrypted connections |
| **iroh-gossip** | 0.97.0 | Broadcast messaging to multiple peers |
| **pkarr** | Latest | DNS-based decentralized peer discovery |
| **x25519-dalek** | 2.0 | X25519 key exchange for E2EE |
| **chacha20poly1305** | 0.10 | Authenticated encryption (AEAD) |
| **serde** | 1.0 | Serialization/deserialization |
| **tokio** | 1.x | Async runtime |
| **uuid** | 1.x | Unique message IDs |
| **anyhow** | 1.x | Error handling |

### 1.2 Frontend (Rust + egui)

Based on **hello_android** project structure:

| Crate | Version | Purpose |
|-------|---------|---------|
| **eframe** | 0.34 | GUI framework (hello_android uses this) |
| **egui** | 0.34 | UI toolkit |
| **egui_extras** | 0.34 | Extra UI components, images |

### 1.3 Platform Support

- **Desktop:** Windows, Linux, macOS (via eframe)
- **Mobile:** Android (via cargo-apk, as shown in hello_android)

---

## 2. System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (egui)                          │
│    Contact List │ Chat Window │ Settings │ File Transfer       │
├─────────────────────────────────────────────────────────────────┤
│                     APPLICATION LAYER                           │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│   │ ContactMgr   │  │ MessageMgr   │  │ SessionMgr   │          │
│   └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                     CRYPTO LAYER (E2EE)                         │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│   │ X3DH Key     │  │ Double       │  │ Session      │          │
│   │ Exchange     │  │ Ratchet      │  │ State        │          │
│   └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                     NETWORK LAYER (P2P)                         │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│   │ Iroh         │  │ Gossip       │  │ Discovery    │          │
│   │ Endpoint     │  │ (Broadcast) │  │ (Pkarr)     │          │
│   └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Module Design

### 3.1 Network Layer (`src/p2p/`)

**Purpose:** Handle all P2P communication - adapted from Artemis net.rs

#### Key Components:

```rust
// File: src/p2p/mod.rs

/// The main P2P backend that wraps Iroh networking
pub struct P2PBackend {
    /// Iroh Endpoint - manages connections
    endpoint: Endpoint,
    
    /// Gossip - for broadcasting to all connected peers
    gossip: GossipApi,
    
    /// Our unique NodeId (also serves as username/address)
    node_id: NodeId,
}

/// Topic for global chat (peers subscribe to this topic)
pub const CHAT_TOPIC: [u8; 32] = [
    0x6e, 0x6f, 0x64, 0x65, 0x63, 0x68, 0x61, 0x74,
    0x5f, 0x74, 0x6f, 0x70, 0x69, 0x63, 0x5f, 0x32,
    0x30, 0x32, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];
```

**API (to be implemented):**
```rust
impl P2PBackend {
    /// Create new P2P backend with optional secret key
    pub async fn new(secret_key: Option<SecretKey>) -> Result<Self>;
    
    /// Send message to specific peer (unicast)
    pub async fn send_to(&self, peer: NodeId, data: Vec<u8>) -> Result<()>;
    
    /// Send message to all connected peers (broadcast)
    pub async fn broadcast(&self, data: Vec<u8>) -> Result<()>;
    
    /// Send file to peer
    pub async fn send_file(&self, peer: NodeId, path: PathBuf) -> Result<()>;
    
    /// Get next incoming event (message, file, peer join/leave)
    pub async fn next_event(&self) -> Result<Option<NetworkEvent>>;
    
    /// Get our NodeId (used as our "username")
    pub fn node_id(&self) -> NodeId;
    
    /// Gracefully shutdown
    pub async fn shutdown(&self) -> Result<()>;
}
```

#### Iroh 0.97.0 API Changes (vs old 0.22):

| Old (0.22) | New (0.97) |
|------------|------------|
| `Endpoint::builder().discovery_n0()` | `Endpoint::bind(presets::N0)` |
| `Gossip::builder().spawn()` | `GossipApi::builder().spawn()` |
| `NodeId` | Same |
| `Router` | Simplified protocol handling |

---

### 3.2 Crypto Layer (`src/crypto/`)

**Purpose:** End-to-end encryption using simplified Signal Protocol

#### Simplified Design (for student project):

```
┌─────────────────────────────────────────────────────────┐
│                    CRYPTO MODULE                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  User Identity:                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │  NodeId (from Iroh)                              │   │
│  │  Chat Secret Key (X25519)                        │   │
│  │  Chat Public Key  (X25519)                       │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  Key Bundle (shared via Pkarr for discovery):          │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Public Key                                       │   │
│  │  Signed Pre-Key (SPK) - rotated weekly            │   │
│  │  One-Time Pre-Keys (OPK) - 100 keys              │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  Session (per contact):                                 │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Root Key          (derived from X3DH)            │   │
│  │  Sending Chain    (for encrypting messages)       │   │
│  │  Receiving Chain  (for decrypting messages)       │   │
│  │  Message Counter   (prevents replay attacks)      │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

#### Implementation:

```rust
// File: src/crypto/mod.rs

/// Represents a user's cryptographic identity
pub struct UserIdentity {
    /// Our NodeId (also our network address)
    pub node_id: NodeId,
    
    /// X25519 key pair for chat encryption
    pub keypair: x25519_dalek::StaticSecret,
    pub public_key: x25519_dalek::PublicKey,
    
    /// Signed pre-key for key exchange
    pub signed_prekey: SignedPreKey,
    
    /// One-time pre-keys (used once, then discarded)
    pub prekeys: Vec<OneTimePreKey>,
}

/// A session with a specific contact
pub struct Session {
    /// Their public key
    pub remote_public_key: x25519_dalek::PublicKey,
    
    /// Root key - base for deriving message keys
    pub root_key: [u8; 32],
    
    /// Sending chain key
    pub sending_chain_key: [u8; 32],
    
    /// Receiving chain key  
    pub receiving_chain_key: [u8; 32],
    
    /// Message counters (for replay protection)
    pub sending_count: u64,
    pub receiving_count: u64,
}

/// Encrypt/decrypt messages
pub struct CryptoManager;

impl CryptoManager {
    /// Generate new user identity
    pub fn generate_identity() -> UserIdentity;
    
    /// Start X3DH key exchange with a contact's key bundle
    pub fn x3dh_initiate(bundle: &KeyBundle) -> Result<(Session, Vec<u8>)>;
    
    /// Complete X3DH (called when we receive their response)
    pub fn x3dh_respond(ephemeral_public: &[u8]) -> Result<Session>;
    
    /// Encrypt a message using the session
    pub fn encrypt(session: &mut Session, plaintext: &[u8]) -> Result<Vec<u8>>;
    
    /// Decrypt a message using the session
    pub fn decrypt(session: &mut Session, ciphertext: &[u8]) -> Result<Vec<u8>>;
    
    /// Serialize session for storage
    pub fn serialize_session(session: &Session) -> Vec<u8>;
    
    /// Deserialize session from storage
    pub fn deserialize_session(data: &[u8]) -> Result<Session>;
}
```

#### Encryption Flow:

```
SENDER                                                  RECEIVER
   │                                                        │
   │  1. Get contact's key bundle (from Pkarr/DHT)          │
   │                                                        │
   │  2. X3DH: Generate ephemeral key                      │
   │     Compute shared secret with their SPK              │
   │                                                        │
   │  3. Derive root key from shared secret                │
   │                                                        │
   │  4. Derive first message key from root key             │
   │                                                        │
   │  5. Encrypt message: ChaCha20-Poly1305                 │
   │     ciphertext = encrypt(plaintext, message_key)       │
   │                                                        │
   │  6. Send: [ephemeral_pk] [ciphertext] ──────────────>   │
   │                                                        │  7. X3DH: Compute same shared secret
   │                                                        │  8. Derive root key
   │                                                        │  9. Derive message key
   │                                                        │ 10. Decrypt: ChaCha20-Poly1305
   │                                                        │     plaintext = decrypt(ciphertext, message_key)
   │                                                        │
   │  11. For next message: ratchet forward                │
   │      (both sides update chain keys)                   │
```

---

### 3.3 Protocol Layer (`src/protocol/`)

**Purpose:** Define message formats for chat communication

```rust
// File: src/protocol/mod.rs

/// All message types in NodeChat
#[derive(Serialize, Deserialize)]
pub enum ChatMessage {
    /// First contact: share our key bundle
    Handshake {
        /// Our NodeId
        node_id: NodeId,
        /// Our public key for E2EE
        public_key: PublicKey,
        /// Signed pre-key
        signed_prekey: SignedPreKey,
        /// One-time pre-keys (all of them for initial sync)
        prekeys: Vec<OneTimePreKey>,
    },
    
    /// Encrypted chat message
    Encrypted {
        /// UUID of this message
        id: Uuid,
        /// Sender's NodeId
        sender: NodeId,
        /// Unix timestamp
        timestamp: i64,
        /// The encrypted payload (E2EE)
        payload: Vec<u8>,
        /// Message type (text, image, file offer)
        message_type: MessageType,
    },
    
    /// Request to start a chat (initiates X3DH)
    ChatRequest {
        /// Who we want to chat with
        target: NodeId,
        /// Our public key
        public_key: PublicKey,
    },
    
    /// Acknowledge chat request
    ChatAccept {
        /// Our public key
        public_key: PublicKey,
        /// X3DH response data
        x3dh_data: Vec<u8>,
    },
    
    /// File transfer offer
    FileOffer {
        id: Uuid,
        filename: String,
        size: u64,
        hash: [u8; 32],
    },
    
    /// Request to download file
    FileRequest {
        id: Uuid,
    },
}

/// Types of messages
#[derive(Serialize, Deserialize, Clone)]
pub enum MessageType {
    Text,
    Image,
    File,
    System,  // "User joined", "User left", etc.
}
```

---

### 3.4 Storage Layer (`src/storage/`)

**Purpose:** Store contacts, messages, and sessions locally

```rust
// File: src/storage/mod.rs

/// Contact information
#[derive(Serialize, Deserialize)]
pub struct Contact {
    /// Their NodeId (also their network address)
    pub node_id: NodeId,
    
    /// Display name (user-chosen, not verified)
    pub display_name: String,
    
    /// Their public key for E2EE
    pub public_key: PublicKey,
    
    /// When we first added this contact
    pub added_at: i64,
    
    /// Last message timestamp
    pub last_seen: i64,
    
    /// Session state (encrypted)
    #[serde(skip)]
    pub session: Option<Session>,
}

/// Chat message history
#[derive(Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: Uuid,
    pub sender: NodeId,
    pub recipient: NodeId,
    pub timestamp: i64,
    pub content: String,  // Decrypted content
    pub message_type: MessageType,
}

/// Storage manager
pub struct Storage {
    /// Path to data directory
    data_dir: PathBuf,
}

impl Storage {
    /// Initialize storage
    pub fn new() -> Result<Self>;
    
    /// Save contact
    pub fn save_contact(&self, contact: &Contact) -> Result<()>;
    
    /// Load all contacts
    pub fn load_contacts(&self) -> Result<Vec<Contact>>;
    
    /// Save message
    pub fn save_message(&self, msg: &StoredMessage) -> Result<()>;
    
    /// Load messages for a contact
    pub fn load_messages(&self, contact: &NodeId) -> Result<Vec<StoredMessage>>;
    
    /// Save session state
    pub fn save_session(&self, node_id: &NodeId, session: &Session) -> Result<()>;
    
    /// Load session state
    pub fn load_session(&self, node_id: &NodeId) -> Result<Option<Session>>;
}
```

---

### 3.5 Application Layer (`src/app/`)

**Purpose:** Tie everything together - handles UI events and coordinates modules

```rust
// File: src/app/mod.rs

/// Main application state
pub struct App {
    /// P2P networking
    p2p: P2PBackend,
    
    /// Cryptography
    crypto: CryptoManager,
    
    /// Our identity
    identity: UserIdentity,
    
    /// Contact list
    contacts: Vec<Contact>,
    
    /// Current chat (selected contact)
    active_chat: Option<NodeId>,
    
    /// Message history (for active chat)
    messages: Vec<StoredMessage>,
    
    /// Storage
    storage: Storage,
    
    /// Application state
    state: AppState,
}

#[derive(Default)]
pub enum AppState {
    #[default]
    Starting,
    LoadingIdentity,
    Ready,
    Connecting,    // Connecting to network
    Error(String),
}

impl App {
    /// Initialize the app
    pub async fn new() -> Result<Self>;
    
    /// Process incoming network events
    pub async fn handle_network_event(&mut self, event: NetworkEvent);
    
    /// Send a message
    pub async fn send_message(&mut self, content: String) -> Result<()>;
    
    /// Add a new contact
    pub async fn add_contact(&mut self, node_id: NodeId, display_name: String) -> Result<()>;
    
    /// Select a chat
    pub fn select_chat(&mut self, node_id: NodeId);
}
```

---

### 3.6 Frontend (egui)

**Reference:** Based on `hello_android` project structure

```
hello_android structure:
├── src/
│   ├── main.rs        →  Entry point with eframe::run_native
│   └── lib.rs         →  MyApp struct implementing eframe::App
│                       
│   UI uses:
│   - egui::CentralPanel
│   - egui::SidePanel (for contacts)
│   - egui::TextEdit  (for input)
│   - egui::Button    (for actions)
```

#### NodeChat UI Layout:

```rust
// File: src/ui/mod.rs

use eframe::{egui, App, Frame};

pub struct NodeChatUI {
    /// The backend app
    app: App,
}

impl NodeChatUI {
    pub fn new(app: App) -> Self;
}

impl App for NodeChatUI {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        // Left panel: Contact list
        egui::SidePanel::left("contacts")
            .min_width(200.0)
            .show_inside(ui, |ui| {
                ui.heading("Contacts");
                ui.separator();
                
                for contact in &self.app.contacts {
                    let selected = self.app.active_chat == Some(contact.node_id);
                    if ui.selectable_label(selected, &contact.display_name).clicked() {
                        self.app.select_chat(contact.node_id);
                    }
                }
            });
        
        // Center panel: Chat window
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                if let Some(active) = &self.app.active_chat {
                    // Messages
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for msg in &self.app.messages {
                            // Display message
                        }
                    });
                    
                    // Input area at bottom
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text("Type a message...")
                        .show(ui);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a contact to start chatting");
                    });
                }
            });
    }
}
```

---

## 4. File Structure

```
nodechat/
├── ARCHITECTURE.md           # This file
├── README.md                 # Project description
├── Cargo.toml               # Rust dependencies
├── SPEC.md                   # Detailed specification
│
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Module exports
│   │
│   ├── p2p/                 # Network layer (adapted from artemis)
│   │   ├── mod.rs           # P2PBackend definition
│   │   ├── endpoint.rs      # Iroh Endpoint management
│   │   ├── gossip.rs        # Broadcast messaging
│   │   └── discovery.rs     # Pkarr peer discovery
│   │
│   ├── crypto/              # Encryption layer
│   │   ├── mod.rs           # CryptoManager, UserIdentity
│   │   ├── x3dh.rs          # Key exchange
│   │   ├── ratchet.rs       # Double ratchet
│   │   └── session.rs       # Session management
│   │
│   ├── protocol/            # Message formats
│   │   └── mod.rs           # ChatMessage, MessageType
│   │
│   ├── storage/             # Local data storage
│   │   └── mod.rs           # Storage, Contact, StoredMessage
│   │
│   ├── app/                 # Application logic
│   │   └── mod.rs           # App struct and methods
│   │
│   └── ui/                  # Frontend (egui)
│       ├── mod.rs           # NodeChatUI
│       └── widgets.rs       # Custom UI components
│
└── hello_android/           # Reference for UI (don't modify)
    ├── src/
    │   ├── main.rs
    │   └── lib.rs
    └── Cargo.toml
```

---

## 5. Implementation Order

### Phase 1: Foundation (Week 1-2)
- [ ] Set up project structure with Cargo.toml
- [ ] Implement basic P2P networking with Iroh 0.97
- [ ] Get peer discovery working (Pkarr)
- [ ] Test basic send/receive

### Phase 2: Encryption (Week 3-4)
- [ ] Implement X25519 key generation
- [ ] Implement ChaCha20-Poly1305 encryption
- [ ] Implement simplified X3DH key exchange
- [ ] Implement session management

### Phase 3: Chat Protocol (Week 5-6)
- [ ] Define message types (Handshake, Encrypted, etc.)
- [ ] Handle contact addition
- [ ] Handle message send/receive flow
- [ ] Message history storage

### Phase 4: User Interface (Week 7-8)
- [ ] Set up eframe (based on hello_android)
- [ ] Contact list panel
- [ ] Chat window
- [ ] Message input
- [ ] Settings/identity display

### Phase 5: Features (Week 9-10)
- [ ] File transfer
- [ ] Online/offline status
- [ ] Message delivery status

### Phase 6: Testing & Documentation (Week 11-12)
- [ ] Unit tests
- [ ] Integration tests
- [ ] README and documentation
- [ ] Prepare for defense presentation

---

## 6. Key Differences from Artemis

| Artemis (Research) | NodeChat (Student Project) |
|--------------------|---------------------------|
| Stealth/persistence features | Removed - chat app needs no persistence tricks |
| Machine-to-machine (C2) | Human-to-human (chat) |
| Transport encryption only | Full E2EE (Signal-style) |
| Complex anti-analysis | None - legitimate app |
| Iroh 0.22 | Iroh 0.97.0 (updated API) |
| No UI | egui frontend |

---

## 7. Testing Plan

### Unit Tests
- Crypto functions (encrypt/decrypt)
- Session serialization
- Message parsing

### Integration Tests
- Two nodes exchanging messages
- Contact addition flow
- File transfer

### Manual Tests
- UI interactions
- Cross-platform (if time permits)

---

## 8. Defense Points

When presenting this project, emphasize:

1. **Decentralization:** No central server - uses Pkarr for peer discovery
2. **End-to-End Encryption:** Each message is encrypted with unique keys
3. **Forward Secrecy:** Compromised keys can't decrypt past messages
4. **Real P2P:** Direct peer-to-peer communication using Iroh
5. **Research Foundation:** Built on Artemis P2P research

---

## 9. References

- [Iroh Documentation](https://docs.iroh.computer/)
- [Iroh GitHub](https://github.com/n0-computer/iroh)
- [Signal Protocol](https://signal.org/docs/)
- [hello_android (UI reference)](../hello_android/)
- [Artemis P2P research](../artemis/src/)

---

## 10. Questions for Supervisor

1. Is the simplified X3DH acceptable, or should we implement full X3DH?
2. Should we include group chats?
3. What's the expected message storage duration?
4. Should we include safety number verification UI?

---

*Last Updated: 2026-03-29*
*Project: NodeChat - Secure Decentralized Chat*
