use anyhow::Result;
use uuid::Uuid;

use crate::contract::{AppEvent, MessageStatus};
use crate::storage::queries::{self, PeerRecord, MessageRecord};
use crate::p2p::NetworkEvent;
use super::RealBackend;
use super::utils::current_timestamp;

impl RealBackend {
    /// Process incoming background P2P events.
    pub async fn handle_network_event(&mut self, event: NetworkEvent) -> Vec<AppEvent> {
        let mut events = vec![];
        match event {
            NetworkEvent::PeerConnected { node_id, .. } => {
                tracing::info!(peer=%node_id, "peer connected at transport level");
                self.begin_message_transmission(node_id.clone());
                
                events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));

                if let Ok(groups) = queries::list_groups(&self.conn) {
                    let net = self.network.clone();
                    let nid = node_id.clone();
                    tokio::spawn(async move {
                        for g in groups { let _ = net.subscribe_group(&g.topic_id, vec![nid.clone()]).await; }
                    });
                }
                
                if self.active_conversation_id == node_id {
                    if let Ok(conv) = self.build_conversation_view(&node_id) { events.push(AppEvent::ConversationUpdated(conv)); }
                }
            }
            NetworkEvent::PeerDisconnected { node_id } => {
                tracing::info!(peer=%node_id, "peer disconnected");
                events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));
                if self.active_conversation_id == node_id {
                    if let Ok(conv) = self.build_conversation_view(&node_id) { events.push(AppEvent::ConversationUpdated(conv)); }
                }
            }
            NetworkEvent::DirectMessage { from, payload } => {
                use crate::p2p::protocol::{DirectFrame, DIRECT_MAGIC, MAGIC, HandshakeFrame, HELLO, HELLO_ACK};
                
                if payload.starts_with(&MAGIC) {
                    if let Ok(frame) = HandshakeFrame::decode(&payload) {
                        tracing::info!(peer=%from, kind=%frame.kind, name=%frame.display_name, "NC1H handshake received");
                        let _ = queries::upsert_peer(&self.conn, &PeerRecord {
                            node_id:         from.clone(),
                            display_name:    frame.display_name.clone(),
                            endpoint_ticket: frame.ticket.clone(),
                            x25519_pubkey:   hex::encode(frame.x25519_public),
                            verified:        true,
                        });

                        if frame.kind == HELLO {
                            self.spawn_handshake(from.clone(), frame.ticket.clone(), 2);
                            self.begin_message_transmission(from.clone());
                        } else if frame.kind == HELLO_ACK {
                            self.begin_message_transmission(from.clone());
                        }
                        events.push(AppEvent::ContactListUpdated(self.build_contact_list().unwrap_or_default()));
                        events.push(AppEvent::ChatListUpdated(self.build_chat_list().unwrap_or_default()));
                        if self.active_conversation_id == from {
                            if let Ok(conv) = self.build_conversation_view(&from) { events.push(AppEvent::ConversationUpdated(conv)); }
                        }
                    }
                } else {
                    let mut plaintext_payload = payload.clone();
                    if let Ok(Some(peer)) = queries::get_peer(&self.conn, &from) {
                        if let Ok(Some(me)) = queries::get_local_identity(&self.conn) {
                            if peer.x25519_pubkey.len() == 64 {
                                if let Ok(bytes) = hex::decode(&peer.x25519_pubkey) {
                                    let mut peer_pubkey_bytes = [0u8; 32];
                                    peer_pubkey_bytes.copy_from_slice(&bytes[..32]);
                                    let my_secret_bytes: [u8; 32] = me.x25519_secret.try_into().unwrap_or([0u8; 32]);
                                    let (my_x_secret, _) = crate::crypto::derive_x25519_keypair(&my_secret_bytes);
                                    let shared_key = crate::crypto::derive_shared_secret(&my_x_secret.to_bytes(), &peer_pubkey_bytes);
                                    if let Ok(decrypted) = crate::crypto::decrypt(&payload, &shared_key) {
                                        plaintext_payload = decrypted;
                                    }
                                }
                            }
                        }
                    }

                    if plaintext_payload.starts_with(&DIRECT_MAGIC) {
                        if let Ok(frame) = DirectFrame::decode(&plaintext_payload) {
                            match frame {
                                DirectFrame::Text { id, content } => {
                                    let text = String::from_utf8_lossy(&content).to_string();
                                    let mut kind = "standard".to_string();
                                    let mut invite_topic_id = String::new();
                                    let mut invite_group_name = String::new();
                                    let mut invite_key = String::new();

                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(msg_type) = parsed.get("type").and_then(|t| t.as_str()) {
                                            if msg_type == "contact_share" { kind = "contact_share".to_string(); }
                                            else if msg_type == "group_invite" {
                                                kind = "group_invite".to_string();
                                                invite_topic_id = parsed.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                invite_group_name = parsed.get("group_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                invite_key = parsed.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            }
                                        }
                                    }

                                    let status = if self.active_conversation_id == from { MessageStatus::Read } else { MessageStatus::Delivered };

                                    let record = MessageRecord {
                                        id, kind, target_id: from.clone(), sender_id: from.clone(), content: text,
                                        timestamp: current_timestamp(), received_at: current_timestamp(),
                                        status, invite_topic_id, invite_group_name, invite_key,
                                    };
                                    let _ = queries::insert_message(&self.conn, &record);
                                    let _ = self.send_receipt(from.clone(), id, status == MessageStatus::Read);


                                    
                                    if let Ok(msg) = self.to_message_item(&record) {
                                        events.push(AppEvent::MessageAppended { conversation_id: from.clone(), message: msg });
                                        if let Ok(chat_list) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(chat_list)); }
                                    }
                                }
                                DirectFrame::Receipt { id, .. } => {
                                    let _ = queries::advance_status(&self.conn, &id, MessageStatus::Delivered);
                                    events.push(AppEvent::MessageStatusChanged { conversation_id: from.clone(), message_id: id.to_string(), status: MessageStatus::Delivered });
                                    if let Ok(chat_list) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(chat_list)); }
                                }
                            }
                        }
                    }
                }
            }
            NetworkEvent::GroupMessage { topic, payload, .. } => {
                if let Ok(Some(group)) = queries::get_group(&self.conn, &topic) {
                    if let Ok(plaintext) = crate::crypto::decrypt(&payload, &group.symmetric_key) {
                        if plaintext.starts_with(&crate::p2p::protocol::GROUP_MAGIC) {
                            if let Ok(frame) = crate::p2p::protocol::GroupFrame::decode(&plaintext) {
                                if let Ok(text) = String::from_utf8(frame.content) {
                                    let status = if self.active_conversation_id == topic { MessageStatus::Read } else { MessageStatus::Delivered };

                                    let record = MessageRecord {
                                        id: frame.id, kind: "standard".to_string(), target_id: topic.clone(), sender_id: frame.sender_id.clone(),
                                        content: text, timestamp: frame.timestamp, received_at: current_timestamp(), status,
                                        invite_topic_id: String::new(), invite_group_name: String::new(), invite_key: String::new(),
                                    };
                                    let _ = queries::insert_message(&self.conn, &record);


                                    if let Ok(msg) = self.to_message_item(&record) {
                                        events.push(AppEvent::MessageAppended { conversation_id: topic.clone(), message: msg });
                                        if let Ok(chat_list) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(chat_list)); }
                                    }
                                }
                            }
                        } else if plaintext.starts_with(&crate::p2p::protocol::SYNC_MAGIC) {
                            if let Ok(sync) = crate::p2p::protocol::SyncFrame::decode(&plaintext) {
                                match sync {
                                    crate::p2p::protocol::SyncFrame::Query { topic, after_timestamp } => {
                                        if let Ok(messages) = queries::list_messages_after(&self.conn, &topic, after_timestamp) {
                                            if !messages.is_empty() {
                                                let frames: Vec<_> = messages.into_iter().map(|m| crate::p2p::protocol::GroupFrame {
                                                    id: m.id, sender_id: m.sender_id, timestamp: m.timestamp, content: m.content.into_bytes(),
                                                }).collect();
                                                let reply = crate::p2p::protocol::SyncFrame::Reply { topic: topic.clone(), messages: frames }.encode();
                                                if let Ok(ciphertext) = crate::crypto::encrypt(&reply, &group.symmetric_key) {
                                                    let net = self.network.clone();
                                                    let t = topic.clone();
                                                    tokio::spawn(async move { let _ = net.broadcast_group(&t, ciphertext).await; });
                                                }
                                            }
                                        }
                                    }
                                    crate::p2p::protocol::SyncFrame::Reply { topic, messages } => {
                                        for frame in messages {
                                            if let Ok(text) = String::from_utf8(frame.content) {
                                                let _ = queries::insert_message(&self.conn, &MessageRecord {
                                                    id: frame.id, kind: "standard".to_string(), target_id: topic.clone(), sender_id: frame.sender_id.clone(),
                                                    content: text, timestamp: frame.timestamp, received_at: current_timestamp(), status: MessageStatus::Delivered,
                                                    invite_topic_id: String::new(), invite_group_name: String::new(), invite_key: String::new(),
                                                });
                                                
                                                // Note: We don't notify for every single synced message to avoid spamming the user.
                                                // We only notify for the latest one if it's new.
                                            }
                                        }
                                        if self.active_conversation_id == topic {
                                            if let Ok(msgs) = self.build_message_items(&topic) { events.push(AppEvent::MessageListReplaced { conversation_id: topic.clone(), messages: msgs }); }
                                        }
                                        if let Ok(chats) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(chats)); }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            NetworkEvent::GroupNeighborUp { topic, .. } | NetworkEvent::GroupNeighborDown { topic, .. } => {
                let _ = self.synchronize_group_history(&topic);
                self.begin_message_transmission(topic.clone());
                if let Ok(chats) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(chats)); }
                if self.active_conversation_id == topic {
                    if let Ok(conv) = self.build_conversation_view(&topic) { events.push(AppEvent::ConversationUpdated(conv)); }
                }
            }
        }
        events
    }

    fn send_receipt(&self, target: String, message_id: Uuid, is_read: bool) -> Result<()> {
        let frame = crate::p2p::protocol::DirectFrame::Receipt { id: message_id, is_read }.encode();
        let net = self.network.clone();
        tokio::spawn(async move { let _ = net.send_direct(&target, None, frame).await; });
        Ok(())
    }
}
