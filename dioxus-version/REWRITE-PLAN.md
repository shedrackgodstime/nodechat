# NodeChat Dioxus Rewrite — Architecture Plan

## Overview

Rewrite the Dioxus mockup into a clean architecture wired to a real P2P backend using
iroh 1.0.0+ for transport, iroh-gossip for group messaging, and rusqlite for local
storage. Channel-based bridge (mpsc) connects the Dioxus UI to a Tokio backend.

---

## Target Versions

| Crate | Version |
|---|---|
| `iroh` | `1.0.1` |
| `iroh-gossip` | `0.101.0` |
| `iroh-tickets` | `1.0.0` |
| `iroh-blobs` | `0.103.0` |
| `dioxus` | `0.7.1` (features: `router`) |
| `tokio` | `1` (rt-multi-thread, sync, time, macros, io-util) |
| `rusqlite` | `0.31` (features: `bundled`) |
| `x25519-dalek` | `2` (features: `static_secrets`) |
| `chacha20poly1305` | `0.10` |
| `sha2` | `0.10` |
| `rand` | `0.9` |
| `serde` / `serde_json` | `1` |
| `tracing` / `tracing-subscriber` | `0.1` / `0.3` |
| `thiserror` | `2` |
| `anyhow` | `1` |

---

## Module Structure

```
src/
  main.rs            Entry point — init tracing, spawn backend, launch Dioxus app
  lib.rs             Re-exports
  app.rs             App root — Dioxus Router with route enum

  contract/
    mod.rs           Domain types
    command.rs       Command enum (UI → Backend)
    event.rs         Event enum (Backend → UI)
    types.rs         Message, Contact, Group, Identity, ChatPreview, etc.

  bridge/
    mod.rs           Bridge struct — owns mpsc senders/receivers
    spawn.rs         spawn_backend() — starts Tokio task, returns Bridge handle

  state/
    mod.rs           AppState struct — Dioxus Signals, derived computations
    signals.rs       Signal helpers, selectors, memoized values

  backend/
    mod.rs           Backend struct — coordinator
    storage.rs       SQLite schema + CRUD (rusqlite, bundled)
    network.rs       Iroh endpoint + gossip + connection management
    crypto.rs        ChaCha20-Poly1305 AEAD, X25519 ECDH, key derivation
    commands.rs      Command handler dispatch
    events.rs        Network event handler (incoming messages, sync, receipts)
    identity.rs      Identity generation, persistence, peer resolution

  transport/
    mod.rs           Transport abstraction layer
    protocol.rs      Wire frames: HandshakeFrame, MessageFrame, GroupFrame, SyncFrame
    direct.rs        Direct 1-to-1 messaging over iroh connections
    group.rs         Gossip-based group messaging (subscribe, broadcast, leave)

  components/
    mod.rs           Component module declarations
    sidebar.rs       Chat/contact list sidebar
    chat_pane.rs     Message thread view
    dashboard.rs     Empty-state dashboard
    onboarding.rs    First-run wizard
    lock_screen.rs   PIN lock screen
    info_drawer.rs   Contact/group info slide-out
    modals.rs        Create group, add contact modals
    settings_tab.rs  Settings panel
    diagnostics_tab.rs  Debug log viewer

  routes.rs          Route enum — Onboarding, Chat/{id}, Settings, etc.
```

---

## Bridge Pattern (mpsc Channel)

### Flow

```
Dioxus UI ──Command──▶ Bridge ──mpsc──▶ Backend
Dioxus UI ◀──Event──── Bridge ◀──mpsc──── Backend
```

### Command Enum (UI → Backend)

```rust
pub enum Command {
    // Identity
    GenerateIdentity { display_name: String },
    LoadIdentity,

    // Contacts
    AddContact { ticket: String },
    RemoveContact { peer_id: String },

    // Direct messages
    SendDirectMessage { peer_id: String, plaintext: String },

    // Groups
    CreateGroup { name: String, members: Vec<String> },
    JoinGroup { ticket: String },
    SendGroupMessage { group_id: String, plaintext: String },

    // Sync
    SyncRequest { peer_id: String },

    // Snapshot
    RequestSnapshot,
}
```

### Event Enum (Backend → UI)

```rust
pub enum Event {
    // Identity
    IdentityLoaded { identity: IdentityView },
    IdentityCreated { identity: IdentityView },

    // Contacts
    ContactAdded { contact: ContactView },
    ContactRemoved { peer_id: String },

    // Messages
    DirectMessageReceived { message: MessageView },
    GroupMessageReceived { message: MessageView },
    MessageDelivered { message_id: String },
    MessageFailed { message_id: String, reason: String },

    // Groups
    GroupCreated { group: GroupView },
    GroupJoined { group: GroupView },

    // Network
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    NetworkError { message: String },

    // Snapshot
    SnapshotLoaded { snapshot: AppSnapshot },
}
```

### Bridge Implementation

```rust
pub struct Bridge {
    cmd_tx: mpsc::Sender<Command>,
    evt_rx: mpsc::Receiver<Event>,
}

impl Bridge {
    pub async fn send(&self, cmd: Command) -> Result<()> {
        self.cmd_tx.send(cmd).await.map_err(|_| Error::ChannelClosed)
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.evt_rx.recv().await
    }
}
```

---

## State Layer (Dioxus Signals)

```rust
#[derive(Clone)]
pub struct AppState {
    // Identity
    pub identity: Signal<Option<IdentityView>>,
    pub is_unlocked: Signal<bool>,

    // Data
    pub chats: Signal<Vec<ChatPreview>>,
    pub active_chat: Signal<Option<String>>,
    pub messages: Signal<HashMap<String, Vec<MessageView>>>,

    // UI
    pub is_loading: Signal<bool>,
    pub error: Signal<Option<String>>,

    // Bridge handle (wrapped in Signal for Dioxus)
    pub bridge: Signal<Option<Bridge>>,
}
```

---

## Routing

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/onboarding")]
    Onboarding,
    #[route("/lock")]
    Lock,
    #[route("/chat/:chat_id")]
    Chat { chat_id: String },
    #[route("/settings")]
    Settings,
    #[route("/diagnostics")]
    Diagnostics,
}
```

---

## Storage Schema (SQLite)

```sql
CREATE TABLE identity (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    secret_key BLOB NOT NULL,
    public_key BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE peers (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    public_key BLOB NOT NULL,
    ticket TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    added_at INTEGER NOT NULL
);

CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    gossip_topic BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE group_members (
    group_id TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, peer_id)
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    chat_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'direct',
    status TEXT NOT NULL DEFAULT 'sent',
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_messages_chat ON messages(chat_id, created_at);
```

---

## Iroh Integration Notes

- Use `iroh::Endpoint::builder(presets::N0)` for endpoint setup
- `EndpointAddr` for peer addressing (distinct from `NodeAddr`)
- `iroh::SecretKey::generate()` for identity keys
- `RelayMode::N0` for relay configuration
- `iroh_tickets::endpoint::EndpointTicket` for human-readable tickets
- iroh-gossip `0.101.0` with `features = ["net"]` for group topics

---

## Platform Strategy

1. **Desktop** (Linux, Windows) — primary target, full feature set
2. **Android** — Dioxus mobile, same backend with platform adaptations
3. **Web** (WASM) — iroh HTTP transport fallback, subset of features

---

## Phases

### Phase 1: Scaffold + Dependencies
- Update Cargo.toml with all latest versions
- Create module skeleton
- Set up Bridge channels + spawn_backend stub

### Phase 2: Storage + Identity
- SQLite schema + CRUD
- Identity generation + persistence
- PIN lock

### Phase 3: Transport + Network
- Iroh endpoint + gossip setup
- Direct messaging
- Group messaging

### Phase 4: Bridge Wiring
- Connect all Commands to backend handlers
- Connect all Events to UI signal updates

### Phase 5: UI Polish
- Route navigation
- Loading states, error handling
- Lock screen, onboarding flow

### Phase 6: Testing + Platform
- Unit tests for crypto, storage, protocol
- Integration tests
- Android build configuration
