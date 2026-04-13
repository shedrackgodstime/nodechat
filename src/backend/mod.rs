//! NodeChat Backend — Modular Actor Implementation
//! ---------------------------------------------------------
//! This module coordinates the application state, database,
//! and P2P network. It is partitioned into sub-modules for
//! better maintainability and architectural clarity.

use std::path::PathBuf;
use anyhow::Result;

use crate::contract::{AppEvent, Command};
use crate::storage::queries;
use crate::p2p::{NetworkEvent, NetworkManager};

pub mod utils;
pub mod views;
pub mod commands;
pub mod events;
pub mod ops;

/// The primary backend controller for NodeChat.
/// Manages the local SQLite database and the P2P network lifecycle.
pub struct RealBackend {
    /// SQLite connection handle.
    conn:                   rusqlite::Connection,
    /// Absolute path to the database file.
    db_path:                PathBuf,
    /// Handle to the P2P network orchestration.
    network:                NetworkManager,
    /// The public Peer ID of this local node (hex encoded).
    local_node_id:          String,
    /// The user-defined display name for this node.
    local_display_name:     String,
    /// Temporary storage for selected peers in group creation flows.
    selected_candidates:    Vec<String>,
    /// The ID of the currently active conversation in the UI.
    active_conversation_id: String,
    /// Channel to send events back to the UI.
    event_tx:               std::sync::mpsc::Sender<AppEvent>,
}

impl RealBackend {
    /// Initializes the backend, opens the database, and begins background network tasks.
    pub fn open(
        network_channel: tokio::sync::mpsc::Sender<NetworkEvent>,
        event_channel: std::sync::mpsc::Sender<AppEvent>,
    ) -> Result<Self> {
        let database_path = utils::resolve_database_path();
        let connection = crate::storage::initialize(&database_path)?;

        let (node_id, display_name, secret_key) =
            match queries::get_local_identity(&connection)? {
                Some(identity) => {
                    tracing::info!(name = %identity.display_name, node_id = %identity.node_id_hex, "Loaded existing local identity");
                    (
                        identity.node_id_hex, 
                        identity.display_name, 
                        Some(iroh::SecretKey::from_bytes(&identity.x25519_secret.try_into().unwrap_or([0u8; 32])))
                    )
                },
                None => {
                    tracing::warn!("No local identity detected; awaiting registration.");
                    (String::new(), String::new(), None)
                },
            };

        let network_manager = NetworkManager::new(network_channel);
        let backend = Self {
            conn: connection,
            db_path: database_path.clone(),
            network: network_manager.clone(),
            local_node_id: node_id.clone(),
            local_display_name: display_name,
            selected_candidates: Vec::new(),
            active_conversation_id: String::new(),
            event_tx: event_channel.clone(),
        };

        // If an identity exists, begin proactive social background tasks
        if let Some(sk) = secret_key {
            let net = network_manager.clone();
            let dp = database_path.clone();
            let etc_tx = event_channel.clone();
            let local_id = node_id.clone();
            tokio::spawn(async move {
                if let Err(e) = net.initialize(Some(sk)).await {
                    tracing::error!("P2P Initialization Failure: {}", e);
                } else {
                    tracing::info!("P2P subsystem ready — active watchdog online");
                    loop {
                        if let Ok(conn) = crate::storage::initialize(&dp) {
                            // 1. Peer Re-dial Loop
                            if let Ok(peers) = queries::list_peers(&conn) {
                                for peer in peers {
                                    let n = net.clone();
                                    tokio::spawn(async move {
                                        let _ = n.dial_peer(&peer.node_id, Some(&peer.endpoint_ticket)).await;
                                    });
                                }
                            }

                            // 2. Refresh UI State (Proactive synchronization)
                            if let Ok(chats) = RealBackend::build_chat_list_static(&conn, &net, &local_id) {
                                let _ = etc_tx.send(AppEvent::ChatListUpdated(chats));
                            }
                            if let Ok(contacts) = RealBackend::build_contact_list_static(&conn, &net) {
                                let _ = etc_tx.send(AppEvent::ContactListUpdated(contacts));
                            }
                            if let Ok(candidates) = RealBackend::build_group_candidates_static(&conn, &net, &[]) {
                                let _ = etc_tx.send(AppEvent::GroupCandidatesUpdated(candidates));
                            }

                            // 3. Group Topic Synchronization
                            if let Ok(groups) = queries::list_groups(&conn) {
                                for group in groups {
                                    let _ = net.subscribe_group(&group.topic_id, vec![]).await;
                                }
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                    }
                }
            });
        }
        
        Ok(backend)
    }

    /// Public interface for processing UI commands.
    pub async fn handle_command(&mut self, command: Command) -> Vec<AppEvent> {
        match self.dispatch(command).await {
            Ok(events) => events,
            Err(error) => {
                tracing::error!("Command Dispatch Error: {error}");
                vec![AppEvent::UserError(error.to_string())]
            }
        }
    }

    /// Proactive helper to list chat data statically from any connection.
    pub fn build_chat_list_static(
        connection: &rusqlite::Connection,
        network: &crate::p2p::NetworkManager,
        local_node_id: &str
    ) -> Result<Vec<crate::contract::ChatListItem>> {
        let previews = queries::list_chat_previews(connection, local_node_id)?;
        Ok(previews.into_iter().map(|p| {
            let neighbor_count = if p.is_group { network.group_neighbor_count(&p.id) } else { 0 };
            let is_online = if p.is_group { neighbor_count > 0 } else { network.has_connection(&p.id) };
            crate::contract::ChatListItem {
                conversation_id:          p.id,
                kind:                     if p.is_group { crate::contract::ConversationKind::Group } else { crate::contract::ConversationKind::Direct },
                title:                    p.title,
                initials:                 p.initials,
                last_message:             if !p.is_session_ready && p.last_message.is_empty() { "Waiting for handshake...".to_string() } else { p.last_message },
                last_message_status:      p.last_message_status,
                is_last_message_outgoing: p.is_outgoing,
                timestamp:                utils::format_hms(p.timestamp),
                member_count:             if p.is_group { (neighbor_count + 1) as i32 } else { 0 },     
                unread_count:             0,     
                is_online,
                is_relay:                 false, 
                is_verified:              p.is_verified,
                is_session_ready:         if p.is_group { neighbor_count > 0 } else { p.is_session_ready },
                has_queued_messages:      p.has_queued,
            }
        }).collect())
    }

    /// Proactive helper to list contact data statically.
    pub fn build_contact_list_static(
        connection: &rusqlite::Connection,
        network: &crate::p2p::NetworkManager,
    ) -> Result<Vec<crate::contract::ContactListItem>> {
        let contacts = queries::list_peers(connection)?;
        Ok(contacts.into_iter().map(|c| crate::contract::ContactListItem {
            contact_id: c.node_id.clone(),
            peer_id: c.node_id.clone(),
            display_name: c.display_name.clone(),
            initials: queries::derive_initials(&c.display_name),
            is_online: network.has_connection(&c.node_id),
            is_relay: false,
            is_verified: c.verified,
            is_session_ready: network.has_connection(&c.node_id), // Assumption: if connected at iroh level, session is likely ready or pending
            direct_conversation_id: c.node_id,
        }).collect())
    }

    /// Proactive helper to list candidates statically.
    pub fn build_group_candidates_static(
        connection: &rusqlite::Connection,
        network: &crate::p2p::NetworkManager,
        selected: &[String],
    ) -> Result<Vec<crate::contract::GroupCandidateItem>> {
        let contacts = queries::list_peers(connection)?;
        Ok(contacts.into_iter().map(|c| crate::contract::GroupCandidateItem {
            contact_id: c.node_id.clone(),
            display_name: c.display_name.clone(),
            initials: queries::derive_initials(&c.display_name),
            is_selected: selected.contains(&c.node_id),
            is_online: network.has_connection(&c.node_id),
        }).collect())
    }
}
