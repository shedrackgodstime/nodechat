//! The NodeChat Actor Worker — the async backbone of the application.
//!
//! `NodeChatWorker::run` drives a `tokio::select!` loop that:
//!   1. Handles `Command`s from the Slint UI.
//!   2. Handles `NetworkEvent`s from `NetworkManager`.
//!   3. Flushes the offline message queue every `QUEUE_FLUSH_INTERVAL_SECS`.
//!
//! The UI thread is never blocked; all I/O and crypto happen here (RULES.md N-03).
//! The DB connection, crypto state, and network handle all live here (RULES.md A-02).

pub mod commands;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use rusqlite::Connection;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::crypto::CryptoManager;
use crate::p2p::{NetworkEvent, NetworkManager};
use crate::storage::{self, queries};

use commands::{AppEvent, Command, MessageStatus};

/// How often the offline queue flush task runs (RULES.md R-02).
const QUEUE_FLUSH_INTERVAL_SECS: u64 = 10;

/// Channel capacity for network events from the `NetworkManager`.
const NETWORK_EVENT_CHANNEL_CAPACITY: usize = 64;

/// The central actor. Owns the DB connection, crypto state, and network handle.
///
/// Receives `Command`s from the Slint UI, emits `AppEvent`s back via
/// `slint::invoke_from_event_loop` (set up in `src/ui/mod.rs`).
pub struct NodeChatWorker {
    db:         Connection,
    crypto:     Option<CryptoManager>,
    network:    NetworkManager,
    rx_commands: mpsc::Receiver<Command>,
    tx_events:  broadcast::Sender<AppEvent>,
    rx_network: mpsc::Receiver<NetworkEvent>,
}

impl NodeChatWorker {
    /// Construct the worker. Call from an async context at application startup.
    ///
    /// * `rx_commands` — the receiving end of the command channel from the UI.
    /// * `tx_events`   — broadcast sender used to push `AppEvent`s to the UI listener.
    /// * `db_path`     — path to the SQLite database file.
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened or the network fails to bind.
    pub async fn new(
        rx_commands: mpsc::Receiver<Command>,
        tx_events: broadcast::Sender<AppEvent>,
        db_path: &std::path::Path,
    ) -> Result<Self> {
        let db = storage::initialize(db_path)
            .context("failed to initialise local database")?;

        let (net_tx, rx_network) = mpsc::channel(NETWORK_EVENT_CHANNEL_CAPACITY);

        let mut network = NetworkManager::new(net_tx);
        network.initialize().await.context("failed to bind network endpoint")?;

        // Load identity if it exists.
        let local_identity = queries::get_local_identity(&db)?;
        let crypto = if let Some(rec) = local_identity {
            let mut iroh_seed = [0u8; 32];
            iroh_seed.copy_from_slice(&rec.node_id_bytes);
            
            // Recover iroh public key (NodeID) from the secret seed
            let iroh_secret = iroh::SecretKey::from_bytes(&iroh_seed);
            let public_node_id = iroh_secret.public();
            
            let mut x25519_secret = [0u8; 32];
            x25519_secret.copy_from_slice(&rec.x25519_secret);
            
            let identity = crate::crypto::Identity::from_bytes(*public_node_id.as_bytes(), x25519_secret);
            Some(CryptoManager::new(identity))
        } else {
            None
        };

        Ok(Self {
            db,
            crypto,
            network,
            rx_commands,
            tx_events,
            rx_network,
        })
    }

    /// Run the actor select loop. Never returns under normal operation.
    pub async fn run(mut self) {
        let mut flush_interval =
            tokio::time::interval(tokio::time::Duration::from_secs(QUEUE_FLUSH_INTERVAL_SECS));

        loop {
            tokio::select! {
                // ── UI Commands ──────────────────────────────────────────────
                Some(cmd) = self.rx_commands.recv() => {
                    if let Err(e) = self.handle_command(cmd).await {
                        tracing::error!("command handler error: {:?}", e);
                        self.emit(AppEvent::Error { message: format!("Internal error: {}", e) });
                    }
                }

                // ── Network Events ───────────────────────────────────────────
                Some(event) = self.rx_network.recv() => {
                    if let Err(e) = self.handle_network(event).await {
                        tracing::error!("network handler error: {:?}", e);
                    }
                }

                // ── Offline Queue Flush ───────────────────────────────────────
                _ = flush_interval.tick() => {
                    if let Err(e) = self.flush_offline_queue().await {
                        tracing::error!("queue flush error: {:?}", e);
                    }
                }
            }
        }
    }

    // ── Command Handlers ──────────────────────────────────────────────────────

    /// Route a `Command` to its single handler (RULES.md A-04).
    async fn handle_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::SendDirectMessage { target, plaintext } => {
                self.cmd_send_direct_message(target, plaintext).await
            }
            Command::SendFile { target, file_path } => {
                self.cmd_send_file(target, file_path).await
            }
            Command::NotifyReadReceipt { target, message_id } => {
                self.cmd_notify_read_receipt(target, message_id).await
            }
            Command::CreateGroup { name } => {
                self.cmd_create_group(name).await
            }
            Command::SendGroupMessage { topic, plaintext } => {
                self.cmd_send_group_message(topic, plaintext).await
            }
            Command::InviteToGroup { target, topic } => {
                self.cmd_invite_to_group(target, topic).await
            }
            Command::MarkVerified { node_id } => {
                self.cmd_mark_verified(node_id).await
            }
            Command::CreateIdentity { name } => {
                self.cmd_create_identity(name).await
            }
            Command::FinaliseIdentity => {
                self.cmd_finalise_identity().await
            }
            Command::ClearMessages => {
                self.cmd_clear_messages().await
            }
            Command::DeleteIdentity => {
                self.cmd_delete_identity().await
            }
        }
    }

    async fn cmd_send_direct_message(&mut self, target: String, plaintext: String) -> Result<()> {
        let id = Uuid::new_v4();
        let now = unix_now();

        // Store locally with status Queued first (RULES.md A-06).
        let record = queries::MessageRecord {
            id,
            msg_type:  "direct".to_owned(),
            target_id: target.clone(),
            sender_id: self.network.local_node_id().unwrap_or_default(),
            content:   plaintext.clone(),
            timestamp: now,
            status:    MessageStatus::Queued,
        };
        queries::insert_message(&self.db, &record).context("failed to store outgoing message")?;

        // Encrypt. If no session key exists yet, we cannot send — queue it.
        let crypto = match self.crypto.as_mut() {
            Some(c) => c,
            None => {
                tracing::warn!("no identity exists — messaging disabled");
                return Ok(());
            }
        };

        if !crypto.has_session(&target) {
            tracing::warn!(peer = %target, "no session key — message queued until handshake completes");
            return Ok(());
        }

        let ciphertext = match crypto.encrypt_direct(&target, plaintext.as_bytes()) {
            Ok(ct) => ct,
            Err(e) => {
                tracing::error!("encryption failed for peer {}: {:?}", target, e);
                return Ok(()); // leave as queued
            }
        };

        match self.network.send_direct(&target, ciphertext).await {
            Ok(()) => {
                // Must get mut ref again since borrow above dropped
                if let Some(c) = self.crypto.as_mut() {
                    c.ratchet_key_for(&target); // C-04
                }
                queries::advance_message_status(&self.db, &id, &MessageStatus::Sent)
                    .context("failed to advance message to Sent")?;
                self.emit(AppEvent::MessageStatusUpdate { id, status: MessageStatus::Sent });
            }
            Err(e) => {
                // Leave as Queued — flush_offline_queue will retry (RULES.md A-06).
                tracing::warn!(peer = %target, "send failed (queued): {:?}", e);
            }
        }
        Ok(())
    }

    async fn cmd_send_file(&mut self, _target: String, _file_path: PathBuf) -> Result<()> {
        // WIRE: file chunking + transfer protocol
        tracing::warn!("file transfer not yet implemented");
        Ok(())
    }

    async fn cmd_notify_read_receipt(&mut self, _target: String, message_id: Uuid) -> Result<()> {
        queries::advance_message_status(&self.db, &message_id, &MessageStatus::Read)
            .context("failed to mark message read")?;
        self.emit(AppEvent::MessageStatusUpdate {
            id: message_id,
            status: MessageStatus::Read,
        });
        Ok(())
    }

    async fn cmd_create_group(&mut self, name: String) -> Result<()> {
        let crypto = match self.crypto.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        let topic_id = Uuid::new_v4().to_string();
        let key = CryptoManager::generate_group_key();
        crypto.register_group_key(&topic_id, key);

        // Store the group with the key encrypted at rest (C-05).
        // WIRE: encrypt `key` under local password-derived key before storing.
        let record = queries::GroupRecord {
            topic_id:      topic_id.clone(),
            group_name:    name,
            symmetric_key: key.to_vec(), // TODO: encrypt at rest (C-05)
        };
        queries::insert_group(&self.db, &record).context("failed to store group")?;

        self.network.subscribe_group(&topic_id).await
            .context("failed to subscribe to new group topic")?;

        tracing::info!(topic = %topic_id, "group created");
        Ok(())
    }

    async fn cmd_send_group_message(&mut self, topic: String, plaintext: String) -> Result<()> {
        let crypto = match self.crypto.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };
        let ciphertext = crypto.encrypt_group(&topic, plaintext.as_bytes())
            .context("group encryption failed")?;

        if let Err(e) = self.network.broadcast_group(&topic, ciphertext).await {
            tracing::error!(topic = %topic, "group broadcast failed: {:?}", e);
            self.emit(AppEvent::Error {
                message: format!("Failed to send group message: {}", e),
            });
        }
        Ok(())
    }

    async fn cmd_invite_to_group(&mut self, target: String, topic: String) -> Result<()> {
        let crypto = match self.crypto.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };
        let key = crypto.group_key_bytes(&topic)
            .context("cannot invite — no group key found")?;

        // Encode invite as a JSON payload sent over the 1:1 E2EE channel.
        let invite_payload = format!(
            r#"{{"type":"group_invite","topic":"{}","key":"{}"}}"#,
            topic,
            hex::encode(key)
        );
        // Route through the normal send path so it benefits from queuing.
        self.cmd_send_direct_message(target, invite_payload).await
    }

    async fn cmd_mark_verified(&mut self, node_id: String) -> Result<()> {
        queries::mark_peer_verified(&self.db, &node_id)
            .context("failed to mark peer verified")?;
        tracing::info!(peer = %node_id, "peer marked as key-verified");
        Ok(())
    }

    async fn cmd_create_identity(&mut self, name: String) -> Result<()> {
        // 1. Generate Iroh identity (Ed25519)
        let mut iroh_seed = [0u8; 32];
        rand::prelude::RngCore::fill_bytes(&mut rand::rng(), &mut iroh_seed);
        let iroh_secret = iroh::SecretKey::from_bytes(&iroh_seed);
        let node_id = iroh_secret.public();
        
        // 2. Generate E2EE identity (X25519)
        let mut x25519_seed = [0u8; 32];
        rand::prelude::RngCore::fill_bytes(&mut rand::rng(), &mut x25519_seed);
        
        // 3. Persist to DB
        let record = queries::LocalIdentityRecord {
            display_name: name.clone(),
            node_id_bytes: iroh_seed.to_vec(), // We store the SEED to recover the secret key
            x25519_secret: x25519_seed.to_vec(),
        };
        queries::insert_local_identity(&self.db, &record).context("failed to save identity")?;
        
        // 4. Initialize live crypto state
        let identity = crate::crypto::Identity::from_bytes(*node_id.as_bytes(), x25519_seed);
        self.crypto = Some(CryptoManager::new(identity));

        // 5. Tell UI we're done
        self.emit(AppEvent::IdentityGenerated {
            display_name: name,
            node_id: node_id.to_string(),
        });
        Ok(())
    }

    async fn cmd_finalise_identity(&mut self) -> Result<()> {
        self.emit(AppEvent::SetupComplete);
        Ok(())
    }

    // ── Network Event Handlers ────────────────────────────────────────────────

    async fn handle_network(&mut self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::DirectMessage { from, payload } => {
                self.net_incoming_direct(from, payload).await
            }
            NetworkEvent::GroupMessage { topic, from, payload } => {
                self.net_incoming_group(topic, from, payload).await
            }
            NetworkEvent::PeerConnected { node_id, via_relay } => {
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: node_id,
                    online: true,
                    via_relay,
                });
                Ok(())
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: node_id,
                    online: false,
                    via_relay: false,
                });
                Ok(())
            }
        }
    }

    async fn net_incoming_direct(&mut self, from: String, payload: Vec<u8>) -> Result<()> {
        let crypto = match self.crypto.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        if !crypto.has_session(&from) {
            tracing::warn!(peer = %from, "received message but no session key — dropping");
            return Ok(());
        }

        let plaintext = match crypto.decrypt_direct(&from, &payload) {
            Ok(pt) => pt,
            Err(e) => {
                // C-03: auth failure → drop, do not retry.
                tracing::error!(peer = %from, "decryption failed (dropping): {:?}", e);
                self.emit(AppEvent::Error {
                    message: format!("Failed to decrypt message from peer — message dropped."),
                });
                return Ok(());
            }
        };

        crypto.ratchet_key_for(&from); // C-04

        let text = String::from_utf8(plaintext).context("message plaintext is not valid UTF-8")?;
        let id  = Uuid::new_v4();
        let now = unix_now();

        let record = queries::MessageRecord {
            id,
            msg_type:  "direct".to_owned(),
            target_id: from.clone(),
            sender_id: from.clone(),
            content:   text.clone(),
            timestamp: now,
            status:    MessageStatus::Delivered,
        };
        queries::insert_message(&self.db, &record).context("failed to store incoming message")?;

        self.emit(AppEvent::IncomingMessage { sender: from, id, plaintext: text, timestamp: now });
        Ok(())
    }

    async fn net_incoming_group(
        &mut self,
        topic: String,
        from: String,
        payload: Vec<u8>,
    ) -> Result<()> {
        let crypto = match self.crypto.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        let plaintext = match crypto.decrypt_group(&topic, &payload) {
            Ok(pt) => pt,
            Err(e) => {
                tracing::error!(topic = %topic, peer = %from, "group decryption failed: {:?}", e);
                return Ok(());
            }
        };

        let text = String::from_utf8(plaintext).context("group message is not valid UTF-8")?;
        let id  = Uuid::new_v4();
        let now = unix_now();

        self.emit(AppEvent::IncomingGroupMessage {
            topic,
            sender: from,
            id,
            plaintext: text,
            timestamp: now,
        });
        Ok(())
    }

    // ── Offline Queue Flush ───────────────────────────────────────────────────

    /// Retry all queued messages for reachable peers (RULES.md A-06).
    async fn flush_offline_queue(&mut self) -> Result<()> {
        let targets = queries::list_peers_with_queued_messages(&self.db)
            .context("failed to list peers with queued messages")?;

        for target in targets {
            let queued = queries::list_queued_messages(&self.db, &target)
                .context("failed to list queued messages")?;

            for msg in queued {
                let ciphertext = {
                    let crypto = match self.crypto.as_mut() {
                        Some(c) => c,
                        None => continue,
                    };
                    if !crypto.has_session(&target) {
                        continue; // cannot send without a session key
                    }
                    match crypto.encrypt_direct(&target, msg.content.as_bytes()) {
                        Ok(ct) => ct,
                        Err(e) => {
                            tracing::error!("flush encrypt failed: {:?}", e);
                            continue;
                        }
                    }
                };

                match self.network.send_direct(&target, ciphertext).await {
                    Ok(()) => {
                        // Must borrow mutably again
                        if let Some(c) = self.crypto.as_mut() {
                            c.ratchet_key_for(&target);
                        }
                        if let Err(e) = queries::advance_message_status(
                            &self.db, &msg.id, &MessageStatus::Sent,
                        ) {
                            tracing::error!("failed to advance flushed message status: {:?}", e);
                        } else {
                            self.emit(AppEvent::MessageStatusUpdate {
                                id:     msg.id,
                                status: MessageStatus::Sent,
                            });
                        }
                    }
                    Err(e) => {
                        tracing::debug!(peer = %target, "flush still unreachable: {:?}", e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn cmd_clear_messages(&mut self) -> Result<()> {
        queries::clear_all_messages(&self.db).context("failed to clear messages")?;
        self.emit(AppEvent::MessagesCleared);
        Ok(())
    }

    async fn cmd_delete_identity(&mut self) -> Result<()> {
        queries::delete_local_identity(&self.db).context("failed to delete identity")?;
        // Shutdown for now; onboarding will trigger on next launch.
        std::process::exit(0);
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Push an `AppEvent` to the broadcast channel consumed by the UI event listener.
    fn emit(&self, event: AppEvent) {
        // Ignore send errors — the UI may have shut down.
        let _ = self.tx_events.send(event);
    }
}

/// Returns the current time as a UTC Unix timestamp in seconds.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
