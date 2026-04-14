use anyhow::Result;
use uuid::Uuid;

use crate::contract::{
    AppEvent, Command, ConversationKind, ConversationView, HistoryScope, MessageStatus, IdentityView,
};
use crate::storage::queries::{self, PeerRecord, GroupRecord, LocalIdentityRecord, MessageRecord};
use super::RealBackend;
use super::utils::{current_timestamp, secure_hash_pin};

impl RealBackend {
    pub(super) async fn dispatch(&mut self, command: Command) -> Result<Vec<AppEvent>> {
        match command {
            Command::Refresh => {
                Ok(vec![AppEvent::SnapshotReady(self.build_snapshot()?)])
            }

            Command::LoadConversation { conversation_id } => {
                self.active_conversation_id = conversation_id.clone();
                
                // Mark the conversation as read before rebuilding the visible state.
                queries::mark_as_read(&self.conn, &conversation_id, &self.local_node_id)?;
                let chat_list = self.build_chat_list()?;

                let view = self.build_conversation_view(&conversation_id)?;
                let messages = self.build_message_items(&conversation_id)?;
                Ok(vec![
                    AppEvent::ConversationUpdated(view.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id: view.conversation_id,
                        messages,
                    },
                    AppEvent::ChatListUpdated(chat_list), // Refresh unread counters and preview state.
                ])
            }

            Command::SendMessage { conversation_id, plaintext } => {
                let is_group = queries::get_group(&self.conn, &conversation_id)?.is_some();
                let now = current_timestamp();
                let record = MessageRecord {
                    id:                Uuid::new_v4(),
                    kind:              "standard".to_string(),
                    target_id:         conversation_id.clone(),
                    sender_id:         self.local_node_id.clone(),
                    content:           plaintext.clone(),
                    timestamp:         now,
                    received_at:       now,
                    status:            MessageStatus::Queued,
                    invite_topic_id:   String::new(),
                    invite_group_name: String::new(),
                    invite_key:        String::new(),
                };
                queries::insert_message(&self.conn, &record)?;
                
                if !is_group {
                    if let Ok(Some(peer)) = queries::get_peer(&self.conn, &conversation_id) {
                        if peer.x25519_pubkey.is_empty() {
                            self.spawn_handshake(conversation_id.clone(), peer.endpoint_ticket, 1);
                        }
                    }
                }
                self.begin_message_transmission(conversation_id.clone());

                let message = self.to_message_item(&record)?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::MessageAppended { conversation_id, message },
                    AppEvent::ChatListUpdated(chat_list),
                ])
            }

            Command::RetryQueuedMessage { conversation_id, .. } => {
                self.begin_message_transmission(conversation_id.clone());
                let messages = self.build_message_items(&conversation_id)?;
                Ok(vec![AppEvent::MessageListReplaced { conversation_id, messages }])
            }

            Command::DeleteConversation { conversation_id, .. } => {
                let is_group = queries::get_group(&self.conn, &conversation_id)?.is_some();
                if is_group {
                    let _ = self.network.unsubscribe_group(&conversation_id).await;
                }
                queries::delete_conversation(&self.conn, &conversation_id, is_group)?;
                if self.active_conversation_id == conversation_id {
                    self.active_conversation_id.clear();
                }
                let chat_list = self.build_chat_list()?;
                let contact_list = self.build_contact_list()?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::ContactListUpdated(contact_list),
                    AppEvent::ConversationUpdated(ConversationView::empty(ConversationKind::Direct)),
                    AppEvent::OperationSuccess("delete-chat".to_string()),
                    AppEvent::StatusNotice("Conversation deleted.".to_string()),
                ])
            }

            Command::AddContact { ticket_or_peer_id } => {
                use iroh_tickets::Ticket;
                let node_id = match iroh_tickets::endpoint::EndpointTicket::deserialize(&ticket_or_peer_id) {
                    Ok(ticket) => {
                        let id = ticket.endpoint_addr().id.to_string();
                        id
                    }
                    Err(_) => ticket_or_peer_id.clone()
                };
                let peer = PeerRecord {
                    node_id:         node_id.clone(),
                    display_name:    super::utils::derive_short_name(&node_id),
                    endpoint_ticket: ticket_or_peer_id.clone(),
                    x25519_pubkey:   String::new(),
                    verified:        false,
                };
                queries::upsert_peer(&self.conn, &peer)?;
                
                self.spawn_handshake(node_id.clone(), ticket_or_peer_id.clone(), 1);
                
                let contact_list = self.build_contact_list()?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::ContactListUpdated(contact_list),
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::OperationSuccess("add-contact".to_string()),
                    AppEvent::StatusNotice(format!("Contact added: {}", peer.display_name)),
                ])
            }

            Command::CreateGroup { name, description, .. } => {
                let topic_id = format!("topic-{}", Uuid::new_v4());
                let symmetric_key = crate::crypto::generate_group_key();

                queries::insert_group(&self.conn, &GroupRecord {
                    topic_id:      topic_id.clone(),
                    group_name:    name.clone(),
                    description:   description.clone(),
                    symmetric_key: symmetric_key.clone(),
                })?;

                let active_nodes = self.network.active_connections();
                let _ = self.network.subscribe_group(&topic_id, active_nodes).await;

                for peer_id in std::mem::take(&mut self.selected_candidates) {
                    let invite_payload = serde_json::json!({
                        "type": "group_invite",
                        "topic": topic_id,
                        "key": hex::encode(&symmetric_key),
                        "group_name": name,
                        "description": description,
                    }).to_string();

                    let invite_record = MessageRecord {
                        id:                Uuid::new_v4(),
                        kind:              "group_invite".to_owned(),
                        target_id:         peer_id.clone(),
                        sender_id:         self.local_node_id.clone(),
                        content:           invite_payload,
                        timestamp:         current_timestamp(),
                        received_at:       current_timestamp(),
                        status:            MessageStatus::Queued,
                        invite_topic_id:   topic_id.clone(),
                        invite_group_name: name.clone(),
                        invite_key:        hex::encode(&symmetric_key),
                    };
                    let _ = queries::insert_message(&self.conn, &invite_record);
                    self.begin_message_transmission(peer_id);
                }

                let chat_list = self.build_chat_list()?;
                let candidates = self.build_group_candidates()?;
                let conv_view = self.build_conversation_view(&topic_id)?;
                Ok(vec![
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::GroupCandidatesUpdated(candidates),
                    AppEvent::ConversationUpdated(conv_view),
                    AppEvent::OperationSuccess("create-group".to_string()),
                    AppEvent::StatusNotice(format!("Group created: {}", name)),
                ])
            }

            Command::InviteToGroup { group_id } => {
                let group = queries::get_group(&self.conn, &group_id)?;
                if group.is_none() {
                    return Ok(vec![AppEvent::UserError("Group not found.".to_string())]);
                }
                let group = group.unwrap();
                let mut chats_updated = false;

                for peer_id in std::mem::take(&mut self.selected_candidates) {
                    let invite_payload = serde_json::json!({
                        "type": "group_invite",
                        "topic": group_id,
                        "key": hex::encode(&group.symmetric_key),
                        "group_name": group.group_name,
                        "description": group.description,
                    }).to_string();

                    let invite_record = MessageRecord {
                        id:                Uuid::new_v4(),
                        kind:              "group_invite".to_owned(),
                        target_id:         peer_id.clone(),
                        sender_id:         self.local_node_id.clone(),
                        content:           invite_payload,
                        timestamp:         current_timestamp(),
                        received_at:       current_timestamp(),
                        status:            MessageStatus::Queued,
                        invite_topic_id:   group_id.clone(),
                        invite_group_name: group.group_name.clone(),
                        invite_key:        hex::encode(&group.symmetric_key),
                    };
                    let _ = queries::insert_message(&self.conn, &invite_record);
                    self.begin_message_transmission(peer_id);
                    chats_updated = true;
                }

                let candidates = self.build_group_candidates()?;
                let mut events = vec![
                    AppEvent::GroupCandidatesUpdated(candidates),
                    AppEvent::OperationSuccess("send-invites".to_string()),
                ];
                if chats_updated {
                    if let Ok(list) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(list)); }
                }
                Ok(events)
            }

            Command::ToggleGroupCandidate { contact_id } => {
                if let Some(pos) = self.selected_candidates.iter().position(|id| *id == contact_id) {
                    self.selected_candidates.remove(pos);
                } else {
                    self.selected_candidates.push(contact_id);
                }
                let candidates = self.build_group_candidates()?;
                Ok(vec![AppEvent::GroupCandidatesUpdated(candidates)])
            }

            Command::OpenDirectConversation { contact_id } => {
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

            Command::CreateIdentity { display_name, pin } => {
                let pin_hash = secure_hash_pin(&pin);
                let mut secret_bytes = [0u8; 32];
                rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret_bytes);
                let secret_key = iroh::SecretKey::from_bytes(&secret_bytes);
                let node_id_hex = hex::encode(secret_key.public().as_bytes());

                let _ = self.network.initialize(Some(secret_key.clone())).await;
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
                    AppEvent::StatusNotice("Identity created successfully.".to_string())
                ])
            }

            Command::FinalizeIdentity => Ok(vec![]),

            Command::UnlockApp { pin } => {
                match queries::get_local_identity(&self.conn)? {
                    None => Ok(vec![AppEvent::UserError("No local identity was found.".to_string())]),
                    Some(id) => {
                        if id.pin_hash.is_empty() || id.pin_hash == secure_hash_pin(&pin) {
                            self.local_node_id = id.node_id_hex;
                            self.local_display_name = id.display_name;
                            let secret_bytes: [u8; 32] = id.x25519_secret.clone().try_into().unwrap_or([0u8; 32]);
                            let secret_key = iroh::SecretKey::from_bytes(&secret_bytes);
                            let _ = self.network.initialize(Some(secret_key)).await;

                            let identity = self.build_identity_view()?;
                            Ok(vec![
                                AppEvent::IdentityUpdated(identity),
                                AppEvent::OperationSuccess("unlock".to_string()),
                            ])
                        } else {
                            Ok(vec![AppEvent::UserError("Incorrect PIN.".to_string())])
                        }
                    }
                }
            }

            Command::ChangePassword { current_pin, new_pin } => {
                match queries::get_local_identity(&self.conn)? {
                    None => Ok(vec![AppEvent::UserError("No local identity was found.".to_string())]),
                    Some(id) => {
                        if id.pin_hash.is_empty() || id.pin_hash == secure_hash_pin(&current_pin) {
                            queries::update_pin_hash(&self.conn, &secure_hash_pin(&new_pin))?;
                            let identity = self.build_identity_view()?;
                            Ok(vec![
                                AppEvent::IdentityUpdated(identity),
                                AppEvent::OperationSuccess("password-update".to_string()),
                                AppEvent::StatusNotice("Password updated.".to_string())
                            ])
                        } else {
                            Ok(vec![AppEvent::UserError("Incorrect current PIN.".to_string())])
                        }
                    }
                }
            }

            Command::ShareContact { contact_id } => {
                let peer = queries::get_peer(&self.conn, &contact_id)?.ok_or_else(|| anyhow::anyhow!("Contact not found"))?;
                let payload = serde_json::json!({
                    "type": "contact_share",
                    "node_id": peer.node_id,
                    "name": peer.display_name,
                    "ticket": peer.endpoint_ticket,
                }).to_string();

                let mut events = vec![];
                let targets = std::mem::take(&mut self.selected_candidates);

                for target_id in targets {
                    let record = MessageRecord {
                        id:                Uuid::new_v4(),
                        kind:              "contact_share".to_string(),
                        target_id:         target_id.clone(),
                        sender_id:         self.local_node_id.clone(),
                        content:           payload.clone(),
                        timestamp:         current_timestamp(),
                        received_at:       current_timestamp(),
                        status:            MessageStatus::Queued,
                        invite_topic_id:   String::new(),
                        invite_group_name: String::new(),
                        invite_key:        String::new(),
                    };
                    let _ = queries::insert_message(&self.conn, &record);
                    self.begin_message_transmission(target_id.clone());

                    if self.active_conversation_id == target_id {
                        if let Ok(msg) = self.to_message_item(&record) {
                            events.push(AppEvent::MessageAppended { conversation_id: target_id.clone(), message: msg });
                        }
                    }
                }
                if let Ok(list) = self.build_chat_list() { events.push(AppEvent::ChatListUpdated(list)); }
                events.push(AppEvent::StatusNotice(format!("Contact shared: {}", peer.display_name)));
                Ok(events)
            }

            Command::UpdateDisplayName { display_name } => {
                if display_name.trim().is_empty() {
                    return Ok(vec![AppEvent::UserError("Display name cannot be empty.".to_string())]);
                }
                queries::update_display_name(&self.conn, &display_name)?;
                self.local_display_name = display_name;
                Ok(vec![AppEvent::SnapshotReady(self.build_snapshot()?)])
            }

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
                    AppEvent::OperationSuccess("reset-identity".to_string()),
                    AppEvent::StatusNotice("Identity reset.".to_string()),
                ])
            }

            Command::SetVerification { peer_id, verified } => {
                queries::set_peer_verified(&self.conn, &peer_id, verified)?;
                Ok(vec![
                    AppEvent::ContactListUpdated(self.build_contact_list()?),
                    AppEvent::ChatListUpdated(self.build_chat_list()?),
                    AppEvent::ConversationUpdated(self.build_conversation_view(&peer_id)?),
                ])
            }

            Command::AcceptGroupInvite { conversation_id, topic_id, invite_key } => {
                let key_bytes = if invite_key.is_empty() { vec![0u8; 32] } else { hex::decode(&invite_key).unwrap_or_else(|_| vec![0u8; 32]) };
                let match_msg = queries::list_messages(&self.conn, &conversation_id)?.into_iter().filter(|m| m.invite_topic_id == topic_id).last();

                let mut group_name = match_msg.as_ref().map(|m| m.invite_group_name.clone()).unwrap_or_default();
                let mut description = String::new();

                if let Some(msg) = match_msg {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&msg.content) {
                        if group_name.is_empty() { group_name = val.get("group_name").and_then(|v| v.as_str()).unwrap_or("").to_string(); }
                        description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    }
                }

                if group_name.is_empty() { group_name = format!("Group {}", &topic_id[..topic_id.len().min(8)]); }
                if !queries::group_exists(&self.conn, &topic_id)? {
                    queries::insert_group(&self.conn, &GroupRecord { topic_id: topic_id.clone(), group_name: group_name.clone(), description, symmetric_key: key_bytes })?;
                }
                let bootstrap = self.network.active_connections();
                let _ = self.network.subscribe_group(&topic_id, bootstrap).await;

                for msg in queries::list_messages(&self.conn, &conversation_id)? {
                    if msg.invite_topic_id == topic_id { let _ = queries::advance_status(&self.conn, &msg.id, MessageStatus::Read); }
                }

                Ok(vec![
                    AppEvent::ChatListUpdated(self.build_chat_list()?),
                    AppEvent::OperationSuccess("join-group".to_string()),
                    AppEvent::StatusNotice(format!("Joined group: {}", group_name)),
                ])
            }

            Command::ClearMessageHistory { scope, .. } => {
                match scope {
                    HistoryScope::AllConversations => queries::clear_messages(&self.conn)?,
                    HistoryScope::ActiveConversation => if !self.active_conversation_id.is_empty() {
                         queries::clear_conversation(&self.conn, &self.active_conversation_id)?;
                    }
                }
                Ok(vec![
                    AppEvent::ChatListUpdated(self.build_chat_list()?),
                    AppEvent::MessageListReplaced { conversation_id: self.active_conversation_id.clone(), messages: vec![] },
                    AppEvent::OperationSuccess("clear-history".to_string()),
                    AppEvent::StatusNotice("Message history cleared.".to_string()),
                ])
            }
            
            Command::DeleteMessage { message_id } => {
                queries::delete_message(&self.conn, &message_id)?;
                let convo_id = self.active_conversation_id.clone();
                let messages = self.build_message_items(&convo_id)?;
                let chat_list = self.build_chat_list()?;
                Ok(vec![
                    AppEvent::MessageListReplaced { conversation_id: convo_id, messages },
                    AppEvent::ChatListUpdated(chat_list),
                    AppEvent::StatusNotice("Message deleted.".to_string()),
                ])
            }
        }
    }
}
