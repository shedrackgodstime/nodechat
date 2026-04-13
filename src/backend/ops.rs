use std::path::Path;
use anyhow::Result;
use crate::contract::{AppEvent, MessageStatus};
use crate::storage::queries;
use crate::p2p::NetworkManager;
use super::RealBackend;

impl RealBackend {
    /// Background task that attempts to send all queued messages for a specific conversation.
    pub fn reconnect_available_peers(&self) -> Result<()> {
        let peers = queries::list_peers(&self.conn)?;
        tracing::info!(count = %peers.len(), "Sweeping peers for proactive connectivity...");
        for peer in peers {
            self.spawn_handshake(peer.node_id.clone(), peer.endpoint_ticket, 1);
            self.begin_message_transmission(peer.node_id);
        }
        Ok(())
    }

    pub fn begin_message_transmission(&self, conversation_id: String) {
        let db = self.db_path.clone();
        let net = self.network.clone();
        let tx = self.event_tx.clone();
        let cid = conversation_id.to_string();
        let nid = self.local_node_id.clone();
        let name = self.local_display_name.clone();
        
        tokio::spawn(async move {
            if let Err(e) = process_queued_messages(&db, &net, &tx, &cid, nid, name).await {
                tracing::error!(peer = %cid, "Message transmission error: {:?}", e);
            }
        });
    }

    pub fn spawn_handshake(&self, target_id: String, ticket: String, stage: u8) {
        let network = self.network.clone();
        let db_path = self.db_path.clone();
        let my_name = self.local_display_name.clone();
        
        tokio::spawn(async move {
            if stage == 1 {
                let _ = execute_peer_handshake(network, db_path, target_id, ticket, my_name).await;
            } else {
                let _ = execute_peer_handshake_ack(network, db_path, target_id, ticket, my_name).await;
            }
        });
    }

    pub(super) fn synchronize_group_history(&self, topic_id: &str) -> Result<()> {
        let network = self.network.clone();
        let db_path = self.db_path.clone();
        let tid = topic_id.to_string();
        
        tokio::spawn(async move {
            let _ = dispatch_sync_query(&db_path, &network, &tid).await;
        });
        Ok(())
    }
}

// ── Background Engine Tasks ───────────────────────────────────────────────────

async fn process_queued_messages(
    db_path: &Path,
    network: &NetworkManager,
    event_tx: &std::sync::mpsc::Sender<AppEvent>,
    conversation_id: &str,
    local_node_id: String,
    my_name: String,
) -> Result<()> {
    let conn = crate::storage::initialize(db_path)?;
    let is_group = queries::get_group(&conn, conversation_id)?.is_some();
    let queued = queries::get_queued_messages(&conn, conversation_id)?;
    if queued.is_empty() { return Ok(()); }

    let ticket_hint = if !is_group {
        queries::get_peer(&conn, conversation_id)?.map(|p| p.endpoint_ticket)
    } else {
        None
    };

    tracing::info!(peer = %conversation_id, count = %queued.len(), "Processing transmission queue...");

    for msg in queued {
        let is_online = network.has_connection(conversation_id);
        let peer = queries::get_peer(&conn, conversation_id)?;
        let is_verified = peer.as_ref().map(|p| p.verified).unwrap_or(false);

        if !is_group && (!is_online || !is_verified) {
             if let Some(ticket) = ticket_hint.clone() {
                 if !is_verified {
                     let net = network.clone();
                     let db = db_path.to_path_buf();
                     let target = conversation_id.to_string();
                     let name = my_name.clone();
                     tokio::spawn(async move {
                         let _ = execute_peer_handshake(net, db, target, ticket, name).await;
                     });
                 } else {
                    let net = network.clone();
                    let target = conversation_id.to_string();
                    tokio::spawn(async move {
                        let _ = net.send_direct(&target, Some(&ticket), vec![]).await; 
                    });
                 }
             }
             break; 
        }

        let frame = crate::p2p::protocol::DirectFrame::Text {
            id: msg.id,
            content: msg.content.clone().into_bytes(),
        }.encode();
        
        let result = if is_group {
            let neighbors = network.group_neighbor_count(conversation_id);
            if neighbors == 0 {
                Err(anyhow::anyhow!("no neighbors"))
            } else {
                let group = queries::get_group(&conn, conversation_id)?;
                let key = group.map(|g| g.symmetric_key).unwrap_or_default();
                
                if key.len() == crate::crypto::KEY_SIZE {
                    let frame = crate::p2p::protocol::GroupFrame {
                        id: msg.id,
                        sender_id: local_node_id.clone(),
                        timestamp: msg.timestamp,
                        content: msg.content.clone().into_bytes(),
                    }.encode();

                    match crate::crypto::encrypt(&frame, &key) {
                        Ok(ciphertext) => network.broadcast_group(conversation_id, ciphertext).await,
                        Err(e) => Err(e),
                    }
                } else {
                    Err(anyhow::anyhow!("invalid group key"))
                }
            }
        } else {
            let peer_pubkey_hex = peer.map(|p| p.x25519_pubkey).unwrap_or_default();
            if peer_pubkey_hex.len() == 64 {
                let mut peer_pubkey_bytes = [0u8; 32];
                if let Ok(bytes) = hex::decode(&peer_pubkey_hex) {
                    peer_pubkey_bytes.copy_from_slice(&bytes[..32]);
                    let identity = queries::get_local_identity(&conn)?.ok_or_else(|| anyhow::anyhow!("No identity"))?;
                    let my_secret_bytes: [u8; 32] = identity.x25519_secret.try_into().unwrap_or([0u8; 32]);
                    let (my_x_secret, _) = crate::crypto::derive_x25519_keypair(&my_secret_bytes);
                    let shared_key = crate::crypto::derive_shared_secret(&my_x_secret.to_bytes(), &peer_pubkey_bytes);
                    
                    match crate::crypto::encrypt(&frame, &shared_key) {
                        Ok(ciphertext) => network.send_direct(conversation_id, ticket_hint.as_deref(), ciphertext).await,
                        Err(_) => network.send_direct(conversation_id, ticket_hint.as_deref(), frame).await,
                    }
                } else {
                    network.send_direct(conversation_id, ticket_hint.as_deref(), frame).await
                }
            } else {
                network.send_direct(conversation_id, ticket_hint.as_deref(), frame).await
            }
        };

        match result {
            Ok(_) => {
                let _ = queries::advance_status(&conn, &msg.id, MessageStatus::Sent);
                let _ = event_tx.send(AppEvent::MessageStatusChanged {
                    conversation_id: conversation_id.to_string(),
                    message_id: msg.id.to_string(),
                    status: MessageStatus::Sent,
                });
                
                if let Ok(chat_list) = RealBackend::build_chat_list_static(&conn, network, &local_node_id) {
                    let _ = event_tx.send(AppEvent::ChatListUpdated(chat_list));
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

pub async fn execute_peer_handshake(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
) -> Result<()> {
    dispatch_handshake_frame(network, db_path, target_id, ticket, my_name, crate::p2p::protocol::HELLO).await
}

async fn execute_peer_handshake_ack(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
) -> Result<()> {
    dispatch_handshake_frame(network, db_path, target_id, ticket, my_name, crate::p2p::protocol::HELLO_ACK).await
}

async fn dispatch_handshake_frame(
    network: NetworkManager,
    db_path: std::path::PathBuf,
    target_id: String,
    ticket: String,
    my_name: String,
    kind: u8,
) -> Result<()> {
    let conn = rusqlite::Connection::open(&db_path)?;
    let identity = queries::get_local_identity(&conn)?.ok_or_else(|| anyhow::anyhow!("no local identity"))?;
    let my_ticket = identity.endpoint_ticket;
    let secret_bytes: [u8; 32] = identity.x25519_secret.clone().try_into().unwrap_or([0u8; 32]);
    let (_, my_pubkey) = crate::crypto::derive_x25519_keypair(&secret_bytes);

    let frame = crate::p2p::protocol::HandshakeFrame {
        kind,
        x25519_public: my_pubkey,
        ticket: my_ticket,
        display_name: my_name,
    };

    network.send_direct(&target_id, Some(&ticket), frame.encode()).await?;
    Ok(())
}

async fn dispatch_sync_query(db_path: &Path, network: &NetworkManager, topic_id: &str) -> Result<()> {
    let conn = crate::storage::initialize(db_path)?;
    let ts = queries::get_latest_received_timestamp(&conn, topic_id)?;
    let frame = crate::p2p::protocol::SyncFrame::Query {
        topic: topic_id.to_owned(),
        after_timestamp: ts,
    }.encode();

    if let Ok(Some(group)) = queries::get_group(&conn, topic_id) {
        if let Ok(ciphertext) = crate::crypto::encrypt(&frame, &group.symmetric_key) {
            network.broadcast_group(topic_id, ciphertext).await?;
        }
    }
    Ok(())
}
