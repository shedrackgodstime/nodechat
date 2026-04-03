//! Data bridge between `AppEvent` backend types and Slint UI models/properties.
//!
//! `apply_event` is the single point where backend data becomes UI state (RULES.md U-05).
//! Only called from inside `slint::invoke_from_event_loop` closures — never directly.

use slint::VecModel;

use crate::core::commands::AppEvent;
use crate::{
    AppWindow, ChatPreview, ContactData, GroupMessageData, MessageData, SelectionData,
};

/// Translate a backend `AppEvent` into Slint property/model updates.
///
/// This function runs on the Slint event thread. Keep it short — no I/O, no blocking (RULES.md U-04).
pub fn apply_event(ui: &AppWindow, event: AppEvent) {
    match event {
        AppEvent::IncomingMessage { sender, id: _, plaintext: _, timestamp: _ } => {
            // WIRE: push message into the active chat ListView model
            tracing::debug!(peer = %sender, "incoming direct message — UI model update pending");
        }

        AppEvent::IncomingGroupMessage { topic, sender, id: _, plaintext: _, timestamp: _ } => {
            // WIRE: push message into the group chat ListView model
            tracing::debug!(topic = %topic, peer = %sender, "incoming group message — UI model update pending");
        }

        AppEvent::IncomingFile { sender, file_name, path: _ } => {
            // WIRE: show file received notification in chat
            tracing::debug!(peer = %sender, file = %file_name, "incoming file — UI update pending");
        }

        AppEvent::MessageStatusUpdate { id, target, is_group, status } => {
            // WIRE: find bubble in model by id and update its status indicator
            tracing::debug!(
                msg = %id,
                target = %target,
                is_group,
                status = %status.as_str(),
                "status update — refreshing conversation view"
            );
        }

        AppEvent::GroupInviteReceived { topic, group_name } => {
            // WIRE: show group invite notification / dialog
            tracing::debug!(topic = %topic, group = %group_name, "group invite — UI update pending");
        }

        AppEvent::PeerOnlineStatus { peer, online, via_relay } => {
            // WIRE: update the status dot and connection mode label
            tracing::debug!(peer = %peer, online, via_relay, "peer status — UI update pending");
            if ui.get_active_peer_node_id() == peer {
                ui.set_active_peer_online(online);
            }
        }

        AppEvent::PeerHandshakeStage { peer, stage } => {
            tracing::debug!(peer = %peer, stage = %stage, "peer handshake stage update");
            if ui.get_active_peer_node_id() == peer {
                ui.set_active_peer_connection_stage(stage.into());
            }
        }

        AppEvent::PeerContactDetails { peer, endpoint_ticket, verified } => {
            if ui.get_active_peer_node_id() == peer {
                ui.set_active_peer_ticket(endpoint_ticket.into());
                ui.set_active_peer_verified(verified);
            }
        }

        AppEvent::SetupComplete => {
            ui.set_has_identity(true);
            tracing::debug!("setup complete — removing overlay");
        }

        AppEvent::IdentityGenerated { display_name, node_id } => {
            ui.set_my_display_name(display_name.into());
            ui.set_my_node_id(node_id.into());
            ui.set_setup_step(3);
        }

        AppEvent::EndpointTicketUpdated { ticket } => {
            ui.set_my_endpoint_ticket(ticket.into());
            tracing::debug!("endpoint ticket updated");
        }

        AppEvent::MessagesCleared => {
            // WIRE: clear the active message list model and notify user
            tracing::info!("messages cleared — UI refresh pending");
        }

        AppEvent::ConversationDeleted { target, is_group } => {
            if is_group {
                if ui.get_active_group_topic_id() == target {
                    ui.set_active_group_messages(slint::VecModel::from_slice(&[]));
                    ui.set_active_group_topic_id(String::new().into());
                    ui.set_active_group_name(String::new().into());
                    ui.set_active_group_members(String::new().into());
                    ui.set_active_peer_connection_stage(String::new().into());
                    ui.set_current_screen(0);
                }
            } else if ui.get_active_peer_node_id() == target {
                ui.set_active_direct_messages(slint::VecModel::from_slice(&[]));
                ui.set_active_peer_node_id(String::new().into());
                ui.set_active_peer_name(String::new().into());
                ui.set_active_peer_initials(String::new().into());
                ui.set_active_peer_online(false);
                ui.set_active_peer_verified(false);
                ui.set_active_peer_connection_stage(String::new().into());
                ui.set_active_peer_ticket(String::new().into());
                ui.set_current_screen(0);
            }
        }

        AppEvent::ConversationCleared { target, is_group } => {
            if is_group {
                if ui.get_active_group_topic_id() == target {
                    ui.set_active_group_messages(slint::VecModel::from_slice(&[]));
                }
            } else if ui.get_active_peer_node_id() == target {
                ui.set_active_direct_messages(slint::VecModel::from_slice(&[]));
            }
        }

        AppEvent::UnlockComplete => {
            ui.set_is_locked(false);
            ui.set_current_screen(0);
            tracing::debug!("unlock complete — returning user into main app");
        }

        AppEvent::ChatsUpdated { chats } => {
            let rows: Vec<ChatPreview> = chats
                .into_iter()
                .map(|chat| ChatPreview {
                    id: chat.id.into(),
                    name: chat.name.into(),
                    initials: chat.initials.into(),
                    last_message: chat.last_message.into(),
                    timestamp: chat.timestamp.into(),
                    unread: chat.unread,
                    is_group: chat.is_group,
                    is_online: chat.is_online,
                    is_relay: chat.is_relay,
                    is_queued: chat.is_queued,
                    is_verified: chat.is_verified,
                })
                .collect();
            ui.set_chats(VecModel::from_slice(&rows));
        }

        AppEvent::ContactsUpdated { contacts } => {
            let rows: Vec<ContactData> = contacts
                .into_iter()
                .map(|contact| ContactData {
                    id: contact.id.into(),
                    name: contact.name.into(),
                    initials: contact.initials.into(),
                    node_id: contact.node_id.into(),
                    is_online: contact.is_online,
                    is_relay: contact.is_relay,
                    is_verified: contact.is_verified,
                })
                .collect();
            ui.set_contact_directory(VecModel::from_slice(&rows));
        }

        AppEvent::GroupSelectionUpdated { contacts, selected_count } => {
            let rows: Vec<SelectionData> = contacts
                .into_iter()
                .map(|contact| SelectionData {
                    id: contact.id.into(),
                    name: contact.name.into(),
                    initials: contact.initials.into(),
                    is_selected: contact.is_selected,
                    is_online: contact.is_online,
                })
                .collect();
            ui.set_group_member_candidates(VecModel::from_slice(&rows));
            ui.set_selected_group_member_count(selected_count);
        }

        AppEvent::DirectConversationLoaded { target, messages } => {
            if ui.get_active_peer_node_id() != target {
                return;
            }
            let rows: Vec<MessageData> = messages
                .into_iter()
                .map(|msg| MessageData {
                    id: msg.id.into(),
                    text: msg.text.into(),
                    timestamp: msg.timestamp.into(),
                    is_mine: msg.is_mine,
                    status: msg.status.into(),
                    is_ephemeral: msg.is_ephemeral,
                    ttl_seconds: msg.ttl_seconds,
                })
                .collect();
            ui.set_active_direct_messages(VecModel::from_slice(&rows));
        }

        AppEvent::GroupConversationLoaded { topic, messages } => {
            if ui.get_active_group_topic_id() != topic {
                return;
            }
            let rows: Vec<GroupMessageData> = messages
                .into_iter()
                .map(|msg| GroupMessageData {
                    id: msg.id.into(),
                    text: msg.text.into(),
                    timestamp: msg.timestamp.into(),
                    is_mine: msg.is_mine,
                    sender_name: msg.sender_name.into(),
                    status: msg.status.into(),
                })
                .collect();
            ui.set_active_group_messages(VecModel::from_slice(&rows));
        }

        AppEvent::NetworkStatus { direct_peers, relay_peers, is_offline } => {
            ui.set_direct_peers(direct_peers);
            ui.set_relay_peers(relay_peers);
            ui.set_is_offline(is_offline);
        }

        AppEvent::Error { message } => {
            tracing::warn!("backend error surfaced to UI: {}", message);
            let current = ui.get_runtime_error_log().to_string();
            let next = if current.trim().is_empty() {
                message.clone()
            } else {
                format!("{current}\n\n{message}")
            };
            let trimmed = if next.len() > 12_000 {
                next[next.len().saturating_sub(10_000)..].to_owned()
            } else {
                next
            };
            ui.set_runtime_error_log(trimmed.into());
        }
    }
}
