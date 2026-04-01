# NodeChat — Coding & Testing Rules

**Document Type:** Strict Engineering Standards
**Project:** NodeChat — Secure Decentralized Chat
**Status:** Binding. No exceptions without documented justification in a PR comment.

> These rules exist for one reason: a two-person team on a 12-week deadline cannot afford to debug
> code that was written fast and loose. Every rule below has caused a real project to fail when ignored.
> Read them once. Then follow them every single day.

---

## Table of Contents

1. [The Non-Negotiables](#1-the-non-negotiables)
2. [Project Structure Rules](#2-project-structure-rules)
3. [Rust Language Rules](#3-rust-language-rules)
4. [Architecture Rules (Actor Model)](#4-architecture-rules-actor-model)
5. [Networking Rules](#5-networking-rules)
6. [Cryptography Rules](#6-cryptography-rules)
7. [Storage Rules](#7-storage-rules)
8. [UI Rules (Slint)](#8-ui-rules-slint)
9. [Error Handling Rules](#9-error-handling-rules)
10. [Testing Rules](#10-testing-rules)
11. [Git Rules](#11-git-rules)
12. [What Gets You Stopped](#12-what-gets-you-stopped)

---

## 1. The Non-Negotiables

These are absolute. Violating any of these stops the build until fixed.

**N-01. The code must compile with zero warnings.**
Run `cargo build` — if there are warnings, fix them before committing. `#[allow(dead_code)]` and `#[allow(unused)]` are banned except in files explicitly marked as scaffolding stubs during early development, and must be removed before Phase 3.

**N-02. `unwrap()` and `expect()` are banned in all non-test code.**
Every `.unwrap()` is a landmine. In a P2P app where network conditions are unpredictable, anything can fail. Use `?`, `if let`, `match`, or return a `Result`. The only exception is `expect()` in `main.rs` for fatal startup failures that cannot be recovered from — and each one must have a comment explaining why it is unrecoverable.

**N-03. The Slint UI thread never blocks.**
No `std::thread::sleep`, no synchronous I/O, no `block_on`, no heavy computation inside any Slint callback closure. If you need to wait for something in the UI, you send a `Command` to the backend and wait for an `AppEvent` back via `invoke_from_event_loop`. Full stop.

**N-04. Cryptographic primitives are never written from scratch.**
No custom encryption. No custom key derivation. No custom random number generation. You use `x25519-dalek`, `chacha20poly1305`, `sha2`, and `rand` from crates.io — audited, maintained, correct. If you find yourself writing bit manipulation for crypto purposes, stop immediately.

**N-05. Every function that can fail returns `Result`.**
`void`-equivalent functions that can fail return `Result<()>`. Functions that return data return `Result<T>`. Functions that genuinely cannot fail (pure math, pure string formatting) may return `T` directly. When in doubt, return `Result`.

**N-06. No `clone()` to silence the borrow checker.**
If you are cloning data to make the compiler happy, you have a design problem — not a cloning problem. Fix the ownership model. The only legitimate `clone()` calls are when you genuinely need two independent copies of data (e.g. sending a message to the UI while also writing it to the DB — two distinct consumers).

**N-07. Dependency versions are pinned.**
`Cargo.toml` specifies exact versions (e.g., `= "0.97.0"`, not `"^0.97"`). `Cargo.lock` is committed to the repository. Check ARCHITECTURE.md for the current latest verified versions before adding dependencies. No one upgrades a dependency mid-project without a team decision and a full test run.

---

## 2. Project Structure Rules

**S-01. One module, one responsibility.**
Each module in `src/` owns exactly one concern: `p2p/` owns all network I/O, `crypto/` owns all key operations, `storage/` owns all database access, `ui/` owns all Slint wiring. A function in `crypto/` never opens a database connection. A function in `storage/` never touches an iroh `Endpoint`. A function in `ui/` never calls a crypto function directly.

**S-02. Modules communicate through defined interfaces only.**
The only way the UI talks to the backend is through `mpsc::Sender<Command>`. The only way the backend talks to the UI is through `slint::invoke_from_event_loop`. No shared mutable state. No `Arc<Mutex<SomeGlobalThing>>` passed between the UI and the backend.

**S-03. The file structure in `ARCHITECTURE.md` is the law.**
Do not create new top-level modules without updating `ARCHITECTURE.md` first and agreeing as a team. New files within existing modules are fine without discussion.

**S-04. No logic in `main.rs`.**
`main.rs` does exactly three things: initializes the Tokio runtime, spawns the `NodeChatWorker`, and calls `NodeChatApp::run()`. Any logic beyond that belongs in a module.

**S-05. `mod.rs` files declare modules and define top-level structs only.**
Implementation details go in named submodules (`direct.rs`, `queries.rs`, etc.). A `mod.rs` that is hundreds of lines long is a sign that responsibilities were not split properly.

**S-06. Zero business logic in `.slint` files.**
`.slint` files in `ui/` contain only: layout, visual properties, colour tokens, animations, and named callbacks. No Rust-equivalent logic. No data fetching. No state management. A `.slint` file that does more than describe what something looks like is wrong.

---

## 3. Rust Language Rules

**R-01. All public types and functions have doc comments.**
Every `pub struct`, `pub enum`, `pub fn` must have a `///` doc comment explaining what it is and what it does — not what the code says, but what it *means*.

```rust
// BAD
pub fn ratchet_key(key: &mut [u8; 32]) { ... }

// GOOD
/// Advances the session key one step using SHA-256.
/// After calling this, the old key value is gone — there is no way to reverse it.
/// Call this exactly once after every message is sent or received.
pub fn ratchet_key(key: &mut [u8; 32]) { ... }
```

**R-02. No magic numbers.**
Every numeric constant that has a meaning gets a named constant.

```rust
// BAD
tokio::time::sleep(Duration::from_secs(10)).await;

// GOOD
const QUEUE_FLUSH_INTERVAL_SECS: u64 = 10;
tokio::time::sleep(Duration::from_secs(QUEUE_FLUSH_INTERVAL_SECS)).await;
```

**R-03. Match arms must be exhaustive — no catch-all `_` that silently ignores variants.**
When matching on `Command` or `AppEvent`, every variant must be handled explicitly. A `_ => {}` arm on an event enum means silently dropping events — which in a message queue means silently losing messages.

```rust
// BAD
match event {
    AppEvent::IncomingMessage { .. } => { ... }
    _ => {}
}

// GOOD
match event {
    AppEvent::IncomingMessage { .. }      => { ... }
    AppEvent::MessageStatusUpdate { .. }  => { ... }
    AppEvent::IncomingGroupMessage { .. } => { ... }
    AppEvent::GroupInviteReceived { .. }  => { ... }
    AppEvent::PeerOnlineStatus { .. }     => { ... }
    AppEvent::IncomingFile { .. }         => { ... }
}
```

**R-04. `async fn` in the backend, synchronous fn everywhere else.**
Only functions in `src/core/` and `src/p2p/` are `async`. Crypto functions are pure synchronous. Storage query functions are synchronous (rusqlite is sync by design). Slint callback closures are synchronous. If a Slint callback needs async work, it sends a `Command` and returns immediately.

**R-05. Structs derive only what they need.**
Do not `#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]` on every struct by default. `Debug` is always fine. `Clone` requires justification (see N-06).

**R-06. Use `&str` for read-only string parameters, `String` for owned strings.**
Function parameters that only read a string take `&str`. Functions that store or return a string take `String`. Never take `String` by value just to call `.as_str()` inside.

**R-07. Lifetimes are not avoided by cloning.**
If the borrow checker requires a lifetime annotation, write it. Do not clone data to avoid writing `'a`.

---

## 4. Architecture Rules (Actor Model)

**A-01. `Command` and `AppEvent` are the only cross-boundary types.**
Nothing that lives in the backend leaks into the UI layer and vice versa. The Slint UI has its own view-model types defined in `src/ui/models.rs` (Slint `ModelRc<T>` structs). These are populated from `AppEvent` data — they are not the same structs as the backend uses.

**A-02. The backend worker owns all mutable state.**
The SQLite connection, the iroh `Endpoint`, the `CryptoManager`, the session keys — all of it lives inside `NodeChatWorker`. The UI has no reference to any of these. It only has the channel sender and a weak reference to the Slint window.

**A-03. `handle_command()` and `handle_network()` never block the select loop.**
These are `async fn` called inside `tokio::select!`. They must not perform long-running synchronous work. SQLite writes are fast enough to be acceptable inline. Long network operations must be spawned with `tokio::spawn()` so the select loop keeps running.

**A-04. Every `Command` has exactly one handler.**
No two branches of `handle_command()` handle the same `Command` variant.

**A-05. `invoke_from_event_loop` is the only path from backend to UI.**
The backend never holds a strong reference to the Slint window. It holds a `Weak<AppWindow>` handle. All UI updates go through `slint::invoke_from_event_loop`. No exceptions.

**A-06. The periodic flush task is the only retry mechanism.**
If sending fails, write `status = 'queued'` to SQLite and let `flush_offline_queue()` handle it. There is exactly one retry path.

---

## 5. Networking Rules

**P-01. Never log message content.**
Logging must never include message plaintext, private keys, shared secrets, or full NodeIds. Logs are for connection events, error states, and delivery status only.

```rust
// BAD
tracing::debug!("Sending message: {}", plaintext);

// GOOD
tracing::debug!("Sending direct message to peer, payload_len={}", payload.len());
```

**P-02. All network errors are handled — none are silently swallowed.**
When a connection fails, a Pkarr lookup returns nothing, or a gossip broadcast fails — the error is logged AND the caller receives it via `Result`. The caller decides whether to queue, retry, or surface to the UI.

**P-03. Peer discovery is always attempted before queuing.**
Before writing a message as `queued`, the backend must attempt Pkarr discovery for that NodeId at least once in the current session.

**P-04. Iroh connections are reused, not reopened per message.**
`NetworkManager` maintains a `HashMap<NodeId, Connection>` and reuses existing connections. Do not open a new iroh connection for every message.

**P-05. Gossip topic subscriptions are idempotent.**
`subscribe_group()` must be a no-op if already subscribed to that topic.

**P-06. DERP relay usage is always surfaced to the UI.**
When iroh falls back to DERP, the UI must be notified via `AppEvent::PeerOnlineStatus` with `via_relay: true`.

---

## 6. Cryptography Rules

**C-01. The private key never leaves `CryptoManager`.**
The raw `StaticSecret` is never passed as a function argument outside of `src/crypto/`. Other modules receive derived artifacts (shared secrets, encrypted payloads) — never the raw key.

**C-02. Nonces are never reused.**
Every call to `encrypt_direct()` and `encrypt_group()` generates a fresh random nonce using `rand::rngs::OsRng`. The nonce is prepended to the ciphertext. Never hardcode a nonce. Never derive a nonce from a counter or timestamp.

```rust
// Correct pattern — always
let mut nonce_bytes = [0u8; 12];
OsRng.fill_bytes(&mut nonce_bytes);
let nonce = Nonce::from(nonce_bytes);
// output: [nonce (12 bytes) || ciphertext]
```

**C-03. Authentication tag failures are fatal for that message.**
When `decrypt_direct()` or `decrypt_group()` returns an authentication error, the message is dropped immediately. It is not retried. It is not passed upward. The error is logged and an `AppEvent` is pushed to the UI indicating decryption failure for that message ID.

**C-04. The hash ratchet advances on every message — no exceptions.**
`ratchet_key()` is called after every successfully sent message AND after every successfully received and decrypted message. The call is never conditional. Session keys live in memory inside `CryptoManager` only — they are never persisted to SQLite. A fresh DH exchange occurs on every app restart.

**C-05. Group keys are stored encrypted in SQLite.**
The `symmetric_key` column in the `groups` table stores the key encrypted under the user's local password-derived key — never plaintext.

**C-06. Decrypted plaintext is what gets stored in SQLite.**
The database stores messages as the user sent or received them. There is no value in storing ciphertext locally when local storage is already encrypted.

**C-07. Safety numbers use a canonical key ordering.**
```
safety_number = SHA-256(lower_pubkey_bytes || higher_pubkey_bytes)
```
Keys are concatenated in lexicographic order so both parties independently compute the same number. Split into groups of 5 digits for display.

---

## 7. Storage Rules

**DB-01. All database access goes through `src/storage/queries.rs`.**
No SQL string is written anywhere else in the codebase. Not in `core/`, not in `p2p/`, not in `ui/`.

**DB-02. Every query function is synchronous and returns `Result<T>`.**
rusqlite is synchronous. Do not wrap it in `spawn_blocking` — it is not needed at this scale.

**DB-03. Use prepared statements — no string-formatted SQL.**

```rust
// BAD
conn.execute(&format!("INSERT INTO peers VALUES ('{}')", node_id), [])?;

// GOOD
conn.execute(
    "INSERT INTO peers (node_id, display_name, x25519_pubkey) VALUES (?1, ?2, ?3)",
    params![node_id, display_name, pubkey_hex],
)?;
```

**DB-04. Message status transitions are one-directional.**
Status advances forward only: `queued` → `sent` → `delivered` → `read`. A delivered message cannot go back to queued. Enforce this in `queries.rs` — check the current status before writing a new one.

**DB-05. The database is initialized with `CREATE TABLE IF NOT EXISTS`.**
Run schema initialization on every startup. It is idempotent and safe.

**DB-06. SQLite WAL mode is enabled on startup.**
```rust
conn.execute_batch("PRAGMA journal_mode=WAL;")?;
conn.execute_batch("PRAGMA foreign_keys=ON;")?;
```
Enable before any other operation.

**DB-07. No N+1 queries.**
Displaying 20 contacts with their last messages = one query with a JOIN. Not 20 queries in a loop.

---

## 8. UI Rules (Slint)

**U-01. `.slint` files contain zero business logic.**
A `.slint` file defines: layout, visual properties, colour tokens, animations, and `callback` declarations. It does not compute state, call functions, or make decisions. If you are writing something in a `.slint` file that could be a Rust function, it belongs in `src/ui/models.rs` instead.

**U-02. All Slint callbacks are wired in `src/ui/mod.rs` at startup — never inline.**
Every `on_*` callback on the Slint window is wired exactly once when `NodeChatApp::new()` runs. No callback is wired inside another callback. No callback is wired conditionally.

```rust
// GOOD — wired once at startup
ui.on_send_message({
    let tx = tx_commands.clone();
    move |text| {
        let _ = tx.try_send(Command::SendDirectMessage { ... });
    }
});
```

**U-03. Every user action sends a `Command` — it does not produce immediate state changes.**
When the user taps "Send", the UI sends `Command::SendDirectMessage` and stops. The message bubble appears in the chat list only when the backend echoes it back as an `AppEvent`. This keeps the UI truthful about what actually happened.

**U-04. `invoke_from_event_loop` closures are kept short.**
The closure passed to `invoke_from_event_loop` runs on the Slint event thread. It must complete quickly — update a model or set a property, then return. Never perform I/O, network calls, or blocking operations inside this closure.

**U-05. Slint model updates go through `apply_event()` in `src/ui/models.rs`.**
All `AppEvent` → Slint model/property translations live in `apply_event()`. This is the single place where backend data becomes UI data. No direct model manipulation happens elsewhere.

**U-06. Weak references are always upgraded before use.**
`as_weak()` produces a `Weak<AppWindow>`. Always call `.upgrade()` and handle the `None` case — the window may have been destroyed.

```rust
// GOOD
if let Some(ui) = handle.upgrade() {
    ui.set_peer_online(true);
}
```

**U-07. All timestamps are stored as UTC Unix integers — displayed in local time.**
Store UTC in SQLite. Convert to local time only in `src/ui/models.rs` at the point of building the display string for Slint. Never store local time in the database.

**U-08. Destructive actions require a two-step confirmation.**
"Clear all messages" and "Delete identity" require the user to confirm in a dedicated Slint dialog component before the `Command` is sent. The `.slint` file defines the dialog component. The confirmation fires the callback. The Rust side sends the command.

**U-09. Chat scroll behaviour is explicit.**
Scroll to the bottom on a new message only if the user was already at the bottom before the message arrived. Slint's `ListView` exposes scroll position — use it. Do not forcibly scroll when the user is reading history.

**U-10. The input field clears only after the command send succeeds.**
Do not clear the text input before confirming `try_send()` returned `Ok`. If the channel is full and the send fails, the user's text must remain in the input field.

---

## 9. Error Handling Rules

**E-01. Use `anyhow::Result` for application-level errors.**
Do not define custom error enums for every module. `anyhow::Result<T>` with `.context("what we were doing")` provides enough information for debugging.

```rust
// GOOD
fn load_identity(path: &Path) -> anyhow::Result<Identity> {
    let bytes = std::fs::read(path)
        .context("failed to read identity file from disk")?;
    // ...
}
```

**E-02. Errors are logged at the boundary where they are handled.**
If an error is propagated up via `?`, it is not logged at every level. Log only at the final handler. One error, one log line.

**E-03. User-facing error messages are plain English.**
When an error surfaces to the UI, the message is written for the user: "Failed to send: peer unreachable" — not "iroh::endpoint::ConnectError: connection timeout after 30s".

**E-04. Panics are only for programmer errors — never for runtime failures.**
`panic!()` and `unreachable!()` are only for states that should be impossible given correct code. They are never used to handle network failures, missing files, or user input errors.

---

## 10. Testing Rules

### 10.1 What Must Be Tested

Every item below is required before the project is submitted. Not optional.

| ID | What | Where |
|---|---|---|
| T-01 | ChaCha20 encrypt → decrypt roundtrip | `src/crypto/mod.rs` tests |
| T-02 | ChaCha20 decrypt fails on tampered ciphertext | `src/crypto/mod.rs` tests |
| T-03 | ChaCha20 decrypt fails on wrong key | `src/crypto/mod.rs` tests |
| T-04 | Hash ratchet produces different key each step | `src/crypto/mod.rs` tests |
| T-05 | Hash ratchet is deterministic (same input → same output) | `src/crypto/mod.rs` tests |
| T-06 | X25519 DH: Alice and Bob derive the same shared secret | `src/crypto/mod.rs` tests |
| T-07 | X25519 DH: Different key pairs derive different secrets | `src/crypto/mod.rs` tests |
| T-08 | Safety number is symmetric (Alice+Bob = Bob+Alice) | `src/crypto/mod.rs` tests |
| T-09 | Group key encrypt → decrypt roundtrip | `src/crypto/mod.rs` tests |
| T-10 | Group key decrypt fails on tampered ciphertext | `src/crypto/mod.rs` tests |
| T-11 | Write peer to DB → read it back, fields match | `src/storage/queries.rs` tests |
| T-12 | Write message with status `queued` → read back, confirm status | `src/storage/queries.rs` tests |
| T-13 | Message status advances: queued → sent → delivered → read | `src/storage/queries.rs` tests |
| T-14 | Message status cannot go backward (delivered → queued must fail) | `src/storage/queries.rs` tests |
| T-15 | Fetch all queued messages for a peer returns correct results | `src/storage/queries.rs` tests |
| T-16 | Write group with key → read back, key bytes match | `src/storage/queries.rs` tests |
| T-17 | Two `NodeChatWorker` instances exchange a plaintext message | `tests/integration_tests.rs` |
| T-18 | Message sent to offline peer is stored as `queued` | `tests/integration_tests.rs` |
| T-19 | Queued message delivers and status updates when peer reconnects | `tests/integration_tests.rs` |
| T-20 | Nonce uniqueness: 1000 encryptions produce 1000 different nonces | `src/crypto/mod.rs` tests |

### 10.2 How Tests Are Written

**T-RULE-01. Test names describe the scenario, not the mechanism.**

```rust
// BAD
#[test]
fn test_encrypt() { ... }

// GOOD
#[test]
fn chacha20_ciphertext_is_unreadable_with_wrong_key() { ... }
```

**T-RULE-02. Unit tests live in the same file as the code they test.**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chacha20_roundtrip_produces_original_plaintext() {
        let key = CryptoManager::generate_group_key();
        let plaintext = b"hello from nodechat";
        let ciphertext = CryptoManager::encrypt_group(plaintext, &key);
        let recovered = CryptoManager::decrypt_group(&ciphertext, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }
}
```

**T-RULE-03. Integration tests live in `tests/` at the project root.**
Tests that spin up two workers, bind two iroh endpoints, or touch the filesystem go in `tests/integration_tests.rs`.

**T-RULE-04. Every test is independent — no shared mutable state between tests.**

```rust
fn test_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    storage::initialize_schema(&conn).unwrap();
    conn
}
```

Never reuse a single database connection across tests. Tests run in parallel.

**T-RULE-05. Crypto tests use fixed test vectors where possible.**

```rust
#[test]
fn sha256_ratchet_matches_known_vector() {
    let mut key = [0u8; 32];
    ratchet_key(&mut key);
    assert_eq!(
        hex::encode(key),
        "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
    );
}
```

**T-RULE-06. Tests must not make real network connections.**
Tests that call `NetworkManager` use a local iroh endpoint bound to `127.0.0.1`. Tests never hit external DERP servers or Pkarr DHT. Tests requiring real network access are marked `#[ignore]` with a comment explaining why.

**T-RULE-07. Test failure messages are human-readable.**

```rust
// BAD
assert!(result.is_ok());

// GOOD
assert!(result.is_ok(), "decryption failed: {:?}", result.err());
```

**T-RULE-08. All tests pass before any commit to `main`.**
Run `cargo test` before every push. A flaky test is treated as a failing test.

### 10.3 Running Tests

```bash
# All unit tests
cargo test

# Crypto tests only
cargo test --lib crypto

# Storage tests only
cargo test --lib storage

# Integration tests
cargo test --test integration_tests

# With stdout visible
cargo test -- --nocapture

# One specific test
cargo test chacha20_roundtrip_produces_original_plaintext
```

### 10.4 Manual Testing Checklist (Pre-Defense)

Run on two separate machines before the defense. Tick each one off physically.

```
[ ] First launch flow completes on desktop (Windows or Linux)
[ ] First launch flow completes on Android (if in scope)
[ ] NodeId QR code scans correctly from one device to another
[ ] 1:1 message delivers in under 2 seconds on the same LAN
[ ] Message shows 'queued' when recipient node is stopped
[ ] Queued message delivers within 15 seconds of recipient node restarting
[ ] Key ratchet system event appears in chat after each message
[ ] Key verification safety numbers match on both devices
[ ] Verified badge appears after marking a contact as verified
[ ] Group created, both members can see and send messages
[ ] Group invite received as a 1:1 message, accepted, swarm joined
[ ] File transfer completes for a file under 5MB
[ ] Ephemeral message timer is visible in bubble
[ ] App restarts with password gate when password is set
[ ] Wrong password shows error and clears input field
[ ] 5 wrong passwords triggers 30-second cooldown
[ ] 'Clear all messages' requires confirmation and works
[ ] Network status dot reflects relay vs direct correctly
[ ] App stays responsive (no UI freeze) during message send
[ ] App stays responsive during group gossip broadcast
```

---

## 11. Git Rules

**G-01. Branch names describe the feature.**
Format: `feature/phase-2-iroh-endpoint`, `fix/crypto-nonce-reuse`, `test/storage-queue-tests`. Never commit directly to `main`.

**G-02. Commit messages are descriptive.**
Format: `[module] short description`

```
// BAD
git commit -m "fix stuff"

// GOOD
git commit -m "[crypto] fix nonce reuse bug in encrypt_direct"
git commit -m "[ui] wire on_send_message callback to Command channel"
git commit -m "[storage] add message status backward-transition guard"
```

**G-03. `Cargo.lock` is always committed.**
Every commit that changes dependencies commits the updated `Cargo.lock`.

**G-04. No commented-out code in commits to `main`.**
If code is disabled, it is deleted. Git history is the backup.

**G-05. Each feature branch is merged only when its tests pass.**
Crypto branch → T-01 through T-10 must pass. Storage branch → T-11 through T-16 must pass. Tests are written as part of the branch, not after merging.

---

## 12. What Gets You Stopped

If any of the following are found during a team review, work stops on new features until the issue is resolved.

| Issue | Why it stops work |
|---|---|
| A `panic!` or `.unwrap()` on a network or DB operation | Will crash the app live during the defense |
| Plaintext message content in a log statement | Security violation — contradicts the core promise of the project |
| A raw private key passed outside `CryptoManager` | Catastrophic design flaw — must be rearchitected |
| A nonce derived from a counter or timestamp | Breaks encryption security — fix before any demo |
| A `Command` or `AppEvent` variant with no handler | Silent data loss |
| Business logic written inside a `.slint` file | Violates S-06 — move to `src/ui/models.rs` |
| A Slint model mutated outside `apply_event()` | Breaks the single update path — causes inconsistent UI state |
| `invoke_from_event_loop` closure doing I/O or blocking work | Freezes the Slint event thread |
| A test that was deleted because it was hard to pass | Tests find bugs — deleting them hides bugs |
| `Cargo.toml` with `*` or `>=` version constraints | Build reproducibility broken |
| Any SQL built from string formatting | DB-03 violation |

---

*Last Updated: 2026-03-30*
*Project: NodeChat — Secure Decentralized Chat*
*Document: Coding & Testing Rules v1.2 — Latest Crate Versions*
