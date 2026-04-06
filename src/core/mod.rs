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
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::crypto::CryptoManager;
use crate::p2p::{NetworkEvent, NetworkManager};
use crate::storage::{self, queries};
use iroh_tickets::Ticket;

use commands::{
    AppEvent, ChatMessageData, ChatPreviewData, Command, ContactDirectoryData,
    GroupMessageData, GroupSelectionData, MessageStatus,
};

/// How often the offline queue flush task runs (RULES.md R-02).
const QUEUE_FLUSH_INTERVAL_SECS: u64 = 5;
/// Channel capacity for network events from the `NetworkManager`.
const NETWORK_EVENT_CHANNEL_CAPACITY: usize = 64;
/// Prefix used for direct-contact handshake frames.
const CONTACT_HANDSHAKE_MAGIC: &[u8; 4] = b"NC1H";
/// Frame type for an initial peer hello.
const CONTACT_HANDSHAKE_HELLO: u8 = 1;
/// Frame type for the hello acknowledgement.
const CONTACT_HANDSHAKE_HELLO_ACK: u8 = 2;
/// Maximum ticket bytes allowed in a direct-contact handshake frame.
const CONTACT_HANDSHAKE_MAX_TICKET_BYTES: usize = 768;
/// Maximum display-name bytes allowed in a direct-contact handshake frame.
const CONTACT_HANDSHAKE_MAX_NAME_BYTES: usize = 128;
/// How often to probe established direct peers for health.
const PEER_HEALTH_PING_INTERVAL_SECS: u64 = 5;
/// How long without any direct activity before we attempt a reconnect.
const PEER_HEALTH_RECONNECT_AFTER_SECS: u64 = 15;
/// Prefix used for encrypted direct-message frames.
const DIRECT_MESSAGE_MAGIC: &[u8; 4] = b"NC1D";
/// Direct frame version.
const DIRECT_MESSAGE_VERSION: u8 = 1;
/// Encrypted frame kind for an actual chat message.
const DIRECT_MESSAGE_KIND_TEXT: u8 = 1;
/// Encrypted frame kind for a delivery receipt.
const DIRECT_MESSAGE_KIND_DELIVERED: u8 = 2;
/// Encrypted frame kind for a read receipt.
const DIRECT_MESSAGE_KIND_READ: u8 = 3;
/// Encrypted frame kind for a health ping.
const DIRECT_MESSAGE_KIND_PING: u8 = 4;
/// Encrypted frame kind for a health pong.
const DIRECT_MESSAGE_KIND_PONG: u8 = 5;
/// Prefix used for group-sync frames over gossip.
const GOSSIP_SYNC_MAGIC: &[u8; 4] = b"NC1G";
/// Gossip sync frame kind for a request.
const GOSSIP_SYNC_KIND_REQUEST: u8 = 1;
/// Gossip sync frame kind for a response.
const GOSSIP_SYNC_KIND_RESPONSE: u8 = 2;

enum DirectPayload {
    Text { message_id: Uuid, plaintext: Vec<u8> },
    Receipt { message_id: Uuid, is_read: bool },
    Ping { nonce: Uuid },
    Pong { nonce: Uuid },
}

/// The central actor. Owns the DB connection, crypto state, and network handle.
///
/// Receives `Command`s from the Slint UI, emits `AppEvent`s back via
/// `slint::invoke_from_event_loop` (set up in `src/ui/mod.rs`).
pub struct NodeChatWorker {
    db:         Connection,
    crypto:     Option<CryptoManager>,
    network:    NetworkManager,
    peer_last_seen: HashMap<String, Instant>,
    peer_next_probe_at: HashMap<String, Instant>,
    peer_probe_failures: HashMap<String, u32>,
    failed_unlock_attempts: u32,
    lock_cooldown_until: Option<tokio::time::Instant>,
    peer_via_relay: HashMap<String, bool>,
    group_member_selection: Vec<String>,
    local_display_name: String,
    app_foreground: bool,
    last_background_at: Option<tokio::time::Instant>,
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


        // Load identity if it exists.
        let local_identity = queries::get_local_identity(&db)?;
        let mut iroh_secret = None;
        let local_display_name = local_identity.as_ref().map(|rec| rec.display_name.clone()).unwrap_or_default();
        let mut crypto = if let Some(rec) = local_identity {
            let mut iroh_seed = [0u8; 32];
            iroh_seed.copy_from_slice(&rec.node_id_bytes);
            
            // Recover iroh public key (NodeID) from the secret seed
            let secret = iroh::SecretKey::from_bytes(&iroh_seed);
            let public_node_id = secret.public();
            iroh_secret = Some(secret);
            
            let mut x25519_secret = [0u8; 32];
            x25519_secret.copy_from_slice(&rec.x25519_secret);
            
            let identity = crate::crypto::Identity::from_bytes(*public_node_id.as_bytes(), x25519_secret);
            Some(CryptoManager::new(identity))
        } else {
            None
        };

        if let Some(ref mut crypto) = crypto {
            for peer in queries::list_peers(&db)? {
                if peer.x25519_pubkey.len() == 64 {
                    if let Ok(bytes) = hex::decode(&peer.x25519_pubkey) {
                        if bytes.len() == 32 {
                            let mut public = [0u8; 32];
                            public.copy_from_slice(&bytes);
                            crypto.init_session(&peer.node_id, &public);
                        }
                    }
                }
            }
        }

        // Initialize network (possibly with our persisted secret)
        network.initialize(iroh_secret).await.context("failed to bind network endpoint")?;

        Ok(Self {
            db,
            crypto,
            network,
            peer_last_seen: HashMap::new(),
            peer_next_probe_at: HashMap::new(),
            peer_probe_failures: HashMap::new(),
            failed_unlock_attempts: 0,
            lock_cooldown_until: None,
            peer_via_relay: HashMap::new(),
            group_member_selection: Vec::new(),
            local_display_name,
            app_foreground: true,
            last_background_at: None,
            rx_commands,
            tx_events,
            rx_network,
        })
    }

    /// Hydrates the crypto engine with persisted group keys and joins gossip topics.
    pub async fn initialize_persisted_state(&mut self) -> Result<()> {
        let groups = queries::list_groups(&mut self.db)?;
        let mut group_ids = Vec::new();

        if let Some(ref mut crypto) = self.crypto {
            for group in groups {
                if group.symmetric_key.len() == 32 {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&group.symmetric_key);
                    crypto.register_group_key(&group.topic_id, key);
                    group_ids.push(group.topic_id.clone());
                }
            }
        }

        // Now that crypto is released, do the async network work
        let bootstrap = self.network.active_connections();
        for topic_id in group_ids {
            if let Err(e) = self.network.subscribe_group(&topic_id, bootstrap.clone()).await {
                tracing::error!(topic = %topic_id, "failed to re-subscribe to group: {:?}", e);
            } else {
                // Request initial sync for each group we re-joined
                let _ = self.broadcast_group_sync_request(&topic_id).await;
            }
        }
        Ok(())
    }

    /// Run the actor select loop. Never returns under normal operation.
    pub async fn run(mut self) {
        let mut flush_interval =
            tokio::time::interval(tokio::time::Duration::from_secs(QUEUE_FLUSH_INTERVAL_SECS));
        let mut health_interval =
            tokio::time::interval(Duration::from_secs(PEER_HEALTH_PING_INTERVAL_SECS));

        self.emit_network_status().await;
        self.emit_chat_list();
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
                    self.emit_network_status().await;
                }

                // ── Peer Health Probe ────────────────────────────────────────
                _ = health_interval.tick() => {
                    if let Err(e) = self.probe_peer_health().await {
                        tracing::error!("peer health probe error: {:?}", e);
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
            Command::AcceptGroupInvite { topic, group_name, symmetric_key } => {
                self.cmd_accept_group_invite(topic, group_name, symmetric_key).await
            }
            Command::ToggleGroupMemberSelection { peer_id } => {
                self.cmd_toggle_group_member_selection(peer_id).await
            }
            Command::SendGroupMessage { topic, plaintext } => {
                self.cmd_send_group_message(topic, plaintext).await
            }
            Command::InviteToGroup { target, topic } => {
                self.cmd_invite_to_group(target, topic).await
            }
            Command::ToggleVerified { node_id, verified } => {
                self.cmd_toggle_verified(node_id, verified).await
            }
            Command::CreateIdentity { name, pin } => {
                self.cmd_create_identity(name, pin).await
            }
            Command::FinaliseIdentity => {
                self.cmd_finalise_identity().await
            }
            Command::AddContact { ticket_or_id } => {
                self.cmd_add_contact(ticket_or_id).await
            }
            Command::ClearMessages { pin } => {
                self.cmd_clear_messages(pin).await
            }
            Command::ClearConversationHistory { target, is_group, pin } => {
                self.cmd_clear_conversation_history(target, is_group, pin).await
            }
            Command::DeleteConversation { target, is_group, pin } => {
                self.cmd_delete_conversation(target, is_group, pin).await
            }
            Command::RetryQueuedMessages { target } => {
                self.cmd_retry_queued_messages(target).await
            }
            Command::DeleteIdentity { pin } => {
                self.cmd_delete_identity(pin).await
            }
            Command::ForceDeleteIdentity => {
                self.cmd_force_delete_identity().await
            }
            Command::UnlockApp { pin } => {
                self.cmd_unlock_app(pin).await
            }
            Command::ChangePassword { current_pin, new_pin } => {
                self.cmd_change_password(current_pin, new_pin).await
            }
            Command::SetAppForeground { foreground } => {
                self.cmd_set_app_foreground(foreground).await
            }
            Command::RefreshLocalInfo => {
                self.cmd_refresh_local_info().await
            }
            Command::LoadConversation { target, is_group } => {
                self.cmd_load_conversation(target, is_group).await
            }
            Command::UpdateDisplayName { name } => {
                self.cmd_update_display_name(name).await
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
        self.emit_chat_list();
        self.emit_conversation_messages(&target, false)?;
        self.touch_peer_activity(&target);

        // Encrypt. If no session key exists yet, we cannot send — queue it.
        let has_crypto = self.crypto.is_some();
        if !has_crypto {
            tracing::warn!("no identity exists — messaging disabled");
            return Ok(());
        }

        let has_session = self
            .crypto
            .as_ref()
            .map(|c| c.has_session(&target))
            .unwrap_or(false);

        if !has_session {
            tracing::info!(peer = %target, "direct handshake needed before first message");
            if let Err(e) = self.send_contact_handshake(&target, false).await {
                tracing::error!(peer = %target, "failed to send contact handshake: {:?}", e);
                self.emit(AppEvent::Error {
                    message: format!("Failed to start handshake with {target}: {e}"),
                });
                return Ok(());
            }
            self.emit(AppEvent::PeerHandshakeStage {
                peer: target.clone(),
                stage: "handshake sent".to_owned(),
            });
            self.touch_peer_activity(&target);
            tracing::warn!(peer = %target, "no session key — message queued until handshake completes");
            self.emit(AppEvent::Error {
                message: format!(
                    "Waiting for handshake with {target}. The message was queued until the peer responds."
                ),
            });
            return Ok(());
        }

        let payload = self.build_direct_message_frame(id, plaintext.as_bytes());
        let ciphertext = match self.crypto.as_mut() {
            Some(c) => match c.encrypt_direct(&target, &payload) {
                Ok(ct) => ct,
                Err(e) => {
                    tracing::error!("encryption failed for peer {}: {:?}", target, e);
                    return Ok(());
                }
            },
            None => {
                tracing::error!("encryption failed for peer {}: no crypto state", target);
                return Ok(());
            }
        };

        if !self.network.has_connection(&target) {
            tracing::info!(peer = %target, "peer not currently connected — message stays queued");
            self.mark_peer_offline(&target);
            self.emit(AppEvent::Error {
                message: format!(
                    "Peer {target} is not connected right now. The message was queued and will send when they reconnect."
                ),
            });
            self.emit_chat_list();
            self.emit_conversation_messages(&target, false)?;
            return Ok(());
        }

        let dial_hint = self.peer_dial_hint(&target);

        match self.network.send_direct(&target, dial_hint.as_deref(), ciphertext).await {
            Ok(()) => {
                queries::advance_message_status(&self.db, &id, &MessageStatus::Sent)
                    .context("failed to advance message to Sent")?;
                self.emit(AppEvent::MessageStatusUpdate {
                    id,
                    target: target.clone(),
                    is_group: false,
                    status: MessageStatus::Sent,
                });
                self.touch_peer_activity(&target);
                self.emit_chat_list();
                self.emit_conversation_messages(&target, false)?;
            }
            Err(e) => {
                // Leave as Queued — flush_offline_queue will retry (RULES.md A-06).
                tracing::warn!(peer = %target, "send failed (queued): {:?}", e);
                self.mark_peer_offline(&target);
                self.emit(AppEvent::Error {
                    message: format!("Failed to send message to {target}: {e}"),
                });
                self.emit_chat_list();
                self.emit_conversation_messages(&target, false)?;
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
            target: _target.to_owned(),
            is_group: false,
            status: MessageStatus::Read,
        });
        self.emit_conversation_messages(&_target, false)?;
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

        let bootstrap = self.network.active_connections();
        self.network.subscribe_group(&topic_id, bootstrap).await
            .context("failed to subscribe to new group topic")?;

        // Request initial sync (though we are the founder, we might be re-joining on another device)
        if let Err(e) = self.broadcast_group_sync_request(&topic_id).await {
            tracing::warn!(topic = %topic_id, "failed to send initial group sync request: {:?}", e);
        }

        tracing::info!(topic = %topic_id, "group created");
        let selected_members = self.group_member_selection.clone();
        for target in selected_members {
            if let Err(e) = self.cmd_invite_to_group(target.clone(), topic_id.clone()).await {
                tracing::warn!(peer = %target, topic = %topic_id, "failed to invite selected group member: {:?}", e);
            }
        }
        self.group_member_selection.clear();
        self.emit_group_selection();
        self.emit_chat_list();
        Ok(())
    }

    async fn cmd_accept_group_invite(
        &mut self,
        topic: String,
        group_name: String,
        symmetric_key: String,
    ) -> Result<()> {
        let crypto = match self.crypto.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        let key_bytes = hex::decode(&symmetric_key)
            .with_context(|| format!("invalid group invite key for topic {topic:?}"))?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("invalid group invite key length for topic {topic}"));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        crypto.register_group_key(&topic, key);

        if queries::get_group(&self.db, &topic)?.is_none() {
            let record = queries::GroupRecord {
                topic_id: topic.clone(),
                group_name: group_name.clone(),
                symmetric_key: key.to_vec(), // TODO: encrypt at rest (C-05)
            };
            queries::insert_group(&self.db, &record).context("failed to store accepted group invite")?;
        }

        let bootstrap = self.network.active_connections();
        self.network
            .subscribe_group(&topic, bootstrap)
            .await
            .context("failed to subscribe to invited group topic")?;

        // Request initial sync to catch up on history
        if let Err(e) = self.broadcast_group_sync_request(&topic).await {
            tracing::warn!(topic = %topic, "failed to send group sync request: {:?}", e);
        }

        self.emit_chat_list();
        Ok(())
    }

    async fn cmd_toggle_group_member_selection(&mut self, peer_id: String) -> Result<()> {
        if let Some(pos) = self.group_member_selection.iter().position(|id| id == &peer_id) {
            self.group_member_selection.remove(pos);
        } else {
            self.group_member_selection.push(peer_id);
        }
        self.emit_group_selection();
        Ok(())
    }

    async fn cmd_send_group_message(&mut self, topic: String, plaintext: String) -> Result<()> {
        let crypto = match self.crypto.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };
        let id = Uuid::new_v4();
        let now = unix_now();
        let record = queries::MessageRecord {
            id,
            msg_type: "group".to_owned(),
            target_id: topic.clone(),
            sender_id: self.network.local_node_id().unwrap_or_default(),
            content: plaintext.clone(),
            timestamp: now,
            status: MessageStatus::Queued,
        };
        queries::insert_message(&self.db, &record).context("failed to store outgoing group message")?;
        self.emit_conversation_messages(&topic, true)?;

        let ciphertext = crypto.encrypt_group(&topic, plaintext.as_bytes())
            .context("group encryption failed")?;

        if let Err(e) = self.network.broadcast_group(&topic, ciphertext).await {
            tracing::error!(topic = %topic, "group broadcast failed: {:?}", e);
            self.emit(AppEvent::Error {
                message: format!("Failed to send group message: {}", e),
            });
        } else {
            queries::advance_message_status(&self.db, &id, &MessageStatus::Sent)
                .context("failed to advance group message to Sent")?;
            self.emit(AppEvent::MessageStatusUpdate {
                id,
                target: topic.clone(),
                is_group: true,
                status: MessageStatus::Sent,
            });
        }
        self.emit_chat_list();
        self.emit_conversation_messages(&topic, true)?;
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
        let group_name = queries::get_group(&self.db, &topic)?
            .map(|group| group.group_name)
            .unwrap_or_else(|| "Group Invite".to_owned());
        let invite_payload = serde_json::json!({
            "type": "group_invite",
            "topic": topic,
            "key": hex::encode(key),
            "group_name": group_name,
        })
        .to_string();
        // Route through the normal send path so it benefits from queuing.
        self.cmd_send_direct_message(target, invite_payload).await
    }

    async fn cmd_toggle_verified(&mut self, node_id: String, verified: bool) -> Result<()> {
        queries::mark_peer_verified(&self.db, &node_id, verified)
            .context("failed to update peer verification status")?;
        self.emit(AppEvent::PeerVerificationUpdated { peer: node_id.clone(), verified });
        self.emit_chat_list(); // Refresh badges
        Ok(())
    }

    async fn cmd_create_identity(&mut self, name: String, pin: String) -> Result<()> {
        // 1. Generate Iroh identity (Ed25519)
        let mut iroh_seed = [0u8; 32];
        rand::prelude::RngCore::fill_bytes(&mut rand::rng(), &mut iroh_seed);
        let iroh_secret = iroh::SecretKey::from_bytes(&iroh_seed);
        let node_id = iroh_secret.public();
        
        // 2. Generate E2EE identity (X25519)
        let mut x25519_seed = [0u8; 32];
        rand::prelude::RngCore::fill_bytes(&mut rand::rng(), &mut x25519_seed);
        
        // 3. Hash the PIN
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::digest(pin.as_bytes());
        for _ in 0..15_000 {
            hash = Sha256::digest(&hash);
        }
        let pin_hash = hex::encode(hash);

        // 4. Persist to DB
        let record = queries::LocalIdentityRecord {
            display_name: name.clone(),
            node_id_bytes: iroh_seed.to_vec(), // We store the SEED to recover the secret key
            x25519_secret: x25519_seed.to_vec(),
            pin_hash,
        };
        queries::insert_local_identity(&self.db, &record).context("failed to save identity")?;
        
        // 4. Initialize live crypto state
        let identity = crate::crypto::Identity::from_bytes(*node_id.as_bytes(), x25519_seed);
        self.crypto = Some(CryptoManager::new(identity));
        self.local_display_name = name.clone();

        // 5. Tell UI we're done
        self.emit(AppEvent::IdentityGenerated {
            display_name: name,
            node_id: node_id.to_string(),
        });
        self.emit_chat_list();
        Ok(())
    }

    async fn cmd_update_display_name(&mut self, name: String) -> Result<()> {
        queries::update_local_identity_name(&self.db, &name).context("failed to update name in DB")?;
        self.local_display_name = name.clone();
        
        // 1. Notify UI so headers and settings screens can update
        self.emit(AppEvent::IdentityUpdated {
            display_name: name.clone(),
        });

        // 2. Refresh chat list (if we show our own name anywhere)
        self.emit_chat_list();

        // 3. (Optional future enhancement): Notify active peers of the name change.
        // For now, names will update on the next handshake/message.
        
        Ok(())
    }

    async fn cmd_finalise_identity(&mut self) -> Result<()> {
        self.emit(AppEvent::SetupComplete);
        self.emit_chat_list();
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
                let peer_id = node_id.clone();
                self.touch_peer_activity(&peer_id);
                self.peer_via_relay.insert(peer_id.clone(), via_relay);
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: node_id.clone(),
                    online: true,
                    via_relay,
                    session_ready: self.has_session(&node_id),
                });
                if let Err(e) = self.flush_offline_queue_for_peer(&peer_id).await {
                    tracing::error!(peer = %peer_id, "failed to flush queued messages after connect: {:?}", e);
                }
                self.emit_chat_list();
                Ok(())
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                // Remove the stale connection immediately so has_connection() is accurate.
                self.network.remove_connection(&node_id);
                self.peer_last_seen.remove(&node_id);
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: node_id.clone(),
                    online: false,
                    via_relay: false,
                    session_ready: false,
                });
                self.emit_chat_list();
                Ok(())
            }
        }
    }

    async fn net_incoming_direct(&mut self, from: String, payload: Vec<u8>) -> Result<()> {
        if let Some((is_ack, their_x25519_public, their_ticket, their_display_name)) = Self::parse_contact_handshake(&payload) {
            let their_pub_hex = hex::encode(their_x25519_public);
            tracing::info!(peer = %from, ack = is_ack, "contact handshake received");
            self.touch_peer_activity(&from);
            self.emit(AppEvent::PeerHandshakeStage {
                peer: from.clone(),
                stage: if is_ack {
                    "ack received".to_owned()
                } else {
                    "handshake received".to_owned()
                },
            });
            match queries::get_peer(&self.db, &from)? {
                Some(peer) => {
                    if peer.x25519_pubkey != their_pub_hex {
                        queries::update_peer_x25519_pubkey(&self.db, &from, &their_pub_hex)
                            .context("failed to store peer public key")?;
                    }
                    if let Some(name) = &their_display_name {
                        if peer.display_name != *name && peer.display_name == "Pending peer" {
                            queries::insert_peer(&self.db, &storage::queries::PeerRecord {
                                node_id: from.clone(),
                                display_name: name.clone(),
                                endpoint_ticket: peer.endpoint_ticket.clone(),
                                x25519_pubkey: their_pub_hex.clone(),
                                verified: peer.verified,
                            }).context("failed to update handshake peer name")?;
                        }
                    }
                    if let Some(ticket) = &their_ticket {
                        if peer.endpoint_ticket != *ticket {
                            queries::update_peer_endpoint_ticket(&self.db, &from, ticket)
                                .context("failed to store peer endpoint ticket")?;
                        }
                    }
                }
                None => {
                    let record = storage::queries::PeerRecord {
                        node_id:         from.clone(),
                        display_name:    their_display_name.clone().unwrap_or_else(|| "Pending peer".to_owned()),
                        endpoint_ticket: their_ticket.clone().unwrap_or_default(),
                        x25519_pubkey:   their_pub_hex.clone(),
                        verified:        false,
                    };
                    queries::insert_peer(&self.db, &record).context("failed to store handshake peer")?;
                }
            }

            if let Some(crypto) = self.crypto.as_mut() {
                crypto.init_session(&from, &their_x25519_public);
            }

            tracing::info!(peer = %from, "contact session initialized");
            self.emit(AppEvent::PeerHandshakeStage {
                peer: from.clone(),
                stage: "session initialized".to_owned(),
            });
            self.emit(AppEvent::PeerOnlineStatus {
                peer: from.clone(),
                online: true,
                via_relay: false,
                session_ready: true,
            });
            self.emit(AppEvent::PeerContactDetails {
                peer: from.clone(),
                display_name: their_display_name.clone().unwrap_or_else(|| "Pending peer".to_owned()),
                endpoint_ticket: their_ticket.clone().unwrap_or_default(),
                verified: false,
            });
            if let Err(e) = self.flush_offline_queue_for_peer(&from).await {
                tracing::error!(peer = %from, "failed to flush queued messages after handshake: {:?}", e);
            }

            if !is_ack {
                tracing::info!(peer = %from, "sending handshake ack");
                if let Err(e) = self.send_contact_handshake(&from, true).await {
                    tracing::error!(peer = %from, "failed to send handshake ack: {:?}", e);
                    self.emit(AppEvent::Error {
                        message: format!("Failed to send handshake ack to {from}: {e}"),
                    });
                } else {
                    self.emit(AppEvent::PeerHandshakeStage {
                        peer: from.clone(),
                        stage: "ack sent".to_owned(),
                    });
                }
            }

            self.emit_chat_list();
            return Ok(());
        }

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

        self.touch_peer_activity(&from);

        match Self::parse_direct_payload(&plaintext) {
            Some(DirectPayload::Text { message_id, plaintext }) => {
                let text = String::from_utf8(plaintext).context("message plaintext is not valid UTF-8")?;
                let now = unix_now();

                let record = queries::MessageRecord {
                    id: message_id,
                    msg_type:  "direct".to_owned(),
                    target_id: from.clone(),
                    sender_id: from.clone(),
                    content:   text.clone(),
                    timestamp: now,
                    status:    MessageStatus::Delivered,
                };
                queries::insert_message(&self.db, &record).context("failed to store incoming message")?;

                self.emit(AppEvent::IncomingMessage {
                    sender: from.clone(),
                    id: message_id,
                    plaintext: text,
                    timestamp: now,
                });

                if let Err(e) = self.send_direct_receipt(&from, message_id, false).await {
                    tracing::debug!(peer = %from, msg = %message_id, "failed to send delivery receipt: {:?}", e);
                }

                self.emit_chat_list();
                self.emit_conversation_messages(&record.target_id, false)?;
            }
            Some(DirectPayload::Receipt { message_id, is_read }) => {
                let desired = if is_read {
                    MessageStatus::Read
                } else {
                    MessageStatus::Delivered
                };
                if let Some(target_id) = self.apply_direct_receipt(message_id, desired.clone())? {
                    self.emit(AppEvent::MessageStatusUpdate {
                        id: message_id,
                        target: target_id,
                        is_group: false,
                        status: desired,
                    });
                    self.emit_chat_list();
                }
            }
            Some(DirectPayload::Ping { nonce }) => {
                tracing::debug!(peer = %from, msg = %nonce, "direct health ping received");
                self.touch_peer_activity(&from);
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: from.clone(),
                    online: true,
                    via_relay: false,
                    session_ready: self.has_session(&from),
                });
                if let Err(e) = self.send_peer_health_pong(&from, nonce).await {
                    tracing::debug!(peer = %from, msg = %nonce, "failed to send health pong: {:?}", e);
                }
            }
            Some(DirectPayload::Pong { nonce }) => {
                tracing::debug!(peer = %from, msg = %nonce, "direct health pong received");
                self.emit(AppEvent::PeerOnlineStatus {
                    peer: from.clone(),
                    online: true,
                    via_relay: false,
                    session_ready: self.has_session(&from),
                });
                self.touch_peer_activity(&from);
            }
            None => {
                tracing::warn!(peer = %from, "direct payload did not match NodeChat frame format");
            }
        }
        Ok(())
    }

    async fn net_incoming_group(
        &mut self,
        topic: String,
        from: String,
        payload: Vec<u8>,
    ) -> Result<()> {
        // Check for Group Sync magic prefix (NC1G)
        if payload.starts_with(GOSSIP_SYNC_MAGIC) && payload.len() > 4 {
            let kind = payload[4];
            let inner = &payload[5..];
            match kind {
                GOSSIP_SYNC_KIND_REQUEST => {
                    return self.handle_group_sync_request(&topic, inner).await;
                }
                GOSSIP_SYNC_KIND_RESPONSE => {
                    return self.handle_group_sync_response(&topic, inner).await;
                }
                _ => {
                    tracing::warn!(topic = %topic, peer = %from, "unknown gossip sync kind: {}", kind);
                    return Ok(());
                }
            }
        }

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

        let record = queries::MessageRecord {
            id,
            msg_type: "group".to_owned(),
            target_id: topic.clone(),
            sender_id: from.clone(),
            content: text.clone(),
            timestamp: now,
            status: MessageStatus::Delivered,
        };
        queries::insert_message(&self.db, &record).context("failed to store incoming group message")?;

        self.emit(AppEvent::IncomingGroupMessage {
            topic,
            sender: from,
            id,
            plaintext: text,
            timestamp: now,
        });
        self.emit_chat_list();
        self.emit_conversation_messages(&record.target_id, true)?;
        Ok(())
    }

    // ── Offline Queue Flush ───────────────────────────────────────────────────

    /// Retry all queued messages for reachable peers (RULES.md A-06).
    async fn flush_offline_queue(&mut self) -> Result<()> {
        let targets = queries::list_peers_with_queued_messages(&self.db)
            .context("failed to list peers with queued messages")?;

        for target in targets {
            self.flush_offline_queue_for_peer(&target).await?;
        }
        Ok(())
    }

    /// Retry queued messages for one direct peer if a session is live.
    async fn flush_offline_queue_for_peer(&mut self, target: &str) -> Result<()> {
        let queued = queries::list_queued_messages(&self.db, target)
            .context("failed to list queued messages")?;
        if queued.is_empty() {
            return Ok(());
        }

        let has_session = self
            .crypto
            .as_ref()
            .map(|c| c.has_session(target))
            .unwrap_or(false);
        if !has_session {
            return Ok(());
        }

        for msg in queued {
            let frame = self.build_direct_message_frame(msg.id, msg.content.as_bytes());
            let ciphertext = match self.crypto.as_mut() {
                Some(crypto) => match crypto.encrypt_direct(target, &frame) {
                    Ok(ct) => ct,
                    Err(e) => {
                        tracing::error!(peer = %target, "flush encrypt failed: {:?}", e);
                        continue;
                    }
                },
                None => continue,
            };

            let dial_hint = self.peer_dial_hint(target);

            match self.network.send_direct(target, dial_hint.as_deref(), ciphertext).await {
                Ok(()) => {
                    if let Err(e) = queries::advance_message_status(
                        &self.db, &msg.id, &MessageStatus::Sent,
                    ) {
                        tracing::error!("failed to advance flushed message status: {:?}", e);
                    } else {
                        self.emit(AppEvent::MessageStatusUpdate {
                            id: msg.id,
                            target: target.to_owned(),
                            is_group: false,
                            status: MessageStatus::Sent,
                        });
                        self.emit_chat_list();
                        self.emit_conversation_messages(target, false)?;
                    }
                }
                Err(e) => {
                    tracing::debug!(peer = %target, "flush still unreachable: {:?}", e);
                    self.mark_peer_offline(target);
                }
            }
        }

        Ok(())
    }

    async fn cmd_add_contact(&mut self, ticket_or_id: String) -> Result<()> {
        let node_id = match iroh_tickets::endpoint::EndpointTicket::deserialize(&ticket_or_id) {
            Ok(ticket) => ticket.endpoint_addr().id.to_string(),
            Err(_) => {
                let parsed: iroh::EndpointId = ticket_or_id
                    .parse()
                    .with_context(|| format!("invalid peer ticket or node id: {ticket_or_id:?}"))?;
                parsed.to_string()
            }
        };

        let record = storage::queries::PeerRecord {
            node_id:         node_id.clone(),
            display_name:    "Pending peer".to_owned(),
            endpoint_ticket: ticket_or_id.clone(),
            x25519_pubkey:   "".to_owned(), // Placeholder until first handshake (C-04)
            verified:        false,
        };
        if queries::get_peer(&self.db, &node_id)?.is_none() {
            queries::insert_peer(&self.db, &record).context("failed to save contact")?;
        } else {
            queries::insert_peer(&self.db, &record).context("failed to update contact")?;
        }

        tracing::info!(peer = %node_id, "added new contact");

        if let Err(e) = self.send_contact_handshake(&node_id, false).await {
            tracing::warn!(peer = %node_id, "handshake timeout: {:?}", e);
            self.mark_peer_offline(&node_id);
            self.emit(AppEvent::PeerHandshakeStage {
                peer: node_id.clone(),
                stage: "Handshake failed (timeout)".to_owned(),
            });
        } else {
            self.emit(AppEvent::PeerHandshakeStage {
                peer: node_id.clone(),
                stage: "Handshake sent".to_owned(),
            });
        }

        self.emit(AppEvent::PeerContactDetails {
            peer: node_id.clone(),
            display_name: "Pending peer".to_owned(),
            endpoint_ticket: ticket_or_id,
            verified: false,
        });

        // Push initial network status update to UI (since peer counts might have changed)
        self.emit_chat_list();
        self.emit_network_status().await;

        Ok(())
    }

    async fn emit_network_status(&mut self) {
        let status = self.network.connection_status();
        self.emit(AppEvent::NetworkStatus {
            direct_peers: status.direct as i32,
            relay_peers:  status.relay as i32,
            is_offline:   status.direct == 0 && status.relay == 0,
        });
    }

    async fn cmd_clear_messages(&mut self, _pin: String) -> Result<()> {
        queries::clear_all_messages(&self.db).context("failed to clear messages")?;
        self.emit(AppEvent::MessagesCleared);
        self.emit_chat_list();
        Ok(())
    }
    async fn cmd_clear_conversation_history(&mut self, target: String, is_group: bool, _pin: String) -> Result<()> {
        queries::clear_conversation_messages(&self.db, &target)
            .context("failed to clear conversation history")?;
            
        self.emit(AppEvent::ConversationCleared { target: target.clone(), is_group });
        self.emit_chat_list();
        self.emit_conversation_messages(&target, is_group)?;
        Ok(())
    }

    async fn cmd_delete_conversation(&mut self, target: String, is_group: bool, _pin: String) -> Result<()> {
        queries::delete_conversation(&self.db, &target, is_group)
            .context("failed to delete conversation")?;
            
        if !is_group {
            self.network.remove_connection(&target);
        }
        
        self.emit(AppEvent::ConversationDeleted { target: target.clone(), is_group });
        self.emit_chat_list();
        Ok(())
    }

    async fn cmd_retry_queued_messages(&mut self, target: String) -> Result<()> {
        if let Err(e) = self.flush_offline_queue_for_peer(&target).await {
            tracing::error!(peer = %target, "manual queue retry failed: {:?}", e);
            self.emit(AppEvent::Error {
                message: format!("Retry now failed for {target}: {e}"),
            });
        }
        self.emit_chat_list();
        self.emit_conversation_messages(&target, false)?;
        Ok(())
    }

    async fn cmd_delete_identity(&mut self, pin: String) -> Result<()> {
        if !self.verify_pin(&pin)? {
            self.emit(AppEvent::DeleteIdentityFailed { 
                error: "Incorrect PIN. Application reset aborted.".to_owned() 
            });
            return Ok(());
        }

        queries::delete_local_identity(&self.db).context("failed to delete identity")?;
        self.crypto = None;
        self.emit(AppEvent::IdentityDeleted);
        Ok(())
    }

    async fn cmd_force_delete_identity(&mut self) -> Result<()> {
        queries::delete_local_identity(&self.db).context("failed to force delete identity")?;
        self.crypto = None;
        self.emit(AppEvent::IdentityDeleted);
        Ok(())
    }

    async fn cmd_unlock_app(&mut self, pin: String) -> Result<()> {
        // Check if we're in a cooldown period
        if let Some(until) = self.lock_cooldown_until {
            let now = tokio::time::Instant::now();
            if now < until {
                let secs = (until - now).as_secs() + 1;
                self.emit(AppEvent::UnlockFailed {
                    error: format!("Too many attempts. Try again in {}s.", secs),
                });
                return Ok(());
            } else {
                // Cooldown expired — reset
                self.lock_cooldown_until = None;
                self.failed_unlock_attempts = 0;
            }
        }

        // Load stored pin_hash from DB
        let identity = queries::get_local_identity(&self.db)
            .context("failed to load identity for unlock")?;

        let stored_hash = match identity {
            Some(ref id) => id.pin_hash.clone(),
            None => {
                // No identity — shouldn't happen, but let through
                self.emit(AppEvent::UnlockComplete);
                return Ok(());
            }
        };

        // If no PIN was set during setup, allow empty pass-through
        if stored_hash.is_empty() {
            self.failed_unlock_attempts = 0;
            self.emit(AppEvent::UnlockComplete);
            return Ok(());
        }

        // Hash the submitted PIN using the same 15,000-round SHA-256 stretch
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::digest(pin.as_bytes());
        for _ in 0..15_000 {
            hash = Sha256::digest(&hash);
        }
        let submitted_hash = hex::encode(hash);

        if submitted_hash == stored_hash {
            self.failed_unlock_attempts = 0;
            self.lock_cooldown_until = None;
            self.emit(AppEvent::UnlockComplete);
        } else {
            self.failed_unlock_attempts += 1;
            tracing::warn!(attempts = self.failed_unlock_attempts, "wrong PIN entered");

            if self.failed_unlock_attempts >= 5 {
                self.lock_cooldown_until =
                    Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(30));
                self.failed_unlock_attempts = 0;
                self.emit(AppEvent::UnlockFailed {
                    error: "Too many failed attempts. Locked for 30 seconds.".to_owned(),
                });
            } else {
                let remaining = 5 - self.failed_unlock_attempts;
                self.emit(AppEvent::UnlockFailed {
                    error: format!("Incorrect PIN. {} attempt(s) remaining.", remaining),
                });
            }
        }

        Ok(())
    }

    fn verify_pin(&self, pin: &str) -> Result<bool> {
        use sha2::{Digest, Sha256};

        let identity = queries::get_local_identity(&self.db)
            .context("failed to load identity for verification")?;
        
        let stored_hash = match identity {
            Some(id) => id.pin_hash,
            None => return Ok(true), // No identity = nothing to verify
        };

        if stored_hash.is_empty() {
            return Ok(true);
        }

        let mut hash = Sha256::digest(pin.as_bytes());
        for _ in 0..15_000 {
            hash = Sha256::digest(&hash);
        }
        let submitted_hash = hex::encode(hash);

        Ok(submitted_hash == stored_hash)
    }

    async fn cmd_change_password(&mut self, current_pin: String, new_pin: String) -> Result<()> {
        use sha2::{Digest, Sha256};

        // Load stored identity
        let identity = queries::get_local_identity(&self.db)
            .context("failed to load identity for password change")?;

        let stored_hash = match identity {
            Some(ref id) => id.pin_hash.clone(),
            None => {
                self.emit(AppEvent::PasswordChangeFailed {
                    error: "No identity found.".to_owned(),
                });
                return Ok(());
            }
        };

        // Verify current PIN (allow bypass if no PIN was set)
        if !stored_hash.is_empty() {
            let mut hash = Sha256::digest(current_pin.as_bytes());
            for _ in 0..15_000 {
                hash = Sha256::digest(&hash);
            }
            let submitted_hash = hex::encode(hash);

            if submitted_hash != stored_hash {
                self.emit(AppEvent::PasswordChangeFailed {
                    error: "Incorrect current password.".to_owned(),
                });
                return Ok(());
            }
        }

        // Hash the new PIN
        let mut new_hash = Sha256::digest(new_pin.as_bytes());
        for _ in 0..15_000 {
            new_hash = Sha256::digest(&new_hash);
        }
        let new_pin_hash = hex::encode(new_hash);

        // Persist
        queries::update_pin_hash(&self.db, &new_pin_hash)
            .context("failed to save new pin_hash")?;

        tracing::info!("access PIN updated successfully");
        self.emit(AppEvent::PasswordChanged);
        Ok(())
    }

    async fn cmd_set_app_foreground(&mut self, foreground: bool) -> Result<()> {
        let prev_foreground = self.app_foreground;
        self.app_foreground = foreground;
        tracing::info!(foreground, "android lifecycle state updated");

        if foreground {
            // Check if we need to lock after returning from background
            if !prev_foreground && self.crypto.is_some() {
                if let Some(bg_time) = self.last_background_at {
                    if bg_time.elapsed() >= tokio::time::Duration::from_secs(180) {
                        tracing::info!("app backgrounded for >3 mins — re-locking");
                        self.emit(AppEvent::AppLocked);
                    }
                }
            }
            self.last_background_at = None;

            self.emit_network_status().await;
            self.emit_chat_list();
            if let Err(e) = self.flush_offline_queue().await {
                tracing::error!("queue flush after foreground resume failed: {:?}", e);
            }
            for peer in self.network.active_connections() {
                self.touch_peer_activity(&peer);
            }
        } else {
            // App backgrounded — record the timestamp
            self.last_background_at = Some(tokio::time::Instant::now());
        }

        Ok(())
    }

    async fn cmd_refresh_local_info(&mut self) -> Result<()> {
        self.emit_network_status().await;
        self.emit_chat_list();
        self.emit_endpoint_ticket();
        Ok(())
    }

    async fn cmd_load_conversation(&mut self, target: String, is_group: bool) -> Result<()> {
        self.emit_conversation_messages(&target, is_group)?;
        if !is_group {
            let target_for_stage = target.clone();
            let has_session = self
                .crypto
                .as_ref()
                .map(|c| c.has_session(&target))
                .unwrap_or(false);
            self.emit(AppEvent::PeerOnlineStatus {
                peer: target.clone(),
                online: self.network.has_connection(&target),
                via_relay: false,
                session_ready: has_session,
            });
            self.emit(AppEvent::PeerHandshakeStage {
                peer: target_for_stage,
                stage: if has_session {
                    "session initialized".to_owned()
                } else {
                    "waiting for peer response".to_owned()
                },
            });
            if let Some(peer) = queries::get_peer(&self.db, &target)? {
                self.emit(AppEvent::PeerContactDetails {
                    peer: target.clone(),
                    display_name: peer.display_name,
                    endpoint_ticket: peer.endpoint_ticket,
                    verified: peer.verified,
                });
            }
            if has_session {
                if let Err(e) = self.send_read_receipts_for_peer(&target).await {
                    tracing::error!(peer = %target, "failed to send read receipts: {:?}", e);
                }
            }
        }
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Push an `AppEvent` to the broadcast channel consumed by the UI event listener.
    fn emit(&self, event: AppEvent) {
        // Ignore send errors — the UI may have shut down.
        let _ = self.tx_events.send(event);
    }

    /// Emit the current shareable ticket for the local endpoint.
    fn emit_endpoint_ticket(&self) {
        match self.network.endpoint_ticket() {
            Ok(ticket) => self.emit(AppEvent::EndpointTicketUpdated { ticket }),
            Err(e) => tracing::error!("failed to build endpoint ticket: {:?}", e),
        }
    }

    /// Build a small direct-contact handshake frame.
    fn build_contact_handshake(&self, is_ack: bool) -> Result<Vec<u8>> {
        let crypto = self
            .crypto
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("identity not initialised"))?;
        let ticket = self.network.endpoint_ticket()?;
        let ticket_bytes = ticket.as_bytes();
        let display_name_bytes = self.local_display_name.as_bytes();
        if ticket_bytes.len() > CONTACT_HANDSHAKE_MAX_TICKET_BYTES {
            return Err(anyhow::anyhow!(
                "endpoint ticket too large for handshake frame: {} bytes",
                ticket_bytes.len()
            ));
        }
        if display_name_bytes.len() > CONTACT_HANDSHAKE_MAX_NAME_BYTES {
            return Err(anyhow::anyhow!(
                "display name too large for handshake frame: {} bytes",
                display_name_bytes.len()
            ));
        }

        let mut frame = Vec::with_capacity(4 + 1 + 32 + 2 + ticket_bytes.len() + 2 + display_name_bytes.len());
        frame.extend_from_slice(CONTACT_HANDSHAKE_MAGIC);
        frame.push(if is_ack {
            CONTACT_HANDSHAKE_HELLO_ACK
        } else {
            CONTACT_HANDSHAKE_HELLO
        });
        frame.extend_from_slice(&crypto.x25519_public_bytes());
        frame.extend_from_slice(&(ticket_bytes.len() as u16).to_be_bytes());
        frame.extend_from_slice(ticket_bytes);
        frame.extend_from_slice(&(display_name_bytes.len() as u16).to_be_bytes());
        frame.extend_from_slice(display_name_bytes);
        Ok(frame)
    }

    /// Parse a direct-contact handshake frame.
    fn parse_contact_handshake(payload: &[u8]) -> Option<(bool, [u8; 32], Option<String>, Option<String>)> {
        if payload.len() < 4 + 1 + 32 + 2 || &payload[..4] != CONTACT_HANDSHAKE_MAGIC {
            return None;
        }

        let is_ack = match payload[4] {
            CONTACT_HANDSHAKE_HELLO => false,
            CONTACT_HANDSHAKE_HELLO_ACK => true,
            _ => return None,
        };

        let mut pubkey = [0u8; 32];
        pubkey.copy_from_slice(&payload[5..37]);
        let ticket_len_offset = 5 + 32;
        let ticket_len = u16::from_be_bytes([
            payload[ticket_len_offset],
            payload[ticket_len_offset + 1],
        ]) as usize;

        if payload.len() < ticket_len_offset + 2 + ticket_len {
            return None;
        }

        let ticket = if ticket_len == 0 {
            None
        } else {
            let ticket_start = ticket_len_offset + 2;
            let raw_ticket = &payload[ticket_start..ticket_start + ticket_len];
            Some(String::from_utf8(raw_ticket.to_vec()).ok()?)
        };

        let name_offset = ticket_len_offset + 2 + ticket_len;
        if payload.len() < name_offset + 2 {
            return Some((is_ack, pubkey, ticket, None));
        }

        let display_name_len = u16::from_be_bytes([
            payload[name_offset],
            payload[name_offset + 1],
        ]) as usize;

        if payload.len() != name_offset + 2 + display_name_len {
            return None;
        }

        let display_name = if display_name_len == 0 {
            None
        } else {
            let name_start = name_offset + 2;
            let raw_name = &payload[name_start..name_start + display_name_len];
            Some(String::from_utf8(raw_name.to_vec()).ok()?)
        };

        Some((is_ack, pubkey, ticket, display_name))
    }

    /// Build an encrypted direct-message frame that carries the message id.
    fn build_direct_message_frame(&self, message_id: Uuid, plaintext: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(4 + 1 + 1 + 16 + 4 + plaintext.len());
        frame.extend_from_slice(DIRECT_MESSAGE_MAGIC);
        frame.push(DIRECT_MESSAGE_VERSION);
        frame.push(DIRECT_MESSAGE_KIND_TEXT);
        frame.extend_from_slice(message_id.as_bytes());
        frame.extend_from_slice(&(plaintext.len() as u32).to_be_bytes());
        frame.extend_from_slice(plaintext);
        frame
    }

    /// Parse a JSON group invite payload carried over the direct channel.
    fn parse_group_invite_payload(content: &str) -> Option<(String, String, String)> {
        let value: Value = serde_json::from_str(content).ok()?;
        if value.get("type")?.as_str()? != "group_invite" {
            return None;
        }
        let topic = value.get("topic")?.as_str()?.to_owned();
        let key = value.get("key")?.as_str()?.to_owned();
        let group_name = value.get("group_name")?.as_str()?.to_owned();
        Some((topic, group_name, key))
    }

    /// Build an encrypted direct receipt frame for delivery/read acknowledgments.
    fn build_direct_receipt_frame(&self, message_id: Uuid, is_read: bool) -> Vec<u8> {
        let mut frame = Vec::with_capacity(4 + 1 + 1 + 16);
        frame.extend_from_slice(DIRECT_MESSAGE_MAGIC);
        frame.push(DIRECT_MESSAGE_VERSION);
        frame.push(if is_read {
            DIRECT_MESSAGE_KIND_READ
        } else {
            DIRECT_MESSAGE_KIND_DELIVERED
        });
        frame.extend_from_slice(message_id.as_bytes());
        frame
    }

    /// Build an encrypted direct health frame.
    fn build_direct_health_frame(&self, nonce: Uuid, is_pong: bool) -> Vec<u8> {
        let mut frame = Vec::with_capacity(4 + 1 + 1 + 16);
        frame.extend_from_slice(DIRECT_MESSAGE_MAGIC);
        frame.push(DIRECT_MESSAGE_VERSION);
        frame.push(if is_pong {
            DIRECT_MESSAGE_KIND_PONG
        } else {
            DIRECT_MESSAGE_KIND_PING
        });
        frame.extend_from_slice(nonce.as_bytes());
        frame
    }

    /// Parse an encrypted direct payload into a text message or receipt.
    fn parse_direct_payload(payload: &[u8]) -> Option<DirectPayload> {
        if payload.len() < 4 + 1 + 1 + 16 || &payload[..4] != DIRECT_MESSAGE_MAGIC {
            return None;
        }
        if payload[4] != DIRECT_MESSAGE_VERSION {
            return None;
        }

        let mut id_bytes = [0u8; 16];
        id_bytes.copy_from_slice(&payload[6..22]);
        let message_id = Uuid::from_bytes(id_bytes);

        match payload[5] {
            DIRECT_MESSAGE_KIND_TEXT => {
                if payload.len() < 22 + 4 {
                    return None;
                }
                let len = u32::from_be_bytes([
                    payload[22],
                    payload[23],
                    payload[24],
                    payload[25],
                ]) as usize;
                if payload.len() != 26 + len {
                    return None;
                }
                Some(DirectPayload::Text {
                    message_id,
                    plaintext: payload[26..].to_vec(),
                })
            }
            DIRECT_MESSAGE_KIND_DELIVERED => Some(DirectPayload::Receipt {
                message_id,
                is_read: false,
            }),
            DIRECT_MESSAGE_KIND_READ => Some(DirectPayload::Receipt {
                message_id,
                is_read: true,
            }),
            DIRECT_MESSAGE_KIND_PING => Some(DirectPayload::Ping { nonce: message_id }),
            DIRECT_MESSAGE_KIND_PONG => Some(DirectPayload::Pong { nonce: message_id }),
            _ => None,
        }
    }

    /// Send a direct delivery/read receipt back to the peer.
    async fn send_direct_receipt(&mut self, node_id: &str, message_id: Uuid, is_read: bool) -> Result<()> {
        let payload = self.build_direct_receipt_frame(message_id, is_read);
        let dial_hint = self.peer_dial_hint(node_id);
        let ciphertext = match self.crypto.as_mut() {
            Some(c) => c
                .encrypt_direct(node_id, &payload)
                .context("failed to encrypt direct receipt")?,
            None => return Ok(()),
        };

        self.network
            .send_direct(node_id, dial_hint.as_deref(), ciphertext)
            .await
            .with_context(|| format!("failed to send direct receipt to {node_id}"))?;
        Ok(())
    }

    /// Send a ping to an established peer to confirm the direct session is still alive.
    async fn send_peer_health_ping(&mut self, node_id: &str) -> Result<()> {
        let nonce = Uuid::new_v4();
        let payload = self.build_direct_health_frame(nonce, false);
        let dial_hint = self.peer_dial_hint(node_id);
        let ciphertext = match self.crypto.as_mut() {
            Some(c) => c
                .encrypt_direct(node_id, &payload)
                .context("failed to encrypt health ping")?,
            None => return Ok(()),
        };

        self.network
            .send_direct(node_id, dial_hint.as_deref(), ciphertext)
            .await
            .with_context(|| format!("failed to send health ping to {node_id}"))?;
        Ok(())
    }

    /// Send a pong reply to confirm the peer is still reachable.
    async fn send_peer_health_pong(&mut self, node_id: &str, nonce: Uuid) -> Result<()> {
        let payload = self.build_direct_health_frame(nonce, true);
        let dial_hint = self.peer_dial_hint(node_id);
        let ciphertext = match self.crypto.as_mut() {
            Some(c) => c
                .encrypt_direct(node_id, &payload)
                .context("failed to encrypt health pong")?,
            None => return Ok(()),
        };

        self.network
            .send_direct(node_id, dial_hint.as_deref(), ciphertext)
            .await
            .with_context(|| format!("failed to send health pong to {node_id}"))?;
        Ok(())
    }

    /// Record the latest time we heard from a peer.
    fn touch_peer_activity(&mut self, peer: &str) {
        self.peer_last_seen.insert(peer.to_owned(), Instant::now());
        self.peer_next_probe_at.remove(peer);
        self.peer_probe_failures.remove(peer);
    }

    /// Schedule the next health probe for a peer using a simple capped backoff.
    fn schedule_peer_probe_backoff(&mut self, peer: &str) {
        let failures = self
            .peer_probe_failures
            .entry(peer.to_owned())
            .and_modify(|n| *n = n.saturating_add(1))
            .or_insert(1);

        let step = failures.saturating_sub(1).min(5);
        let seconds = 5u64.saturating_mul(1u64 << step);
        let delay = Duration::from_secs(seconds.min(300));
        self.peer_next_probe_at
            .insert(peer.to_owned(), Instant::now() + delay);
    }

    /// Probe the direct peers we already know about and reconnect stale ones.
    async fn probe_peer_health(&mut self) -> Result<()> {
        if !self.app_foreground {
            return Ok(());
        }

        let now = Instant::now();
        let peers = queries::list_peers(&self.db).context("failed to list peers for health probe")?;

        for peer in peers {
            if let Some(next_probe) = self.peer_next_probe_at.get(&peer.node_id).copied() {
                if now < next_probe {
                    continue;
                }
            }

            let has_session = self
                .crypto
                .as_ref()
                .map(|crypto| crypto.has_session(&peer.node_id))
                .unwrap_or(false);
            let has_connection = self.network.has_connection(&peer.node_id);
            // Even if we don't have a connection, we want to probe them
            // occasionally to see if they've come online.

            let last_seen = self
                .peer_last_seen
                .get(&peer.node_id)
                .copied()
                .unwrap_or_else(|| {
                    now.checked_sub(Duration::from_secs(PEER_HEALTH_PING_INTERVAL_SECS))
                        .unwrap_or(now)
                });
            let idle_for = now.saturating_duration_since(last_seen);

            if has_connection && has_session {
                if idle_for >= Duration::from_secs(PEER_HEALTH_RECONNECT_AFTER_SECS) {
                    tracing::info!(peer = %peer.node_id, "peer health stale — attempting reconnect");
                    if let Err(e) = self.send_contact_handshake(&peer.node_id, false).await {
                        tracing::warn!(peer = %peer.node_id, "health reconnect failed: {:?}", e);
                        self.mark_peer_offline(&peer.node_id);
                        self.schedule_peer_probe_backoff(&peer.node_id);
                    } else {
                        self.emit(AppEvent::PeerHandshakeStage {
                            peer: peer.node_id.clone(),
                            stage: "handshake sent".to_owned(),
                        });
                    }
                } else if idle_for >= Duration::from_secs(PEER_HEALTH_PING_INTERVAL_SECS) {
                    if let Err(e) = self.send_peer_health_ping(&peer.node_id).await {
                        tracing::warn!(peer = %peer.node_id, "health ping failed: {:?}", e);
                        self.mark_peer_offline(&peer.node_id);
                        self.schedule_peer_probe_backoff(&peer.node_id);
                    }
                }
            } else if idle_for >= Duration::from_secs(PEER_HEALTH_RECONNECT_AFTER_SECS) {
                tracing::info!(peer = %peer.node_id, "peer stale — reconnecting");
                if let Err(e) = self.send_contact_handshake(&peer.node_id, false).await {
                    tracing::warn!(peer = %peer.node_id, "reconnect failed: {:?}", e);
                    self.mark_peer_offline(&peer.node_id);
                    self.schedule_peer_probe_backoff(&peer.node_id);
                }
            }
        }

        Ok(())
    }

    /// Mark a local outgoing message as delivered/read after a receipt arrives.
    fn apply_direct_receipt(&self, message_id: Uuid, status: MessageStatus) -> Result<Option<String>> {
        let message = match queries::get_message(&self.db, &message_id)? {
            Some(message) => message,
            None => return Ok(None),
        };

        let next_status = match status {
            MessageStatus::Delivered => {
                if message.status == MessageStatus::Queued {
                    MessageStatus::Sent
                } else {
                    MessageStatus::Delivered
                }
            }
            MessageStatus::Read => MessageStatus::Read,
            _ => status,
        };

        if message.status == next_status {
            return Ok(Some(message.target_id));
        }

        if message.status == MessageStatus::Queued && next_status == MessageStatus::Delivered {
            queries::advance_message_status(&self.db, &message_id, &MessageStatus::Sent)?;
        }
        if next_status == MessageStatus::Read && message.status == MessageStatus::Queued {
            queries::advance_message_status(&self.db, &message_id, &MessageStatus::Sent)?;
            queries::advance_message_status(&self.db, &message_id, &MessageStatus::Delivered)?;
        } else if message.status == MessageStatus::Sent && next_status == MessageStatus::Read {
            queries::advance_message_status(&self.db, &message_id, &MessageStatus::Delivered)?;
        }

        if next_status != MessageStatus::Queued && next_status != MessageStatus::Sent {
            match queries::get_message(&self.db, &message_id)? {
                Some(current) if current.status != next_status => {
                    queries::advance_message_status(&self.db, &message_id, &next_status)?;
                }
                Some(_) => {}
                None => tracing::debug!(msg = %message_id, "receipt arrived for missing message"),
            }
        }

        Ok(Some(message.target_id))
    }

    /// Send read receipts for every inbound direct message currently in the open thread.
    async fn send_read_receipts_for_peer(&mut self, target: &str) -> Result<()> {
        let messages = queries::list_messages(&self.db, target)
            .context("failed to list conversation messages for read receipts")?;

        for msg in messages.into_iter().filter(|msg| msg.sender_id == target && msg.status != MessageStatus::Read) {
            if let Err(e) = self.send_direct_receipt(target, msg.id, true).await {
                tracing::debug!(peer = %target, msg = %msg.id, "failed to send read receipt: {:?}", e);
                continue;
            }
            if msg.status == MessageStatus::Delivered || msg.status == MessageStatus::Sent {
                if let Err(e) = queries::advance_message_status(&self.db, &msg.id, &MessageStatus::Read) {
                    tracing::debug!(peer = %target, msg = %msg.id, "failed to mark local message read: {:?}", e);
                } else {
                    self.emit(AppEvent::MessageStatusUpdate {
                        id: msg.id,
                        target: target.to_owned(),
                        is_group: false,
                        status: MessageStatus::Read,
                    });
                }
            }
        }

        self.emit_chat_list();
        self.emit_conversation_messages(target, false)?;
        Ok(())
    }

    /// Send our handshake frame to a peer using the stored dial hint or NodeId.
    async fn send_contact_handshake(&mut self, node_id: &str, is_ack: bool) -> Result<()> {
        let payload = self.build_contact_handshake(is_ack)?;
        let dial_hint = self.peer_dial_hint(node_id);
        tracing::info!(peer = %node_id, ack = is_ack, "sending contact handshake");
        self.network
            .send_direct(node_id, dial_hint.as_deref(), payload)
            .await
            .context("failed to send contact handshake")?;
        Ok(())
    }

    /// Resolve the best dialing hint for a peer: prefer the imported ticket string.
    fn peer_dial_hint(&self, node_id: &str) -> Option<String> {
        queries::get_peer(&self.db, node_id)
            .ok()
            .flatten()
            .and_then(|peer| {
                if peer.endpoint_ticket.trim().is_empty() {
                    None
                } else {
                    Some(peer.endpoint_ticket)
                }
            })
    }

    /// Rebuild and push the chat list from SQLite to the UI.
    fn emit_chat_list(&self) {
        match queries::list_chat_previews(&self.db) {
            Ok(previews) => {
                let chats = previews
                    .into_iter()
                    .map(|preview| ChatPreviewData {
                        is_online: self.network.has_connection(&preview.id),
                        is_session_ready: self.has_session(&preview.id),
                        id: preview.id,
                        name: preview.name,
                        initials: preview.initials,
                        last_message: preview.last_message,
                        timestamp: preview.timestamp,
                        unread: preview.unread,
                        is_group: preview.is_group,
                        is_relay: preview.is_relay,
                        is_queued: preview.is_queued,
                        is_verified: preview.is_verified,
                    })
                    .collect();
                self.emit(AppEvent::ChatsUpdated { chats });
                self.emit_contact_directory();
                self.emit_group_selection();
            }
            Err(e) => {
                tracing::error!("failed to rebuild chat list: {:?}", e);
            }
        }
    }

    /// Rebuild and push the contacts directory from SQLite to the UI.
    fn emit_contact_directory(&self) {
        match queries::list_peers(&self.db) {
            Ok(peers) => {
                let contacts = peers
                    .into_iter()
                    .map(|contact| {
                        let initials = queries::derive_initials(&contact.display_name);
                        ContactDirectoryData {
                            id: contact.node_id.clone(),
                            name: contact.display_name,
                            initials,
                            node_id: contact.node_id.clone(),
                            is_online: self.network.has_connection(&contact.node_id),
                            is_session_ready: self.has_session(&contact.node_id),
                            is_relay: self.peer_via_relay.get(&contact.node_id).copied().unwrap_or(false),
                            is_verified: contact.verified,
                        }
                    })
                    .collect();
                self.emit(AppEvent::ContactsUpdated { contacts });
            }
            Err(e) => {
                tracing::error!("failed to list peers for UI directory update: {:?}", e);
            }
        }
    }

    /// Rebuild and push the selectable group members from SQLite to the UI.
    fn emit_group_selection(&self) {
        match queries::list_peers(&self.db) {
            Ok(peers) => {
                let contacts = peers
                    .into_iter()
                    .map(|peer| GroupSelectionData {
                        is_selected: self.group_member_selection.iter().any(|id| id == &peer.node_id),
                        id: peer.node_id.clone(),
                        name: peer.display_name.clone(),
                        initials: peer
                            .display_name
                            .split_whitespace()
                            .filter_map(|part| part.chars().next())
                            .take(2)
                            .map(|ch| ch.to_ascii_uppercase())
                            .collect::<String>(),
                        is_online: self.network.has_connection(&peer.node_id),
                    })
                    .collect::<Vec<_>>();
                let selected_count = self.group_member_selection.len() as i32;
                self.emit(AppEvent::GroupSelectionUpdated { contacts, selected_count });
            }
            Err(e) => tracing::error!("failed to rebuild group selection list: {:?}", e),
        }
    }

    /// Immediately clear cached reachability for a peer after a failed send.
    fn mark_peer_offline(&self, node_id: &str) {
        if !self.app_foreground {
            return;
        }
        self.network.remove_connection(node_id);
        self.emit(AppEvent::PeerOnlineStatus {
            peer: node_id.to_owned(),
            online: false,
            via_relay: false,
            session_ready: false,
        });
        self.emit_chat_list();
    }

    /// Rebuild and push the active conversation thread from SQLite to the UI.
    fn emit_conversation_messages(&self, target: &str, is_group: bool) -> Result<()> {
        let local_node_id = self.network.local_node_id().unwrap_or_default();

        if is_group {
            let rows = queries::list_messages(&self.db, target)?
                .into_iter()
                .map(|msg| {
                    let sender_name = if msg.sender_id == local_node_id {
                        String::new()
                    } else {
                        queries::get_peer(&self.db, &msg.sender_id)?
                            .map(|peer| peer.display_name)
                            .unwrap_or_else(|| msg.sender_id.clone())
                    };

                    Ok(GroupMessageData {
                        id: msg.id.to_string(),
                        text: msg.content,
                        timestamp: msg.timestamp.to_string(),
                        is_mine: msg.sender_id == local_node_id,
                        sender_name,
                        status: msg.status.as_str().to_owned(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            self.emit(AppEvent::GroupConversationLoaded {
                topic: target.to_owned(),
                messages: rows,
            });
        } else {
            let rows = queries::list_messages(&self.db, target)?
                .into_iter()
                .map(|msg| {
                    let invite = Self::parse_group_invite_payload(&msg.content);
                    let invite_group_name = invite.as_ref().map(|invite| invite.1.clone()).unwrap_or_default();
                    let invite_topic_id = invite.as_ref().map(|invite| invite.0.clone()).unwrap_or_default();
                    let invite_key = invite.as_ref().map(|invite| invite.2.clone()).unwrap_or_default();
                    let invite_is_joined = invite
                        .as_ref()
                        .and_then(|invite| queries::get_group(&self.db, &invite.0).ok().flatten())
                        .is_some();
                    Ok(ChatMessageData {
                        id: msg.id.to_string(),
                        text: if invite.is_some() {
                            invite_group_name.clone()
                        } else {
                            msg.content.clone()
                        },
                        timestamp: msg.timestamp.to_string(),
                        is_mine: msg.sender_id == local_node_id,
                        status: msg.status.as_str().to_owned(),
                        is_ephemeral: false,
                        ttl_seconds: 0,
                        is_group_invite: invite.is_some(),
                        invite_group_name,
                        invite_topic_id,
                        invite_key,
                        invite_is_joined,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            self.emit(AppEvent::DirectConversationLoaded {
                target: target.to_owned(),
                messages: rows,
            });
        }

        Ok(())
    }

    /// Build a group-sync request frame.
    fn build_group_sync_request_frame(&self, last_ts: i64) -> Vec<u8> {
        let mut frame = Vec::with_capacity(4 + 1 + 8);
        frame.extend_from_slice(GOSSIP_SYNC_MAGIC);
        frame.push(GOSSIP_SYNC_KIND_REQUEST);
        frame.extend_from_slice(&last_ts.to_be_bytes());
        frame
    }

    /// Build a group-sync response frame containing many messages.
    fn build_group_sync_response_frame(&self, topic: &str, messages: &[queries::MessageRecord]) -> Result<Vec<u8>> {
        let crypto = self.crypto.as_ref().context("no crypto")?;
        let mut inner = Vec::new();
        // Format: [count: u32] [[id: 16b] [sender_id_len: u16] [sender_id] [ts: i64] [content_len: u32] [content]]...
        inner.extend_from_slice(&(messages.len() as u32).to_be_bytes());
        for msg in messages {
            inner.extend_from_slice(msg.id.as_bytes());
            inner.extend_from_slice(&(msg.sender_id.len() as u16).to_be_bytes());
            inner.extend_from_slice(msg.sender_id.as_bytes());
            inner.extend_from_slice(&msg.timestamp.to_be_bytes());
            let content = msg.content.as_bytes();
            inner.extend_from_slice(&(content.len() as u32).to_be_bytes());
            inner.extend_from_slice(content);
        }

        let ciphertext = crypto.encrypt_group(topic, &inner)?;
        let mut frame = Vec::with_capacity(4 + 1 + ciphertext.len());
        frame.extend_from_slice(GOSSIP_SYNC_MAGIC);
        frame.push(GOSSIP_SYNC_KIND_RESPONSE);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    async fn broadcast_group_sync_request(&mut self, topic: &str) -> Result<()> {
        let messages = queries::list_messages(&self.db, topic)?;
        let last_ts = messages.last().map(|m| m.timestamp).unwrap_or(0);
        let frame = self.build_group_sync_request_frame(last_ts);
        self.network.broadcast_group(topic, frame).await
    }

    async fn handle_group_sync_request(&mut self, topic: &str, payload: &[u8]) -> Result<()> {
        if payload.len() < 8 { return Ok(()); }
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(&payload[0..8]);
        let since_ts = i64::from_be_bytes(ts_bytes);

        // Limit the number of messages in one sync response to avoid gossip overflow
        let messages = queries::list_messages_since(&self.db, topic, since_ts, 50)?;
        if messages.is_empty() { return Ok(()); }

        let frame = self.build_group_sync_response_frame(topic, &messages)?;
        self.network.broadcast_group(topic, frame).await
    }

    async fn handle_group_sync_response(&mut self, topic: &str, payload: &[u8]) -> Result<()> {
        let crypto = match self.crypto.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };
        let inner = crypto.decrypt_group(topic, payload)?;
        if inner.len() < 4 { return Ok(()); }

        let mut pos = 0;
        let count = u32::from_be_bytes(inner[0..4].try_into()?);
        pos += 4;

        let mut imported = 0;
        for _ in 0..count {
            if pos + 16 + 2 > inner.len() { break; }
            let id = Uuid::from_slice(&inner[pos..pos+16])?;
            pos += 16;
            let sender_id_len = u16::from_be_bytes(inner[pos..pos+2].try_into()?) as usize;
            pos += 2;
            if pos + sender_id_len + 8 + 4 > inner.len() { break; }
            let sender_id = String::from_utf8_lossy(&inner[pos..pos+sender_id_len]).to_string();
            pos += sender_id_len;
            let timestamp = i64::from_be_bytes(inner[pos..pos+8].try_into()?);
            pos += 8;
            let content_len = u32::from_be_bytes(inner[pos..pos+4].try_into()?) as usize;
            pos += 4;
            if pos + content_len > inner.len() { break; }
            let content = String::from_utf8_lossy(&inner[pos..pos+content_len]).to_string();
            pos += content_len;

            let record = queries::MessageRecord {
                id,
                msg_type: "group".to_owned(),
                target_id: topic.to_owned(),
                sender_id,
                content,
                timestamp,
                status: MessageStatus::Sent, // Synced messages are 'Sent'
            };
            if queries::insert_message(&self.db, &record).is_ok() {
                imported += 1;
            }
        }

        if imported > 0 {
            tracing::info!(topic = %topic, imported = imported, "imported group messages via gossip sync");
            self.emit_conversation_messages(topic, true)?;
            self.emit_chat_list();
        }

        Ok(())
    }

    /// Returns `true` if a live E2EE session key exists for the given peer.
    fn has_session(&self, node_id: &str) -> bool {
        self.crypto
            .as_ref()
            .map(|crypto| crypto.has_session(node_id))
            .unwrap_or(false)
    }
}

/// Returns the current time as a UTC Unix timestamp in seconds.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
}
