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
8. [UI Rules (egui)](#8-ui-rules-egui)
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

**N-03. The UI thread never blocks.**
No `std::thread::sleep`, no synchronous I/O, no `block_on`, no heavy computation in any `egui::App::update()` implementation. If you need to wait for something in the UI, you send a `Command` to the backend and wait for an `AppEvent` back. Full stop.

**N-04. Cryptographic primitives are never written from scratch.**
No custom encryption. No custom key derivation. No custom random number generation. You use `x25519-dalek`, `chacha20poly1305`, `sha2`, and `rand` from crates.io — audited, maintained, correct. If you find yourself writing bit manipulation for crypto purposes, stop immediately.

**N-05. Every function that can fail returns `Result`.**
`void`-equivalent functions that can fail return `Result<()>`. Functions that return data return `Result<T>`. Functions that genuinely cannot fail (pure math, pure string formatting) may return `T` directly. When in doubt, return `Result`.

**N-06. No `clone()` to silence the borrow checker.**
If you are cloning data to make the compiler happy, you have a design problem — not a cloning problem. Fix the ownership model. The only legitimate `clone()` calls are when you genuinely need two independent copies of data (e.g. sending a message to the UI while also writing it to the DB — two distinct consumers).

**N-07. Dependency versions are pinned.**
`Cargo.toml` specifies exact versions (`= "0.29.0"`, not `"^0.29"`). `Cargo.lock` is committed to the repository. No one upgrades a dependency mid-project without a team decision and a full test run.

---

## 2. Project Structure Rules

**S-01. One module, one responsibility.**
Each module in `src/` owns exactly one concern: `p2p/` owns all network I/O, `crypto/` owns all key operations, `storage/` owns all database access, `ui/` owns all rendering. A function in `crypto/` never opens a database connection. A function in `storage/` never touches an iroh `Endpoint`. A function in `ui/` never calls a crypto function directly.

**S-02. Modules communicate through defined interfaces only.**
The only way the UI talks to the backend is through `mpsc::Sender<Command>`. The only way the backend talks to the UI is through `broadcast::Sender<AppEvent>`. No shared mutable state. No `Arc<Mutex<SomeGlobalThing>>` passed between the UI and the backend.

**S-03. The file structure in `ARCHITECTURE.md` is the law.**
Do not create new top-level modules without updating `ARCHITECTURE.md` first and agreeing as a team. New files within existing modules are fine without discussion.

**S-04. No logic in `main.rs`.**
`main.rs` does exactly three things: initializes the Tokio runtime, spawns the `NodeChatWorker`, and hands off to `eframe::run_native()`. Any logic beyond that belongs in a module.

**S-05. `mod.rs` files declare modules and define top-level structs only.**
Implementation details go in named submodules (`direct.rs`, `queries.rs`, etc.). A `mod.rs` that is hundreds of lines long is a sign that responsibilities were not split properly.

---

## 3. Rust Language Rules

**R-01. All public types and functions have doc comments.**
Every `pub struct`, `pub enum`, `pub fn` must have a `///` doc comment explaining what it is and what it does. Not what the code says — what it *means*.

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
When matching on `Command` or `AppEvent`, every variant must be handled explicitly. A `_ => {}` arm means you are silently dropping events — which in a message queue means silently losing messages.

```rust
// BAD
match event {
    AppEvent::IncomingMessage { .. } => { ... }
    _ => {} // what else came in? nobody knows
}

// GOOD
match event {
    AppEvent::IncomingMessage { .. } => { ... }
    AppEvent::MessageStatusUpdate { .. } => { ... }
    AppEvent::IncomingGroupMessage { .. } => { ... }
    AppEvent::GroupInviteReceived { .. } => { ... }
    AppEvent::PeerOnlineStatus { .. } => { ... }
    AppEvent::IncomingFile { .. } => { ... }
}
```

**R-04. `async fn` in the backend, synchronous fn everywhere else.**
Only functions in `src/core/`, `src/p2p/`, and the top of `src/storage/` (connection pool management) are `async`. Crypto functions are pure synchronous. UI functions are synchronous. SQLite queries are synchronous (rusqlite is sync by design — this is correct).

**R-05. Structs derive only what they need.**
Do not `#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]` on every struct by default. Add derives only when you have a concrete use case for them. `Debug` is always fine. `Clone` requires justification (see N-06).

**R-06. Use `&str` for read-only string parameters, `String` for owned strings.**
Function parameters that only read a string take `&str`. Functions that store a string or return it take `String`. Never take `String` by value just to call `.as_str()` inside.

**R-07. Lifetimes are not avoided by cloning.**
If the borrow checker requires a lifetime annotation to express correct ownership, write the lifetime annotation. Do not clone data to avoid writing `'a`.

---

## 4. Architecture Rules (Actor Model)

**A-01. `Command` and `AppEvent` are the only cross-boundary types.**
Nothing that lives in the backend leaks into the UI layer and vice versa. The UI has its own view-model types (`UIChatMessage`, `UIPeer`). These are built from `AppEvent` data — they are not the same structs.

**A-02. The backend worker owns all mutable state.**
The SQLite connection, the iroh `Endpoint`, the `CryptoManager`, the session keys — all of it lives inside `NodeChatWorker`. The UI has no reference to any of these. It only has the channel handles.

**A-03. `handle_command()` and `handle_network()` never block the select loop.**
These are `async fn` called inside `tokio::select!`. They must not perform long-running synchronous operations. SQLite writes are fast enough to be acceptable. A long network operation must be spawned with `tokio::spawn()` so the select loop keeps running.

**A-04. Every `Command` has exactly one handler.**
No two branches of `handle_command()` handle the same `Command` variant. No variant is handled in multiple places.

**A-05. `AppEvent` is broadcast, not sent to a specific receiver.**
The `broadcast::Sender<AppEvent>` is used precisely because there may be multiple UI components listening. Do not attempt to target a specific UI panel with a specific event. UI components filter events that are relevant to them.

**A-06. The periodic flush task is the only retry mechanism.**
Do not implement retry logic anywhere else. If sending fails, write `status = 'queued'` to SQLite and let `flush_offline_queue()` handle it on the next tick. There is exactly one retry path — keeping it that way makes debugging possible.

---

## 5. Networking Rules

**P-01. Never log message content.**
Logging (`println!`, `tracing::debug!`, `eprintln!`) must never include message plaintext, private keys, shared secrets, or full NodeIds of users. Logs are for connection events, error states, and delivery status only.

```rust
// BAD
tracing::debug!("Sending message: {}", plaintext);

// GOOD
tracing::debug!("Sending direct message to peer, payload_len={}", payload.len());
```

**P-02. All network errors are handled — none are silently swallowed.**
When an iroh connection fails, when a Pkarr lookup returns nothing, when a gossip broadcast fails — the error is logged at the appropriate level AND the caller receives it via `Result`. It is then up to the caller to decide whether to queue, retry, or surface to the UI.

**P-03. Peer discovery is always attempted before queuing.**
Before writing a message as `queued`, the backend must attempt Pkarr discovery for the target NodeId at least once in that session. Do not queue a message for a peer that is actually online but whose address has not been refreshed.

**P-04. Iroh connections are reused, not reopened per message.**
For a given `NodeId`, maintain the connection if it is alive. Do not open a new iroh connection for every message — this is expensive and unnecessary. The `NetworkManager` maintains a connection map (`HashMap<NodeId, Connection>`) and reuses existing connections.

**P-05. Gossip topic subscriptions are idempotent.**
Calling `subscribe_group()` for a topic the node is already subscribed to must be a no-op. No duplicate subscriptions. Check before subscribing.

**P-06. DERP relay usage is always surfaced to the UI.**
When iroh falls back to a DERP relay for a connection, the UI must be notified via `AppEvent::PeerOnlineStatus` with the relay flag set. The user has a right to know when their traffic is routing through a relay.

---

## 6. Cryptography Rules

**C-01. The private key never leaves `CryptoManager`.**
The raw `StaticSecret` is never passed as a function argument outside of `src/crypto/`. Other modules receive derived artifacts (shared secrets, encrypted payloads) — never the private key itself. If another module needs to "sign" or "decrypt" something, it calls a method on `CryptoManager`, not on the raw key.

**C-02. Nonces are never reused.**
ChaCha20-Poly1305 requires a unique nonce per encryption. Every call to `encrypt_direct()` and `encrypt_group()` generates a fresh random nonce using `rand::rngs::OsRng`. The nonce is prepended to the ciphertext so the receiver can extract it for decryption. Never hardcode a nonce. Never increment a counter nonce manually.

```rust
// The correct pattern — always
let mut nonce_bytes = [0u8; 12];
OsRng.fill_bytes(&mut nonce_bytes);
let nonce = Nonce::from(nonce_bytes);
// prepend nonce to output: [nonce (12 bytes) | ciphertext]
```

**C-03. Authentication tag failures are fatal for that message.**
When `decrypt_direct()` or `decrypt_group()` returns an authentication error, the message is dropped. It is not retried. It is not passed upward with a warning. The error is logged and an appropriate `AppEvent` is sent to the UI indicating a decryption failure for that message ID. Do not attempt to use partially decrypted data.

**C-04. The hash ratchet advances on every message — no exceptions.**
`ratchet_key()` is called after every successfully sent message and after every successfully received and decrypted message. The call is not conditional. The ratchet state (current session key) lives in a `HashMap<NodeId, [u8; 32]>` inside `CryptoManager`. Session keys are in-memory only — they are not persisted to SQLite. On app restart, a fresh DH exchange occurs.

**C-05. Group keys are stored encrypted in SQLite.**
The `symmetric_key` column in the `groups` table stores the key encrypted under the user's local password-derived key — not plaintext. Even if someone extracts the SQLite file, they cannot read group keys without the password.

**C-06. Decrypted message plaintext is what gets stored in SQLite, not ciphertext.**
The database stores the message as the user sent or received it. There is no value in storing ciphertext locally — the local storage is already protected by the database encryption. Storing ciphertext locally would only make search and display impossible.

**C-07. Key verification (safety numbers) uses both parties' public keys.**
The safety number displayed during key verification is derived as:
```
safety_number = SHA-256(our_pubkey_bytes || their_pubkey_bytes)
```
Split into groups of 5 digits for display. The input to SHA-256 must concatenate keys in a consistent order (lower key bytes first, lexicographically) so both parties compute the same number independently.

---

## 7. Storage Rules

**DB-01. All database access goes through `src/storage/queries.rs`.**
No SQL string is written anywhere else in the codebase. Not in `core/`, not in `p2p/`, not in `ui/`. If you need data from the database, you add a function to `queries.rs` and call it.

**DB-02. Every query function is synchronous and returns `Result<T>`.**
rusqlite is a synchronous library. Do not wrap it in `spawn_blocking` unless profiling proves it is a bottleneck (it will not be, for this scale). Keep it simple.

**DB-03. Use prepared statements — no string-formatted SQL.**
SQL injection from local data is unlikely but string formatting in queries is a bad habit that will bite in unexpected ways. Use rusqlite's `params![]` macro for all parameterized queries.

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
Status can only advance forward: `queued` → `sent` → `delivered` → `read`. A message that is `delivered` cannot go back to `queued`. Enforce this in `queries.rs` — the update function checks the current status before writing a new one.

**DB-05. The database is initialized with `CREATE TABLE IF NOT EXISTS`.**
On every startup, run the schema initialization. This is idempotent and safe. Never check "does the table exist?" separately — let the SQL handle it.

**DB-06. SQLite WAL mode is enabled on startup.**
```rust
conn.execute_batch("PRAGMA journal_mode=WAL;")?;
conn.execute_batch("PRAGMA foreign_keys=ON;")?;
```
WAL mode prevents database locks during concurrent reads. Enable it before any other operation.

**DB-07. No N+1 queries.**
If you are displaying a list of 20 contacts and their last messages, that is one query with a JOIN — not 20 individual message queries inside a loop. Write the query correctly the first time.

---

## 8. UI Rules (egui)

**U-01. `update()` reads events first, then draws.**
The first block in every `update()` call drains `rx_event.try_recv()` into local UI state. Only after all pending events are processed does the drawing code run. Never interleave event processing and drawing.

**U-02. UI state is derived from events — never mutated directly by drawing code.**
The draw code reads from `self.messages_view`, `self.contact_book`, etc. It does not write to them. State changes only happen in `update_state_from_event()`.

**U-03. Every user action sends a `Command` — it does not produce immediate state changes.**
When the user clicks "Send", the UI sends `Command::SendDirectMessage`. It does not immediately add the message to `self.messages_view`. The message appears in the view only when the backend echoes it back as `AppEvent::IncomingMessage` (or a dedicated sent-confirmation event). This keeps the UI truthful about what actually happened.

**U-04. No `unwrap()` on channel sends.**
`tx_cmd.try_send()` can fail if the channel is full. Handle it:
```rust
// BAD
self.tx_cmd.try_send(cmd).unwrap();

// GOOD
if let Err(e) = self.tx_cmd.try_send(cmd) {
    tracing::warn!("Command channel full, dropping command: {e}");
    // surface to UI if appropriate
}
```

**U-05. The chat scroll position is managed explicitly.**
When a new message arrives, scroll to the bottom only if the user was already at the bottom before the message arrived. Do not forcibly scroll to the bottom if the user has scrolled up to read history. egui provides scroll area state for this.

**U-06. All timestamps are displayed in local time.**
Store timestamps as UTC Unix integers in SQLite. Convert to local time only at the display layer in `views.rs`. Never store local time in the database.

**U-07. The input field is cleared only after the Command is sent successfully.**
Do not clear the text input on button click before confirming `try_send()` succeeded. If the channel is full and the send fails, the user's text must still be in the input field.

**U-08. Destructive actions require a confirmation dialog.**
"Clear all messages" and "Delete identity" show a modal confirmation: "Are you sure? This cannot be undone." The action only fires after explicit confirmation. egui's `Window` can implement this inline.

---

## 9. Error Handling Rules

**E-01. Use `anyhow::Result` for application-level errors.**
Do not define custom error enums for every module. `anyhow::Result<T>` with `.context("what we were doing when this failed")` provides enough information for debugging without the overhead of a full error type hierarchy.

```rust
// GOOD
fn load_identity(path: &Path) -> anyhow::Result<Identity> {
    let bytes = std::fs::read(path)
        .context("failed to read identity file from disk")?;
    // ...
}
```

**E-02. Errors are logged at the boundary where they are handled.**
If a function returns a `Result` and the caller handles the error, the caller logs it. If the error is propagated up via `?`, it is not logged at every level — only at the final handler. Otherwise you get the same error printed five times.

**E-03. User-facing error messages are plain English.**
When an error must surface to the UI (via an `AppEvent` or a status label), the message is written for the user — not the developer. "Failed to send: peer unreachable" not "iroh::endpoint::ConnectError: connection timeout after 30s".

**E-04. Panics are only for programmer errors — never for runtime failures.**
`panic!()` and `unreachable!()` are only for states that should be impossible given correct code — like matching on an enum variant that does not exist. They are never used to handle network failures, missing files, or user input errors.

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
| T-17 | Two `NodeChatWorker` instances exchange a plaintext message | `src/core/` integration tests |
| T-18 | Message sent to offline peer is stored as `queued` | `src/core/` integration tests |
| T-19 | Queued message delivers and status updates when peer reconnects | `src/core/` integration tests |
| T-20 | Nonce uniqueness: 1000 encryptions produce 1000 different nonces | `src/crypto/mod.rs` tests |

### 10.2 How Tests Are Written

**T-RULE-01. Every test has a name that describes the scenario, not the mechanism.**

```rust
// BAD
#[test]
fn test_encrypt() { ... }

// GOOD
#[test]
fn chacha20_ciphertext_is_unreadable_with_wrong_key() { ... }
```

**T-RULE-02. Unit tests live in the same file as the code they test.**
At the bottom of `src/crypto/mod.rs`, add:
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
Tests that spin up two workers, bind two iroh endpoints, or touch the filesystem go in `tests/integration_tests.rs` — not in `#[cfg(test)]` blocks inside module files.

**T-RULE-04. Every test is independent — no shared mutable state between tests.**
Use a unique in-memory SQLite database per storage test:
```rust
fn test_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    storage::initialize_schema(&conn).unwrap();
    conn
}
```
Do not use a single global database connection across tests. Tests run in parallel — they will corrupt each other's data.

**T-RULE-05. Crypto tests use fixed test vectors where possible.**
For the hash ratchet and key derivation, hard-code known input/output pairs derived manually or from reference implementations. This proves your implementation matches the specification — not just that it is internally consistent.

```rust
#[test]
fn sha256_ratchet_matches_known_vector() {
    let mut key = [0u8; 32]; // all-zero starting key
    ratchet_key(&mut key);
    // SHA-256([0x00; 32]) = known value — verify against a reference
    assert_eq!(
        hex::encode(key),
        "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
    );
}
```

**T-RULE-06. Tests must not make real network connections.**
Unit and integration tests run offline. Tests that call `NetworkManager` use a local iroh endpoint bound to `127.0.0.1`. Tests never hit external DERP servers, Pkarr DHT, or the internet. If a test requires real network access, it is marked `#[ignore]` with a comment explaining why and how to run it manually.

**T-RULE-07. Test failure messages are human-readable.**
Use `assert_eq!` with a message, not just a bare `assert!`:
```rust
// BAD
assert!(result.is_ok());

// GOOD
assert!(result.is_ok(), "decryption failed: {:?}", result.err());
```

**T-RULE-08. All tests pass before any commit to `main`.**
Run `cargo test` before every push. If a test is flaky (passes sometimes, fails sometimes), it is treated as a failing test — fix it before committing.

### 10.3 Running Tests

```bash
# Run all unit tests
cargo test

# Run only crypto tests
cargo test --lib crypto

# Run only storage tests  
cargo test --lib storage

# Run integration tests
cargo test --test integration_tests

# Run with output visible (useful when debugging a failing test)
cargo test -- --nocapture

# Run a specific test by name
cargo test chacha20_roundtrip_produces_original_plaintext
```

### 10.4 Manual Testing Checklist (Pre-Defense)

Run these manually on two separate machines (or two separate user accounts on one machine) before the defense. Check each one off:

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
[ ] File transfer completes successfully for a file under 5MB
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

**G-01. Branch names describe the feature being built.**
Format: `feature/phase-2-iroh-endpoint`, `fix/crypto-nonce-reuse`, `test/storage-queue-tests`. Never commit directly to `main`.

**G-02. Commit messages are descriptive.**
Format: `[module] short description of what changed`

```
// BAD
git commit -m "fix stuff"
git commit -m "wip"

// GOOD
git commit -m "[crypto] fix nonce reuse bug in encrypt_direct"
git commit -m "[storage] add message status backward-transition guard"
git commit -m "[p2p] reuse iroh connections via NodeId map"
```

**G-03. `Cargo.lock` is committed.**
Every commit that changes dependencies commits the updated `Cargo.lock`. Reviewers and the other team member must be able to reproduce the exact build.

**G-04. No commented-out code in commits to `main`.**
If code is disabled, it is deleted. Git history is the backup. Commented-out code sitting in `main` creates confusion about what is and is not active.

**G-05. Each feature branch is merged only when its tests pass.**
A branch that adds crypto functionality is merged only when T-01 through T-10 pass. A branch that adds storage functionality is merged only when T-11 through T-16 pass. Tests are not written after merging — they are written as part of the same branch.

---

## 12. What Gets You Stopped

If any of the following are found during a team review, work stops on new features until the issue is resolved:

| Issue | Why it stops work |
|---|---|
| A `panic!` or `.unwrap()` on a network or DB operation | Will crash the app live during the defense presentation |
| Plaintext message content in a log statement | Security violation — contradicts the project's core promise |
| A raw private key passed outside `CryptoManager` | Catastrophic design flaw — if found in code review, it must be rearchitected |
| A nonce that is derived from a counter or timestamp | Breaks encryption security — must be fixed before any demo |
| A `Command` or `AppEvent` variant with no handler | Silent data loss — the team will not know messages are disappearing |
| A test that was deleted because it was hard to pass | Tests exist to find bugs — deleting them hides bugs |
| `Cargo.toml` with `*` or `>=` version constraints | Build reproducibility broken — dependency upgrades can silently break the build |
| Any SQL built from string formatting | DB-03 violation — fix before it becomes a habit |
| UI state mutated directly in drawing code | Breaks the Actor Model contract — causes race conditions between draw cycles |

---

*Last Updated: 2026-03-30*  
*Project: NodeChat — Secure Decentralized Chat*  
*Document: Coding & Testing Rules v1.0 — Binding*
