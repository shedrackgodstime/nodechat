//! Real backend — handles every `Command` using the local SQLite database.
//!
//! Runtime-only state (is_online, is_relay, member_count) is always returned
//! as false/0 until the iroh P2P engine is wired in.

use std::path::PathBuf;

use anyhow::Result;
use uuid::Uuid;

use crate::contract::{
    AppEvent, AppFlags, AppInfoView, AppSnapshot, ChatListItem, Command, ContactListItem,
    ConversationKind, ConversationView, GroupCandidateItem, HistoryScope, IdentityView,
    MessageItem, MessageKind, MessageStatus,
};
use crate::storage::queries::{
    self, derive_initials, GroupRecord, LocalIdentityRecord, MessageRecord, PeerRecord,
};

use crate::p2p::{NetworkEvent, NetworkManager};
use iroh_tickets::Ticket;

pub struct RealBackend {
    conn:                   rusqlite::Connection,
    db_path:                PathBuf,
    network:                NetworkManager,
    local_node_id:          String,   // cached for is_outgoing derivation
    local_display_name:     String,   // cached for sender_name in outgoing messages
    selected_candidates:    Vec<String>, // in-memory for group creation flow
    active_conversation_id: String,   // for ClearMessageHistory(ActiveConversation)
    event_tx:               std::sync::mpsc::Sender<AppEvent>,
}

impl RealBackend {
    /// Open (or create) the database and prepare the network manager.
    pub fn open(
        net_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
        event_tx: std::sync::mpsc::Sender<AppEvent>,
    ) -> Result<Self> {
        let path = db_path();
        let conn = crate::storage::initialize(&path)?;

        let (local_node_id, local_display_name, secret_key) =
            match queries::get_local_identity(&conn)? {
                Some(id) => (
                    id.node_id_hex, 
                    id.display_name, 
                    Some(iroh::SecretKey::from_bytes(&id.x25519_secret.try_into().unwrap_or([0u8; 32])))
                ),
                None => (String::new(), String::new(), None),
            };

        let network = NetworkManager::new(net_tx);
        let backend = Self {
            conn,
            db_path: db_path(),
            network: network.clone(),
            local_node_id,
            local_display_name,
            selected_candidates: Vec::new(),
            active_conversation_id: String::new(),
            event_tx,
        };

        if let Some(sk) = secret_key {
            let net = network.clone();
            let b_clone = backend.clone_for_sweep();
            tokio::spawn(async move {
                if let Err(e) = net.initialize(Some(sk)).await {
                    tracing::error!("Failed to initialize P2P network: {}", e);
                } else {
                    tracing::info!("P2P network initialized successfully — social watchdog active");
                    loop {
                        if let Err(e) = b_clone.sweep_peers() {
                            tracing::warn!("Social watchdog sweep failed: {}", e);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    }
                }
            });
        }

        tracing::info!(
            node_id = %backend.local_node_id,
            display_name = %backend.local_display_name,
            "RealBackend started"
        );

        Ok(backend)
    }

    /// Internal helper to allow spawning background tasks during 'open'
    fn clone_for_sweep(&self) -> Self {
        Self {
            conn:                   crate::storage::initialize(&self.db_path).expect("db clone"),
            db_path:                self.db_path.clone(),
            network:                self.network.clone(),
            local_node_id:          self.local_node_id.clone(),
            local_display_name:     self.local_display_name.clone(),
            selected_candidates:    Vec::new(),
            active_conversation_id: String::new(),
            event_tx:               self.event_tx.clone(),
        }
    }

    /// Build the full application snapshot for the initial UI load.
    pub fn snapshot(&self) -> AppSnapshot {
        self.build_snapshot().unwrap_or_else(|e| {
            eprintln!("[RealBackend] snapshot error: {e}");
            AppSnapshot {
                identity:             IdentityView::empty(),
                app_info:             AppInfoView::current(),
                app_flags:            AppFlags { direct_peer_count: 0, relay_peer_count: 0, is_offline: true },
                chat_list:            vec![],
                contact_list:         vec![],
                group_candidates:     vec![],
                active_conversation:  ConversationView::empty(ConversationKind::Direct),
                active_messages:      vec![],
                debug_feed:           vec!["[ERROR] failed to build snapshot".to_string()],
            }
        })
    }

    /// Process a command and return zero or more UI events.
    pub async fn handle_command(&mut self, command: Command) -> Vec<AppEvent> {
        match self.dispatch(command).await {
            Ok(events) => events,
            Err(e) => {
                eprintln!("[RealBackend] command error: {e}");
                vec![AppEvent::UserError(e.to_string())]
            }
        }
    }

    // ── Command Dispatch ─────────────────────────────────────────────────────

    async fn dispatch(&mut self, command: Command) -> Result<Vec<AppEvent>> {
        match command {
            // ── Refresh ──────────────────────────────────────────────────────
            Command::Refresh => {
                Ok(vec![AppEvent::SnapshotReady(self.build_snapshot()?)])
            }

            // ── Load Conversation ─────────────────────────────────────────
            Command::LoadConversation { conversation_id } => {
                self.active_conversation_id = conversation_id.clone();
                let view = self.build_conversation_view(&conversation_id)?;
                let messages = self.build_message_items(&conversation_id)?;
                Ok(vec![
                    AppEvent::ConversationUpdated(view.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id: view.conversation_id,
                        messages,
                    },
                ])
            }

            // ── Send Message ──────────────────────────────────────────────
            Command::SendMessage { conversation_id, plaintext } => {
                let is_group = queries::get_group(&self.conn, &conversation_id)?.is_some();
                let record = MessageRecord {
                    id:                Uuid::new_v4(),
                    kind:              "standard".to_string(),
                    target_id:         conversation_id.clone(),
                    sender_id:         self.local_node_id.clone(),
                    content:           plaintext.clone(),
                    timestamp:         unix_now(),
                    status:            MessageStatus::Queued,
                    invite_topic_id:   String::new(),
                    invite_group_name: String::new(),
                    invite_key:        String::new(),
                };
                queries::insert_message(&self.conn, &record)?;
                
                // --- 🚀 NEW: Non-blocking background send ---
                if !is_group {
                    if let Ok(Some(peer)) = queries::get_peer(&self.conn, &conversation_id) {
                        if !peer.verified {
                            self.spawn_handshake(conversation_id.clone(), peer.endpoint_ticket, 1);
                        }
                    }
                }
                self.spawn_message_pump(conversation_id.clone());

                // Push updated view to UI
                let message = self.to_message_item(&record)?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::MessageAppended { conversation_id, message },
                    AppEvent::ChatListUpdated(chat_list),
                ])
            }

            Command::RetryQueuedMessage { conversation_id, .. } => {
                self.spawn_message_pump(conversation_id.clone());
                let messages = self.build_message_items(&conversation_id)?;
                Ok(vec![AppEvent::MessageListReplaced { conversation_id, messages }])
            }

            // ── Delete Conversation ───────────────────────────────────────
            Command::DeleteConversation { conversation_id, .. } => {
                let is_group = queries::get_group(&self.conn, &conversation_id)?.is_some();
                queries::delete_conversation(&self.conn, &conversation_id, is_group)?;
                if self.active_conversation_id == conversation_id {
                    self.active_conversation_id.clear();
                }
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::ConversationUpdated(ConversationView::empty(ConversationKind::Direct)),
                    AppEvent::StatusNotice("conversation deleted".to_string()),
                ])
            }

            // ── Add Contact ───────────────────────────────────────────────
            Command::AddContact { ticket_or_peer_id } => {
                tracing::info!("AddContact: parsing ticket/peer_id (len={})", ticket_or_peer_id.len());
                let node_id = match iroh_tickets::endpoint::EndpointTicket::deserialize(&ticket_or_peer_id) {
                    Ok(ticket) => {
                        let id = ticket.endpoint_addr().id.to_string();
                        tracing::info!(node_id = %id, "AddContact: parsed as full EndpointTicket");
                        id
                    }
                    Err(e) => {
                        tracing::warn!("AddContact: not a ticket ({}), treating as raw node_id", e);
                        ticket_or_peer_id.clone()
                    }
                };
                let peer = PeerRecord {
                    node_id:         node_id.clone(),
                    display_name:    short_name(&node_id),
                    endpoint_ticket: ticket_or_peer_id.clone(),
                    x25519_pubkey:   String::new(),
                    verified:        false,
                };
                queries::upsert_peer(&self.conn, &peer)?;
                tracing::info!(node_id = %node_id, name = %peer.display_name, "AddContact: peer saved to SQLite");
                
                // --- 🚀 NEW: Trigger Background Handshake ---
                let network = self.network.clone();
                let db_path = self.db_path.clone();
                let target_id = node_id.clone();
                let ticket = ticket_or_peer_id.clone();
                let my_name = self.local_display_name.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = perform_handshake(network, db_path, target_id, ticket, my_name).await {
                        tracing::warn!("handshake failed in background: {:?}", e);
                    }
                });
                
                let contact_list = self.build_contact_list()?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ContactListUpdated(contact_list),
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::StatusNotice(format!("contact added: {}", peer.display_name)),
                ])
            }

            // ── Create Group ──────────────────────────────────────────────
            Command::CreateGroup { name, .. } => {
                let topic_id = format!("topic-{}", Uuid::new_v4());
                let symmetric_key = vec![0u8; 32]; // placeholder until crypto engine is fully re-integrated

                queries::insert_group(&self.conn, &GroupRecord {
                    topic_id:      topic_id.clone(),
                    group_name:    name.clone(),
                    symmetric_key: symmetric_key.clone(),
                })?;

                // Subscribe locally to the gossip topic swarm
                let active_nodes = self.network.active_connections();
                if let Err(e) = self.network.subscribe_group(&topic_id, active_nodes).await {
                    tracing::warn!("failed to subscribe to group swarm {}: {:?}", topic_id, e);
                }

                // Send invitations directly to selected peers asynchronously.
                for peer_id in std::mem::take(&mut self.selected_candidates) {
                    let invite = serde_json::json!({
                        "type": "group_invite",
                        "topic": topic_id,
                        "key": hex::encode(&symmetric_key),
                        "group_name": name,
                    }).to_string();

                    let invite_record = MessageRecord {
                        id:                Uuid::new_v4(),
                        kind:              "group_invite".to_owned(),
                        target_id:         peer_id.clone(),
                        sender_id:         self.local_node_id.clone(),
                        content:           invite,
                        timestamp:         unix_now(),
                        status:            MessageStatus::Queued,
                        // Embedded metadata used by UI rendering
                        invite_topic_id:   topic_id.clone(),
                        invite_group_name: name.clone(),
                        invite_key:        hex::encode(&symmetric_key),
                    };
                    let _ = queries::insert_message(&self.conn, &invite_record);

                    // Actually attempt network send (bypassing crypto layer momentarily)
                    if let Err(e) = self.network.send_direct(&peer_id, None, invite_record.content.into_bytes()).await {
                        tracing::warn!("failed to route invite directly to {}: {:?}", peer_id, e);
                    } else {
                        let _ = queries::advance_status(&self.conn, &invite_record.id, MessageStatus::Sent);
                    }
                }

                let chat_list = self.build_chat_list()?;
                let candidates = self.build_group_candidates()?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::GroupCandidatesUpdated(candidates),
                    AppEvent::StatusNotice(format!("group created: {}", name)),
                ])
            }

            // ── Toggle Group Candidate ─────────────────────────────────────
            Command::ToggleGroupCandidate { contact_id } => {
                if let Some(pos) = self.selected_candidates.iter().position(|id| *id == contact_id) {
                    self.selected_candidates.remove(pos);
                } else {
                    self.selected_candidates.push(contact_id);
                }
                let candidates = self.build_group_candidates()?;
                Ok(vec![AppEvent::GroupCandidatesUpdated(candidates)])
            }

            // ── Open Direct Conversation ───────────────────────────────────
            Command::OpenDirectConversation { contact_id } => {
                // contact_id == peer node_id in our simplified schema
                self.active_conversation_id = contact_id.clone();
                let view = self.build_conversation_view(&contact_id)?;
                let messages = self.build_message_items(&contact_id)?;
                Ok(vec![
                    AppEvent::ConversationUpdated(view.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id: view.conversation_id,
                        messages,
                    },
                ])
            }

            // ── Create Identity ───────────────────────────────────────────
            Command::CreateIdentity { display_name, pin } => {
                let pin_hash = hash_pin(&pin);
                let mut secret_bytes = [0u8; 32];
                {
                    let mut rng = rand::thread_rng();
                    rand::RngCore::fill_bytes(&mut rng, &mut secret_bytes);
                }
                let secret_key = iroh::SecretKey::from_bytes(&secret_bytes);
                let node_id_bytes = secret_key.public().as_bytes().to_vec();
                let node_id_hex = hex::encode(&node_id_bytes);

                // Now we securely initialize the P2P network so the node grabs its live endpoint
                // ticket cleanly. (The previous infinite loading was strictly caused by the Android 
                // native CWD SQLite crash, which has now been fully fixed).
                if let Err(e) = self.network.initialize(Some(secret_key.clone())).await {
                    tracing::error!("P2P Init Error: {}", e);
                }
                let ticket = self.network.endpoint_ticket().unwrap_or_default();

                queries::insert_local_identity(&self.conn, &LocalIdentityRecord {
                    display_name:    display_name.clone(),
                    node_id_hex:     node_id_hex.clone(),
                    x25519_secret:   secret_key.to_bytes().to_vec(),
                    endpoint_ticket: ticket.clone(),
                    pin_hash,
                })?;

                self.local_node_id = node_id_hex;
                self.local_display_name = display_name;

                let identity = self.build_identity_view()?;
                Ok(vec![
                    AppEvent::IdentityUpdated(identity),
                    AppEvent::StatusNotice("identity ready".to_string())
                ])
            }

            // ── Finalize Identity ─────────────────────────────────────────
            Command::FinalizeIdentity => {
                Ok(vec![])
            }

            // ── Unlock App ────────────────────────────────────────────────
            Command::UnlockApp { pin } => {
                match queries::get_local_identity(&self.conn)? {
                    None => Ok(vec![AppEvent::UserError("no identity found".to_string())]),
                    Some(id) => {
                        let ok = id.pin_hash.is_empty() || id.pin_hash == hash_pin(&pin);
                        if ok {
                            self.local_node_id = id.node_id_hex;
                            self.local_display_name = id.display_name;
                            
                            // Load secret key & start network
                            let secret_bytes: [u8; 32] = id.x25519_secret.clone().try_into().unwrap_or([0u8; 32]);
                            let sk = iroh::SecretKey::from_bytes(&secret_bytes);
                            if let Err(e) = self.network.initialize(Some(sk)).await {
                                eprintln!("Failed to init network: {}", e);
                            }

                            let identity = self.build_identity_view()?;
                            Ok(vec![AppEvent::IdentityUpdated(identity)])
                        } else {
                            Ok(vec![AppEvent::UserError("incorrect PIN".to_string())])
                        }
                    }
                }
            }

            // ── Change Password ───────────────────────────────────────────
            Command::ChangePassword { current_pin, new_pin } => {
                match queries::get_local_identity(&self.conn)? {
                    None => Ok(vec![AppEvent::UserError("no identity found".to_string())]),
                    Some(id) => {
                        let ok = id.pin_hash.is_empty() || id.pin_hash == hash_pin(&current_pin);
                        if ok {
                            queries::update_pin_hash(&self.conn, &hash_pin(&new_pin))?;
                            Ok(vec![AppEvent::StatusNotice("password updated".to_string())])
                        } else {
                            Ok(vec![AppEvent::UserError("incorrect current PIN".to_string())])
                        }
                    }
                }
            }

            // ── Share Contact ─────────────────────────────────────────────
            Command::ShareContact { .. } => {
                // P2P feature — implemented when iroh is wired
                Ok(vec![AppEvent::StatusNotice("share: pending P2P engine".to_string())])
            }

            // ── Update Display Name ───────────────────────────────────────
            Command::UpdateDisplayName { display_name } => {
                if display_name.trim().is_empty() {
                    return Ok(vec![AppEvent::UserError("display name cannot be empty".to_string())]);
                }
                queries::update_display_name(&self.conn, &display_name)?;
                self.local_display_name = display_name;
                let identity = self.build_identity_view()?;
                Ok(vec![AppEvent::IdentityUpdated(identity)])
            }

            // ── Reset Identity ────────────────────────────────────────────
            Command::ResetIdentity { .. } => {
                queries::delete_all(&self.conn)?;
                self.local_node_id.clear();
                self.local_display_name.clear();
                self.selected_candidates.clear();
                self.active_conversation_id.clear();
                Ok(vec![
                    AppEvent::IdentityUpdated(IdentityView::empty()),
                    AppEvent::ChatListUpdated(vec![]),
                    AppEvent::ContactListUpdated(vec![]),
                    AppEvent::MessageListReplaced { conversation_id: String::new(), messages: vec![] },
                    AppEvent::StatusNotice("identity reset".to_string()),
                ])
            }

            // ── Set Verification ──────────────────────────────────────────
            Command::SetVerification { peer_id, verified } => {
                queries::set_peer_verified(&self.conn, &peer_id, verified)?;
                let contact_list = self.build_contact_list()?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ContactListUpdated(contact_list),
                    AppEvent::ChatListUpdated(chat_list),
                ])
            }

            // ── Accept Group Invite ───────────────────────────────────────
            Command::AcceptGroupInvite { topic_id, .. } => {
                if !queries::group_exists(&self.conn, &topic_id)? {
                    let label_len = topic_id.len().min(8);
                    queries::insert_group(&self.conn, &GroupRecord {
                        topic_id:      topic_id.clone(),
                        group_name:    format!("Group {}", &topic_id[..label_len]),
                        symmetric_key: vec![0u8; 32],
                    })?;
                }
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::StatusNotice(format!("joined group {}", topic_id)),
                ])
            }

            // ── Clear Message History ─────────────────────────────────────
            Command::ClearMessageHistory { scope, .. } => {
                match scope {
                    HistoryScope::AllConversations => {
                        queries::clear_messages(&self.conn)?;
                    }
                    HistoryScope::ActiveConversation => {
                        if !self.active_conversation_id.is_empty() {
                            queries::clear_conversation(&self.conn, &self.active_conversation_id)?;
                        }
                    }
                }
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::MessageListReplaced {
                        conversation_id: self.active_conversation_id.clone(),
                        messages: vec![],
                    },
                    AppEvent::StatusNotice("history cleared".to_string()),
                ])
            }
        }
    }

    /// Process incoming background P2P events.
    pub async fn handle_network_event(&mut self, event: NetworkEvent) -> Vec<AppEvent> {
        let mut events = vec![];
        match event {
            NetworkEvent::PeerConnected { node_id, .. } => {
                tracing::info!(peer=%node_id, "peer connected at transport level");
                self.spawn_message_pump(node_id.clone());
                
                // Refresh Lists
                events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));
                
                // Refresh Header if active
                if self.active_conversation_id == node_id {
                    if let Ok(conv) = self.build_conversation_view(&node_id) {
                        events.push(AppEvent::ConversationUpdated(conv));
                    }
                }
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                tracing::info!(peer=%node_id, "peer disconnected");
                
                // Refresh Lists
                events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));
                
                // Refresh Header if active
                if self.active_conversation_id == node_id {
                    if let Ok(conv) = self.build_conversation_view(&node_id) {
                        events.push(AppEvent::ConversationUpdated(conv));
                    }
                }
            }
            NetworkEvent::DirectMessage { from, payload } => {
                use crate::p2p::protocol::{DirectFrame, DIRECT_MAGIC, MAGIC, HandshakeFrame, HELLO, HELLO_ACK};
                
                if payload.starts_with(&MAGIC) {
                    if let Ok(frame) = HandshakeFrame::decode(&payload) {
                        tracing::info!(peer=%from, kind=%frame.kind, name=%frame.display_name, "NC1H handshake received");
                        
                        // Update Peer Record
                        let _ = queries::upsert_peer(&self.conn, &PeerRecord {
                            node_id:         from.clone(),
                            display_name:    frame.display_name.clone(),
                            endpoint_ticket: frame.ticket.clone(),
                            x25519_pubkey:   hex::encode(frame.x25519_public),
                            verified:        true,
                        });

                        if frame.kind == HELLO {
                            // Send ACK back
                            let network = self.network.clone();
                            let target = from.clone();
                            let ticket = frame.ticket.clone();
                            let my_name = self.local_display_name.clone();
                            let db_path = self.db_path.clone();

                            tokio::spawn(async move {
                                let _ = perform_handshake_ack(network, db_path, target, ticket, my_name).await;
                            });

                            // --- 🚀 AUTO-FLUSH: Trigger pump now that we have a connection
                            self.spawn_message_pump(from.clone());
                        } else if frame.kind == HELLO_ACK {
                            // Handshake finished! Flush any queued messages for this peer.
                            tracing::info!(peer=%from, "Handshake finished (HELLO_ACK) — starting message pump");
                            self.spawn_message_pump(from.clone());
                        }

                        // Refresh Lists
                        events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                        events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));

                        // Refresh Header if active
                        if self.active_conversation_id == from {
                            if let Ok(conv) = self.build_conversation_view(&from) {
                                events.push(AppEvent::ConversationUpdated(conv));
                            }
                        }

                        return events;
                    }
                } else if payload.starts_with(&DIRECT_MAGIC) {
                    if let Ok(frame) = DirectFrame::decode(&payload) {
                        match frame {
                            DirectFrame::Text { id, content } => {
                                let text = String::from_utf8_lossy(&content).to_string();
                                let msg_id = id.to_string();
                                tracing::info!(peer=%from, id=%msg_id, "NC2D framed message received");

                                let record = MessageRecord {
                                    id,
                                    kind: "standard".to_string(),
                                    target_id: from.clone(),
                                    sender_id: from.clone(),
                                    content: text,
                                    timestamp: unix_now(),
                                    status: MessageStatus::Delivered,
                                    invite_topic_id: String::new(),
                                    invite_group_name: String::new(),
                                    invite_key: String::new(),
                                };
                                let _ = queries::insert_message(&self.conn, &record);

                                // Send Receipt back to peer
                                let _ = self.send_receipt(from.clone(), id, false);
                                
                                if let Ok(msg) = self.to_message_item(&record) {
                                    events.push(AppEvent::MessageAppended {
                                        conversation_id: from.clone(),
                                        message: msg,
                                    });
                                    if let Ok(chat_list) = self.build_chat_list() {
                                        events.push(AppEvent::ChatListUpdated(chat_list));
                                    }
                                }
                            }
                            DirectFrame::Receipt { id, is_read: _ } => {
                                tracing::info!(peer=%from, id=%id.to_string(), "NC2D receipt received");
                                let _ = queries::advance_status(&self.conn, &id, MessageStatus::Delivered);
                                events.push(AppEvent::MessageStatusChanged {
                                    conversation_id: from.clone(),
                                    message_id: id.to_string(),
                                    status: MessageStatus::Delivered,
                                });
                                // Also update Chat List to refresh preview status
                                if let Ok(chat_list) = self.build_chat_list() {
                                    events.push(AppEvent::ChatListUpdated(chat_list));
                                }
                            }
                        }
                        return events;
                    }
                }

                if let Ok(text) = String::from_utf8(payload) {
                    let record = MessageRecord {
                        id: Uuid::new_v4(),
                        kind: "standard".to_string(),
                        target_id: from.clone(),
                        sender_id: from.clone(),
                        content: text,
                        timestamp: unix_now(),
                        status: MessageStatus::Delivered,
                        invite_topic_id: String::new(),
                        invite_group_name: String::new(),
                        invite_key: String::new(),
                    };
                    let _ = queries::insert_message(&self.conn, &record);
                    
                    if let Ok(msg) = self.to_message_item(&record) {
                        events.push(AppEvent::MessageAppended {
                            conversation_id: from.clone(),
                            message: msg,
                        });
                        if let Ok(chat_list) = self.build_chat_list() {
                            events.push(AppEvent::ChatListUpdated(chat_list));
                        }
                    }
                }
            }
            NetworkEvent::GroupMessage { .. } => {
                // Group receive to follow
            }
        }
        events
    }

    fn send_receipt(&self, target: String, message_id: uuid::Uuid, is_read: bool) -> Result<()> {
        let frame = crate::p2p::protocol::DirectFrame::Receipt { id: message_id, is_read }.encode();
        let net = self.network.clone();
        tokio::spawn(async move {
            let _ = net.send_direct(&target, None, frame).await;
        });
        Ok(())
    }
    // ── Snapshot Builders ─────────────────────────────────────────────────────

    fn build_snapshot(&self) -> Result<AppSnapshot> {
        Ok(AppSnapshot {
            identity:            self.build_identity_view()?,
            app_info:            AppInfoView::current(),
            app_flags:           AppFlags { direct_peer_count: 0, relay_peer_count: 0, is_offline: true },
            chat_list:           self.build_chat_list()?,
            contact_list:        self.build_contact_list()?,
            group_candidates:    self.build_group_candidates()?,
            active_conversation: ConversationView::empty(ConversationKind::Direct),
            active_messages:     vec![],
            debug_feed:          vec![],
        })
    }

    fn build_identity_view(&self) -> Result<IdentityView> {
        match queries::get_local_identity(&self.conn)? {
            None => Ok(IdentityView::empty()),
            Some(id) => Ok(IdentityView {
                display_name:    id.display_name.clone(),
                initials:        derive_initials(&id.display_name),
                peer_id:         id.node_id_hex,
                endpoint_ticket: id.endpoint_ticket,
                is_locked:       false,
                has_identity:    true,
            }),
        }
    }

    fn build_chat_list(&self) -> Result<Vec<ChatListItem>> {
        Self::build_chat_list_static(&self.conn, &self.network, &self.local_node_id)
    }

    pub fn build_chat_list_static(
        conn: &rusqlite::Connection,
        network: &NetworkManager,
        local_node_id: &str
    ) -> Result<Vec<ChatListItem>> {
        let previews = queries::list_chat_previews(conn, local_node_id)?;
        Ok(previews.into_iter().map(|p| {
            let is_online = if p.is_group { false } else { network.has_connection(&p.id) };
            ChatListItem {
                conversation_id:          p.id,
                kind:                     if p.is_group { ConversationKind::Group } else { ConversationKind::Direct },
                title:                    p.title,
                initials:                 p.initials,
                last_message:             if !p.is_session_ready && p.last_message.is_empty() { "Waiting for handshake...".to_string() } else { p.last_message },
                last_message_status:      p.last_message_status,
                is_last_message_outgoing: p.is_outgoing,
                timestamp:                format_timestamp(p.timestamp),
                member_count:             0,     
                unread_count:             0,     
                is_online,
                is_relay:                 false, 
                is_verified:              p.is_verified,
                is_session_ready:         p.is_session_ready,
                has_queued_messages:      p.has_queued,
            }
        }).collect())
    }

    fn build_contact_list(&self) -> Result<Vec<ContactListItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| {
            let is_online = self.network.has_connection(&p.node_id);
            ContactListItem {
                contact_id:              p.node_id.clone(),
                peer_id:                 p.node_id.clone(),
                display_name:            p.display_name.clone(),
                initials:                derive_initials(&p.display_name),
                is_online,
                is_relay:                false, 
                is_verified:             p.verified,
                is_session_ready:        !p.x25519_pubkey.is_empty(),
                direct_conversation_id:  p.node_id, 
            }
        }).collect())
    }

    fn build_group_candidates(&self) -> Result<Vec<GroupCandidateItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| {
            let is_online = self.network.has_connection(&p.node_id);
            GroupCandidateItem {
                contact_id:   p.node_id.clone(),
                display_name: p.display_name.clone(),
                initials:     derive_initials(&p.display_name),
                is_selected:  self.selected_candidates.contains(&p.node_id),
                is_online,
            }
        }).collect())
    }

    fn build_conversation_view(&self, id: &str) -> Result<ConversationView> {
        // Group check first
        if let Some(g) = queries::get_group(&self.conn, id)? {
            return Ok(ConversationView {
                conversation_id:  g.topic_id.clone(),
                kind:             ConversationKind::Group,
                title:            g.group_name.clone(),
                initials:         derive_initials(&g.group_name),
                peer_id:          String::new(),
                ticket:           String::new(),
                is_online:        false,
                is_relay:         false,
                is_verified:      true,
                is_session_ready: false,
                connection_stage: String::new(),
                member_count:     0,
                return_screen:    0,
            });
        }
        // Direct peer
        if let Some(p) = queries::get_peer(&self.conn, id)? {
            let is_online = self.network.has_connection(&p.node_id);
            let is_session_ready = !p.x25519_pubkey.is_empty();
            let connection_stage = if is_online {
                if p.verified { "Secure P2P session active" } else { "Handshaking..." }
            } else {
                "Peer offline"
            }.to_string();

            return Ok(ConversationView {
                conversation_id:  p.node_id.clone(),
                kind:             ConversationKind::Direct,
                title:            p.display_name.clone(),
                initials:         derive_initials(&p.display_name),
                peer_id:          p.node_id.clone(),
                ticket:           p.endpoint_ticket,
                is_online,
                is_relay:         false,
                is_verified:      p.verified,
                is_session_ready,
                connection_stage,
                member_count:     1,
                return_screen:    0,
            });
        }
        Ok(ConversationView::empty(ConversationKind::Direct))
    }

    fn build_message_items(&self, target_id: &str) -> Result<Vec<MessageItem>> {
        queries::list_messages(&self.conn, target_id)?
            .into_iter()
            .map(|m| self.to_message_item(&m))
            .collect()
    }

    fn to_message_item(&self, r: &MessageRecord) -> Result<MessageItem> {
        let is_outgoing = r.sender_id == self.local_node_id;

        let sender_name = if is_outgoing {
            self.local_display_name.clone()
        } else {
            match queries::get_peer(&self.conn, &r.sender_id)? {
                Some(p) => p.display_name,
                None    => r.sender_id.chars().take(8).collect(),
            }
        };

        let invite_is_joined = !r.invite_topic_id.is_empty()
            && queries::group_exists(&self.conn, &r.invite_topic_id)?;

        let kind = match r.kind.as_str() {
            "group_invite" => MessageKind::GroupInvite,
            "system"       => MessageKind::System,
            _              => MessageKind::Standard,
        };

        Ok(MessageItem {
            message_id:       r.id.to_string(),
            conversation_id:  r.target_id.clone(),
            sender_name,
            text:             r.content.clone(),
            timestamp:        format_timestamp(r.timestamp),
            is_outgoing,
            is_system:        r.kind == "system",
            status:           r.status,
            kind,
            invite_group_name: r.invite_group_name.clone(),
            invite_topic_id:   r.invite_topic_id.clone(),
            invite_key:        r.invite_key.clone(),
            invite_is_joined,
            is_ephemeral:  false,
            ttl_seconds:   0,
        })
    }

    /// Background task that attempts to send all queued messages for a specific conversation.
    pub fn sweep_peers(&self) -> Result<()> {
        let peers = queries::list_peers(&self.conn)?;
        tracing::info!(count = %peers.len(), "Sweeping peers for proactive connectivity...");
        for peer in peers {
            // Trigger proactive handshake/dial.
            // Even if offline, Iroh will attempt to find them in the background.
            self.spawn_handshake(peer.node_id.clone(), peer.endpoint_ticket, 1);
            
            // Also wake the pump in case there are messages waiting from a previous session
            self.spawn_message_pump(peer.node_id);
        }
        Ok(())
    }

    pub fn spawn_message_pump(&self, conversation_id: String) {
        let db = self.db_path.clone();
        let net = self.network.clone();
        let tx = self.event_tx.clone();
        let cid = conversation_id.to_string();
        let nid = self.local_node_id.clone();
        let my_name = self.local_display_name.clone(); // Added
        
        tokio::spawn(async move {
            if let Err(e) = Self::flush_queued_messages_internal(&db, &net, &tx, &cid, nid, my_name).await {
                tracing::error!(peer = %cid, "Flush loop error: {:?}", e);
            }
        });
    }

    pub fn spawn_handshake(&self, target_id: String, ticket: String, stage: u8) {
        let network = self.network.clone();
        let db_path = self.db_path.clone();
        let my_name = self.local_display_name.clone();
        
        tokio::spawn(async move {
            if stage == 1 {
                let _ = perform_handshake(network, db_path, target_id, ticket, my_name).await;
            } else {
                let _ = perform_handshake_ack(network, db_path, target_id, ticket, my_name).await;
            }
        });
    }

    async fn flush_queued_messages_internal(
        db_path: &std::path::Path,
        network: &NetworkManager,
        event_tx: &std::sync::mpsc::Sender<AppEvent>,
        conversation_id: &str,
        local_node_id: String,
        my_name: String, // Added
    ) -> Result<()> {
        let conn = crate::storage::initialize(db_path)?;
        let is_group = queries::get_group(&conn, conversation_id)?.is_some();
        let queued = queries::get_queued_messages(&conn, conversation_id)?;
        if queued.is_empty() { return Ok(()); }

        // Fetch ticket as dial hint if it's a direct conversation
        let ticket_hint = if !is_group {
            queries::get_peer(&conn, conversation_id)?.map(|p| p.endpoint_ticket)
        } else {
            None
        };

        tracing::info!(
            peer = %conversation_id, 
            count = %queued.len(), 
            is_group = %is_group, 
            has_hint = %ticket_hint.is_some(),
            "Flushing queued messages..."
        );

        for msg in queued {
            // 1. Connectivity Check: is the peer actually online right now?
            let is_online = network.has_connection(conversation_id);
            
            // 2. Handshake Check: have we finished the handshake session?
            let peer = queries::get_peer(&conn, conversation_id)?;
            let is_verified = peer.map(|p| p.verified).unwrap_or(false);

            if !is_group && (!is_online || !is_verified) {
                 // If we aren't online/verified, we trigger the handshake and STOP.
                 // We do NOT fall through to send_direct yet. 
                 // We wait for the 'HELLO_ACK' or 'PeerConnected' to wake us up again.
                 if ticket_hint.is_some() {
                     tracing::info!(peer = %conversation_id, "Flush: peer not ready. Triggering proactive dial and waiting...");
                     
                     // If not verified, also trigger handshake in background
                     // (Handshake logic handles its own 'spawn_message_pump' on completion)
                     if !is_verified {
                         let net = network.clone();
                         let db = db_path.to_path_buf();
                         let target = conversation_id.to_string();
                         let ticket = ticket_hint.clone().unwrap_or_default();
                         let name = my_name.clone();
                         tokio::spawn(async move {
                             let _ = perform_handshake(net, db, target, ticket, name).await;
                         });
                     } else {
                        // Already verified but vanished? Dial again to wake Iroh up
                        let net = network.clone();
                        let target = conversation_id.to_string();
                        let hint = ticket_hint.clone();
                        tokio::spawn(async move {
                            let _ = net.send_direct(&target, hint.as_deref(), vec![]).await; // Empty ping to wake
                        });
                     }
                 } else {
                     tracing::info!(peer = %conversation_id, online = %is_online, verified = %is_verified, "Flush paused: peer is offline and no dial hint available.");
                 }
                 break; // CRITICAL: Stop the loop here. Wait for connection events to re-trigger the pump.
            }

            // Attempt network send
            let frame = crate::p2p::protocol::DirectFrame::Text {
                id: msg.id,
                content: msg.content.clone().into_bytes(),
            }.encode();
            
            let result = if is_group {
                network.broadcast_group(conversation_id, msg.content.clone().into_bytes()).await
            } else {
                network.send_direct(conversation_id, ticket_hint.as_deref(), frame).await
            };

            match result {
                Ok(_) => {
                    // Update DB - only advance if network call truly succeeded
                    let _ = queries::advance_status(&conn, &msg.id, MessageStatus::Sent);
                    // Update UI
                    let _ = event_tx.send(AppEvent::MessageStatusChanged {
                        conversation_id: conversation_id.to_string(),
                        message_id: msg.id.to_string(),
                        status: MessageStatus::Sent,
                    });
                    
                    // Refresh Sidebar — ensures "Ghost Clock" vanishes immediately
                    if let Ok(chat_list) = Self::build_chat_list_static(&conn, network, &local_node_id) {
                        let _ = event_tx.send(AppEvent::ChatListUpdated(chat_list));
                    }
                }
                Err(e) => {
                    tracing::debug!(peer = %conversation_id, "Flush failed for msg {}: {:?}", msg.id, e);
                    // Stop flushing for this peer if connection failed
                    break;
                }
            }
        }
        Ok(())
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Current Unix time in seconds.
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Format a Unix timestamp (seconds) as "HH:MM".
/// Returns an empty string for timestamp 0.
fn format_timestamp(secs: i64) -> String {
    if secs == 0 {
        return String::new();
    }
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    format!("{:02}:{:02}", h, m)
}

/// Derive a short human-readable name from a raw ticket/peer_id string.
fn short_name(id: &str) -> String {
    let t = id.trim();
    if t.len() <= 12 {
        t.to_string()
    } else {
        format!("Peer_{}", &t[t.len() - 6..])
    }
}

/// Simple PIN hashing — replace with argon2 when P2P security layer lands.
fn hash_pin(pin: &str) -> String {
    if pin.is_empty() {
        String::new()
    } else {
        // Prefix differentiates an empty hash from "no PIN set"
        format!("sha256_placeholder:{}", pin)
    }
}

// ── Database Path Resolution ──────────────────────────────────────────────────

/// Resolve the path to the local database file.
#[cfg(not(target_os = "android"))]
pub fn db_path() -> PathBuf {
    use directories::ProjectDirs;
    if let Some(proj) = ProjectDirs::from("com", "nodechat", "NodeChat") {
        let dir = proj.data_dir().to_path_buf();
        std::fs::create_dir_all(&dir).ok();
        dir.join("nodechat.db")
    } else {
        PathBuf::from("nodechat.db")
    }
}

/// On Android the data directory is managed by the OS.
/// We fall back to a relative path; the Android manifest grants us write access.
#[cfg(target_os = "android")]
pub fn db_path() -> PathBuf {
    if let Some(data_dir) = crate::ANDROID_DATA_DIR.get() {
        data_dir.join("nodechat.db")
    } else {
        PathBuf::from("nodechat.db")
    }
}

// ── Background Handshake ──────────────────────────────────────────────────────

async fn perform_handshake(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
) -> Result<()> {
    perform_handshake_internal(network, db_path, target_id, ticket, my_name, crate::p2p::protocol::HELLO).await
}

async fn perform_handshake_ack(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
) -> Result<()> {
    perform_handshake_internal(network, db_path, target_id, ticket, my_name, crate::p2p::protocol::HELLO_ACK).await
}

async fn perform_handshake_internal(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
    kind: u8,
) -> Result<()> {
    tracing::info!(peer = %target_id, kind = %kind, "background handshake: starting stage...");
    
    // 1. Fetch our own public key
    let conn = rusqlite::Connection::open(&db_path)?;
    let identity = queries::get_local_identity(&conn)?
        .ok_or_else(|| anyhow::anyhow!("no local identity"))?;
    let my_ticket = identity.endpoint_ticket;
    
    let secret_bytes: [u8; 32] = identity.x25519_secret.clone().try_into().unwrap_or([0u8; 32]);
    let secret = iroh::SecretKey::from_bytes(&secret_bytes);
    let my_pubkey: [u8; 32] = *secret.public().as_bytes();

    // 2. Build the frame
    use crate::p2p::protocol::HandshakeFrame;
    let frame = HandshakeFrame {
        kind,
        x25519_public: my_pubkey,
        ticket: my_ticket,
        display_name: my_name,
    };

    // 3. Dial and Send
    tracing::info!(peer = %target_id, "background handshake: dialing...");
    network.send_direct(&target_id, Some(&ticket), frame.encode()).await?;
    
    tracing::info!(peer = %target_id, kind = %kind, "background handshake: stage sent successfully");
    
    Ok(())
}
