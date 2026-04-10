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
    network:                NetworkManager,
    local_node_id:          String,   // cached for is_outgoing derivation
    local_display_name:     String,   // cached for sender_name in outgoing messages
    selected_candidates:    Vec<String>, // in-memory for group creation flow
    active_conversation_id: String,   // for ClearMessageHistory(ActiveConversation)
}

impl RealBackend {
    /// Open (or create) the database and prepare the network manager.
    pub fn open(net_tx: tokio::sync::mpsc::Sender<NetworkEvent>) -> Result<Self> {
        let path = db_path();
        let conn = crate::storage::initialize(&path)?;

        let (local_node_id, local_display_name) =
            match queries::get_local_identity(&conn)? {
                Some(id) => (id.node_id_hex, id.display_name),
                None => (String::new(), String::new()),
            };

        Ok(Self {
            conn,
            network: NetworkManager::new(net_tx),
            local_node_id,
            local_display_name,
            selected_candidates: Vec::new(),
            active_conversation_id: String::new(),
        })
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

                if is_group {
                    if let Err(e) = self.network.broadcast_group(&conversation_id, plaintext.into_bytes()).await {
                        tracing::warn!("Failed to broadcast group message: {:?}", e);
                    } else {
                        let _ = queries::advance_status(&self.conn, &record.id, MessageStatus::Sent);
                    }
                } else {
                    if let Err(e) = self.network.send_direct(&conversation_id, None, plaintext.into_bytes()).await {
                        tracing::warn!("Failed to send direct message: {:?}", e);
                    } else {
                        let _ = queries::advance_status(&self.conn, &record.id, MessageStatus::Sent);
                    }
                }

                // Push updated view to UI
                let message = self.to_message_item(&record)?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::MessageAppended { conversation_id, message },
                    AppEvent::ChatListUpdated(chat_list),
                ])
            }

            // ── Retry Queued ──────────────────────────────────────────────
            Command::RetryQueuedMessage { conversation_id, message_id } => {
                if let Ok(id) = Uuid::parse_str(&message_id) {
                    // Best-effort — ignore transition errors (already advanced)
                    let _ = queries::advance_status(&self.conn, &id, MessageStatus::Sent);
                }
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
                queries::insert_peer(&self.conn, &peer)?;
                tracing::info!(node_id = %node_id, name = %peer.display_name, "AddContact: peer saved to SQLite");
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
            NetworkEvent::PeerConnected { node_id, via_relay } => {
                tracing::info!(peer=%node_id, relay=%via_relay, "peer connected");
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                tracing::info!(peer=%node_id, "peer disconnected");
            }
            NetworkEvent::DirectMessage { from, payload } => {
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
        let previews = queries::list_chat_previews(&self.conn, &self.local_node_id)?;
        Ok(previews.into_iter().map(|p| ChatListItem {
            conversation_id:          p.id,
            kind:                     if p.is_group { ConversationKind::Group } else { ConversationKind::Direct },
            title:                    p.title,
            initials:                 p.initials,
            last_message:             p.last_message,
            last_message_status:      p.last_message_status,
            is_last_message_outgoing: p.is_outgoing,
            timestamp:                format_timestamp(p.timestamp),
            member_count:             0,     // runtime P2P
            unread_count:             0,     // simplified MVP
            is_online:                false, // runtime P2P
            is_relay:                 false, // runtime P2P
            is_verified:              p.is_verified,
            is_session_ready:         false, // runtime P2P
            has_queued_messages:      p.has_queued,
        }).collect())
    }

    fn build_contact_list(&self) -> Result<Vec<ContactListItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| ContactListItem {
            contact_id:              p.node_id.clone(),
            peer_id:                 p.node_id.clone(),
            display_name:            p.display_name.clone(),
            initials:                derive_initials(&p.display_name),
            is_online:               false, // runtime P2P
            is_relay:                false, // runtime P2P
            is_verified:             p.verified,
            is_session_ready:        false, // runtime P2P
            direct_conversation_id:  p.node_id, // same as node_id
        }).collect())
    }

    fn build_group_candidates(&self) -> Result<Vec<GroupCandidateItem>> {
        let peers = queries::list_peers(&self.conn)?;
        Ok(peers.into_iter().map(|p| GroupCandidateItem {
            contact_id:   p.node_id.clone(),
            display_name: p.display_name.clone(),
            initials:     derive_initials(&p.display_name),
            is_selected:  self.selected_candidates.contains(&p.node_id),
            is_online:    false, // runtime P2P
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
            return Ok(ConversationView {
                conversation_id:  p.node_id.clone(),
                kind:             ConversationKind::Direct,
                title:            p.display_name.clone(),
                initials:         derive_initials(&p.display_name),
                peer_id:          p.node_id.clone(),
                ticket:           p.endpoint_ticket,
                is_online:        false,
                is_relay:         false,
                is_verified:      p.verified,
                is_session_ready: false,
                connection_stage: "not connected".to_string(),
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
