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
const QUEUE_FLUSH_INTERVAL_SECS: u64 = 10;
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
/// How often to probe established direct peers for health.
const PEER_HEALTH_PING_INTERVAL_SECS: u64 = 20;
/// How long without any direct activity before we attempt a reconnect.
const PEER_HEALTH_RECONNECT_AFTER_SECS: u64 = 40;
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
    peer_via_relay: HashMap<String, bool>,
    group_member_selection: Vec<String>,
    app_foreground: bool,
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
            peer_via_relay: HashMap::new(),
            group_member_selection: Vec::new(),
            app_foreground: true,
            rx_commands,
            tx_events,
            rx_network,
        })
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
            Command::ToggleGroupMemberSelection { peer_id } => {
                self.cmd_toggle_group_member_selection(peer_id).await
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
            Command::AddContact { ticket_or_id, display_name } => {
                self.cmd_add_contact(ticket_or_id, display_name).await
            }
            Command::ClearMessages => {
                self.cmd_clear_messages().await
            }
            Command::ClearConversationHistory { target, is_group } => {
                self.cmd_clear_conversation_history(target, is_group).await
            }
            Command::DeleteConversation { target, is_group } => {
                self.cmd_delete_conversation(target, is_group).await
            }
            Command::RetryQueuedMessages { target } => {
                self.cmd_retry_queued_messages(target).await
            }
            Command::DeleteIdentity => {
                self.cmd_delete_identity().await
            }
            Command::UnlockApp => {
                self.cmd_unlock_app().await
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

        self.network.subscribe_group(&topic_id).await
            .context("failed to subscribe to new group topic")?;

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
        self.emit_chat_list();
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
                    peer: node_id,
                    online: true,
                    via_relay,
                });
                if let Err(e) = self.flush_offline_queue_for_peer(&peer_id).await {
                    tracing::error!(peer = %peer_id, "failed to flush queued messages after connect: {:?}", e);
                }
                Ok(())
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                if self.app_foreground {
                    self.emit(AppEvent::PeerOnlineStatus {
                        peer: node_id,
                        online: false,
                        via_relay: false,
                    });
                }
                Ok(())
            }
        }
    }

    async fn net_incoming_direct(&mut self, from: String, payload: Vec<u8>) -> Result<()> {
        if let Some((is_ack, their_x25519_public, their_ticket)) = Self::parse_contact_handshake(&payload) {
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
                        display_name:    from.clone(),
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

    async fn cmd_add_contact(&mut self, ticket_or_id: String, display_name: String) -> Result<()> {
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
            display_name:    display_name.clone(),
            endpoint_ticket: ticket_or_id.clone(),
            x25519_pubkey:   "".to_owned(), // Placeholder until first handshake (C-04)
            verified:        false,
        };
        if queries::get_peer(&self.db, &node_id)?.is_none() {
            queries::insert_peer(&self.db, &record).context("failed to save contact")?;
        } else {
            queries::insert_peer(&self.db, &record).context("failed to update contact")?;
        }

        tracing::info!(peer = %node_id, name = %display_name, "added new contact");

        if let Err(e) = self.send_contact_handshake(&node_id, false).await {
            tracing::error!(peer = %node_id, "failed to send contact handshake: {:?}", e);
            self.mark_peer_offline(&node_id);
            self.emit(AppEvent::Error {
                message: format!("Could not send handshake to {node_id}: {e}"),
            });
        } else {
            self.emit(AppEvent::PeerHandshakeStage {
                peer: node_id.clone(),
                stage: "handshake sent".to_owned(),
            });
        }

        self.emit(AppEvent::PeerContactDetails {
            peer: node_id.clone(),
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

    async fn cmd_clear_messages(&mut self) -> Result<()> {
        queries::clear_all_messages(&self.db).context("failed to clear messages")?;
        self.emit(AppEvent::MessagesCleared);
        self.emit_chat_list();
        Ok(())
    }

    async fn cmd_clear_conversation_history(&mut self, target: String, is_group: bool) -> Result<()> {
        queries::clear_conversation_messages(&self.db, &target)
            .context("failed to clear conversation history")?;
        self.emit(AppEvent::ConversationCleared {
            target: target.clone(),
            is_group,
        });
        self.emit_chat_list();
        self.emit_conversation_messages(&target, is_group)?;
        Ok(())
    }

    async fn cmd_delete_conversation(&mut self, target: String, is_group: bool) -> Result<()> {
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

    async fn cmd_delete_identity(&mut self) -> Result<()> {
        queries::delete_local_identity(&self.db).context("failed to delete identity")?;
        // Shutdown for now; onboarding will trigger on next launch.
        std::process::exit(0);
    }

    async fn cmd_unlock_app(&mut self) -> Result<()> {
        self.emit(AppEvent::UnlockComplete);
        Ok(())
    }

    async fn cmd_set_app_foreground(&mut self, foreground: bool) -> Result<()> {
        self.app_foreground = foreground;
        tracing::info!(foreground, "android lifecycle state updated");

        if foreground {
            self.emit_network_status().await;
            self.emit_chat_list();
            if let Err(e) = self.flush_offline_queue().await {
                tracing::error!("queue flush after foreground resume failed: {:?}", e);
            }
            for peer in self.network.active_connections() {
                self.touch_peer_activity(&peer);
            }
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
                online: has_session,
                via_relay: false,
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
        if ticket_bytes.len() > CONTACT_HANDSHAKE_MAX_TICKET_BYTES {
            return Err(anyhow::anyhow!(
                "endpoint ticket too large for handshake frame: {} bytes",
                ticket_bytes.len()
            ));
        }

        let mut frame = Vec::with_capacity(4 + 1 + 32 + 2 + ticket_bytes.len());
        frame.extend_from_slice(CONTACT_HANDSHAKE_MAGIC);
        frame.push(if is_ack {
            CONTACT_HANDSHAKE_HELLO_ACK
        } else {
            CONTACT_HANDSHAKE_HELLO
        });
        frame.extend_from_slice(&crypto.x25519_public_bytes());
        frame.extend_from_slice(&(ticket_bytes.len() as u16).to_be_bytes());
        frame.extend_from_slice(ticket_bytes);
        Ok(frame)
    }

    /// Parse a direct-contact handshake frame.
    fn parse_contact_handshake(payload: &[u8]) -> Option<(bool, [u8; 32], Option<String>)> {
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
        if payload.len() != ticket_len_offset + 2 + ticket_len {
            return None;
        }

        let ticket = if ticket_len == 0 {
            None
        } else {
            let raw_ticket = &payload[ticket_len_offset + 2..];
            Some(String::from_utf8(raw_ticket.to_vec()).ok()?)
        };

        Some((is_ack, pubkey, ticket))
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
        let seconds = 20u64.saturating_mul(1u64 << step);
        let delay = Duration::from_secs(seconds.min(600));
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

            if !has_session && !has_connection {
                continue;
            }

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
                    .map(|peer| ContactDirectoryData {
                        id: peer.node_id.clone(),
                        is_online: self.network.has_connection(&peer.node_id),
                        name: peer.display_name.clone(),
                        initials: peer
                            .display_name
                            .split_whitespace()
                            .filter_map(|part| part.chars().next())
                            .take(2)
                            .map(|ch| ch.to_ascii_uppercase())
                            .collect::<String>(),
                        node_id: peer.node_id.clone(),
                        is_relay: *self.peer_via_relay.get(&peer.node_id).unwrap_or(&false),
                        is_verified: peer.verified,
                    })
                    .collect();
                self.emit(AppEvent::ContactsUpdated { contacts });
            }
            Err(e) => tracing::error!("failed to rebuild contacts list: {:?}", e),
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
                    Ok(ChatMessageData {
                        id: msg.id.to_string(),
                        text: msg.content,
                        timestamp: msg.timestamp.to_string(),
                        is_mine: msg.sender_id == local_node_id,
                        status: msg.status.as_str().to_owned(),
                        is_ephemeral: false,
                        ttl_seconds: 0,
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
