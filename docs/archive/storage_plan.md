# NodeChat-New: Local Storage Implementation Plan

## Objective

Replace the `MockBackend` with a real, persistent SQLite database using `rusqlite` (bundled).  
The goal is **simplicity** — 4 tables, close to the old project, no extra indirection.

---

## Engine

```toml
rusqlite = { version = "=0.31.0", features = ["bundled"] }
uuid     = { version = "=1.8.0",  features = ["v4"] }
```

- `bundled`: SQLite is compiled into the binary. No system dependency. Works on Linux, Windows, macOS, and Android.
- WAL mode + foreign keys enabled on every open.

---

## Schema (4 Tables)

```sql
-- Singleton. id = 1 always.
CREATE TABLE IF NOT EXISTS local_identity (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    display_name    TEXT NOT NULL,
    node_id_hex     TEXT NOT NULL,
    x25519_secret   BLOB NOT NULL,
    endpoint_ticket TEXT NOT NULL DEFAULT '',
    pin_hash        TEXT NOT NULL DEFAULT ''
);

-- Contact book.
CREATE TABLE IF NOT EXISTS peers (
    node_id         TEXT PRIMARY KEY,
    display_name    TEXT NOT NULL,
    endpoint_ticket TEXT NOT NULL DEFAULT '',
    x25519_pubkey   TEXT NOT NULL DEFAULT '',
    verified        INTEGER NOT NULL DEFAULT 0
);

-- Swarm groups joined locally.
CREATE TABLE IF NOT EXISTS groups (
    topic_id        TEXT PRIMARY KEY,
    group_name      TEXT NOT NULL,
    symmetric_key   BLOB NOT NULL
);

-- All messages (direct and group).
-- target_id = peers.node_id (direct) OR groups.topic_id (group)
-- sender_id = sender's node_id_hex (never the string "me")
CREATE TABLE IF NOT EXISTS messages (
    id                TEXT PRIMARY KEY,
    kind              TEXT NOT NULL DEFAULT 'standard',
    target_id         TEXT NOT NULL,
    sender_id         TEXT NOT NULL,
    content           TEXT NOT NULL,
    timestamp         INTEGER NOT NULL,
    status            TEXT NOT NULL DEFAULT 'queued',
    invite_topic_id   TEXT NOT NULL DEFAULT '',
    invite_group_name TEXT NOT NULL DEFAULT '',
    invite_key        TEXT NOT NULL DEFAULT ''
);
```

---

## Derivation Rules (Never Stored)

| Field | How to derive |
|---|---|
| `initials` | First char of each word in `display_name`, max 2 chars |
| `is_outgoing` | `sender_id == local_identity.node_id_hex` |
| `has_queued_messages` | `COUNT(*) FROM messages WHERE target_id = ? AND status = 'queued'` > 0 |
| `invite_is_joined` | `COUNT(*) FROM groups WHERE topic_id = invite_topic_id` > 0 |
| `is_system` | `messages.kind == 'system'` |
| `sender_name` | Look up `peers.display_name WHERE node_id = sender_id`; if local node, use `local_identity.display_name`; else truncate sender_id to 8 chars |
| `conversation_id` | For direct: `peers.node_id`. For group: `groups.topic_id` |
| `direct_conversation_id` | Same as `peers.node_id` |
| `has_identity` | `SELECT COUNT(*) FROM local_identity` > 0 |
| `is_locked` | Session-only state. Not stored. False after successful unlock |
| `unread_count` | `0` for now — acceptable MVP default (old project did same) |
| `member_count` | Runtime from iroh-gossip. DB returns `0` |
| `is_online/relay/session_ready` | Runtime P2P state. Never stored |
| `connection_stage` | Runtime P2P state. Never stored |
| `timestamp` (string) | Format `messages.timestamp` (i64 Unix seconds) → e.g. `"14:32"` |
| `is_ephemeral` | Always `false`. Future feature |
| `ttl_seconds` | Always `0`. Future feature |

---

## File Structure

```
src/
  storage/
    mod.rs      ← open DB, WAL pragma, apply schema
    queries.rs  ← all SQL functions, one per operation, Result<T> returns
```

No other files. No ORM. Raw SQL with `rusqlite::params![]`.

---

## Query Functions to Implement (in `queries.rs`)

### Identity
- `insert_local_identity(conn, record) -> Result<()>`
- `get_local_identity(conn) -> Result<Option<LocalIdentityRecord>>`
- `update_display_name(conn, name) -> Result<()>`
- `update_endpoint_ticket(conn, ticket) -> Result<()>`
- `update_pin_hash(conn, hash) -> Result<()>`
- `delete_all(conn) -> Result<()>` — for ResetIdentity

### Peers
- `insert_peer(conn, record) -> Result<()>` — upsert on conflict
- `get_peer(conn, node_id) -> Result<Option<PeerRecord>>`
- `list_peers(conn) -> Result<Vec<PeerRecord>>`
- `update_peer_ticket(conn, node_id, ticket) -> Result<()>`
- `update_peer_pubkey(conn, node_id, pubkey) -> Result<()>`
- `set_peer_verified(conn, node_id, verified) -> Result<()>`
- `delete_peer(conn, node_id) -> Result<()>`

### Groups
- `insert_group(conn, record) -> Result<()>`
- `get_group(conn, topic_id) -> Result<Option<GroupRecord>>`
- `list_groups(conn) -> Result<Vec<GroupRecord>>`
- `delete_group(conn, topic_id) -> Result<()>`

### Messages
- `insert_message(conn, record) -> Result<()>` — INSERT OR IGNORE
- `list_messages(conn, target_id) -> Result<Vec<MessageRecord>>`
- `advance_status(conn, id, new_status) -> Result<()>` — forward-only
- `list_queued(conn, target_id) -> Result<Vec<MessageRecord>>`
- `has_queued(conn, target_id) -> Result<bool>`
- `clear_messages(conn) -> Result<()>` — all conversations
- `clear_conversation(conn, target_id) -> Result<()>` — one conversation
- `delete_conversation(conn, target_id, is_group) -> Result<()>` — messages + peer or group

### Chat List (composite query — builds `ChatPreviewRecord`)
- `list_chat_previews(conn, local_node_id) -> Result<Vec<ChatPreviewRecord>>`

---

## Implementation Order

1. **`Cargo.toml`** — add `rusqlite` dependency
2. **`src/storage/mod.rs`** — `initialize(path)`, `enable_pragmas()`, `apply_schema()`
3. **`src/storage/queries.rs`** — all functions above, in the order listed
4. **`src/lib.rs`** — replace `MockRuntime` start with a real `Storage` struct
5. **Unit tests** — in-memory DB, one test per critical path

---

## Key Rules (from old project's RULES.md pattern)

- Every SQL string lives in `queries.rs`. Nowhere else.
- Every function returns `Result<T>`.
- Status transitions are forward-only: `queued → sent → delivered → read`.
- `INSERT OR IGNORE` for messages (idempotent receive).
- `ON CONFLICT DO UPDATE` for peers (upsert on reconnect).
- WAL mode enabled before any query.
- Foreign keys ON (enforced at pragma level).
