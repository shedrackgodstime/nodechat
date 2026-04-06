//! Data bridge between `AppEvent` backend types and Slint UI models/properties.
//!
//! `apply_event` is the single point where backend data becomes UI state (RULES.md U-05).
//! Only called from inside `slint::invoke_from_event_loop` closures — never directly.

use slint::{VecModel, Model};

use crate::core::commands::AppEvent;
use crate::{
    AppWindow, ChatPreview, ContactData, MessageData, SelectionData, LogEntry,
};

fn push_log(ui: &AppWindow, level: &str, target: &str, message: &str) {
    let mut logs: Vec<LogEntry> = ui.get_debug_logs().iter().collect();
    
    // Fallback timestamp if chrono isn't available, but we'll try to use a simple one
    let timestamp = format!("{:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() % 86400);

    logs.insert(0, LogEntry {
        timestamp: timestamp.into(),
        level: level.into(),
        target: target.into(),
        message: message.into(),
    });

    if logs.len() > 300 {
        logs.truncate(300);
    }

    ui.set_debug_logs(VecModel::from_slice(&logs));
}

fn active_conversation_matches(ui: &AppWindow, target: &str, is_group: bool) -> bool {
    let convo = ui.get_active_conversation();
    convo.id == target && (convo.kind == "group") == is_group
}

fn clear_active_conversation_messages(ui: &AppWindow) {
    ui.set_active_conversation_messages(VecModel::from_slice(&[]));
}

fn reset_active_conversation(ui: &AppWindow) {
    let mut convo = ui.get_active_conversation();
    convo.kind = "direct".into();
    convo.id = String::new().into();
    convo.title = String::new().into();
    convo.initials = String::new().into();
    convo.ticket = String::new().into();
    convo.is_online = false;
    convo.is_session_ready = false;
    convo.is_verified = false;
    convo.connection_stage = String::new().into();
    convo.member_count = "0".into();
    convo.return_screen = 0;
    ui.set_active_conversation(convo);
}

/// Translate a backend `AppEvent` into Slint property/model updates.
///
/// This function runs on the Slint event thread. Keep it short — no I/O, no blocking (RULES.md U-04).
pub fn apply_event(ui: &AppWindow, event: AppEvent) {
    match event {
        AppEvent::IncomingMessage { sender, id: _, plaintext: _, timestamp: _ } => {
            push_log(ui, "INFO", "network", &format!("incoming message from {sender}"));
        }

        AppEvent::IncomingGroupMessage { topic, sender, id: _, plaintext: _, timestamp: _ } => {
            // WIRE: push message into the group chat ListView model
            tracing::debug!(topic = %topic, peer = %sender, "incoming group message — UI model update pending");
        }

        AppEvent::IncomingFile { sender, file_name, path: _ } => {
            // WIRE: show file received notification in chat
            tracing::debug!(peer = %sender, file = %file_name, "incoming file — UI update pending");
        }

        AppEvent::MessageStatusUpdate { id, target: _, is_group: _, status } => {
            push_log(ui, "INFO", "network", &format!("message status update: {id} -> {status:?}"));
        }

        AppEvent::GroupInviteReceived { topic, group_name } => {
            // WIRE: show group invite notification / dialog
            tracing::debug!(topic = %topic, group = %group_name, "group invite — UI update pending");
        }

        AppEvent::PeerOnlineStatus { peer, online, via_relay, session_ready } => {
            // WIRE: update the status dot and connection mode label
            tracing::debug!(peer = %peer, online, via_relay, "peer status — UI update pending");
            let mut convo = ui.get_active_conversation();
            if convo.kind == "direct" && convo.id == peer {
                convo.is_online = online;
                convo.is_session_ready = session_ready;
                ui.set_active_conversation(convo);
            }
        }

        AppEvent::PeerHandshakeStage { peer, stage } => {
            push_log(ui, if stage.contains("failed") { "WARN" } else { "INFO" }, "network", &format!("Handshake with {peer}: {stage}"));
            tracing::debug!(peer = %peer, stage = %stage, "peer handshake stage update");
            let mut convo = ui.get_active_conversation();
            if convo.kind == "direct" && convo.id == peer {
                convo.connection_stage = stage.into();
                ui.set_active_conversation(convo);
            }
        }

        AppEvent::PeerContactDetails { peer, display_name, endpoint_ticket, verified } => {
            let mut convo = ui.get_active_conversation();
            if convo.kind == "direct" && convo.id == peer {
                convo.title = display_name.into();
                convo.ticket = endpoint_ticket.into();
                convo.is_verified = verified;
                ui.set_active_conversation(convo);
            }
        }

        AppEvent::SetupComplete => {
            ui.set_has_identity(true);
            tracing::debug!("setup complete — removing overlay");
        }

        AppEvent::IdentityGenerated { display_name, node_id } => {
            let initials = crate::storage::queries::derive_initials(&display_name);
            ui.set_my_display_name(display_name.into());
            ui.set_my_initials(initials.into());
            ui.set_my_node_id(node_id.into());
            ui.set_setup_step(3);
            push_log(ui, "INFO", "p2p", "New P2P identity generated.");
        }

        AppEvent::IdentityUpdated { display_name } => {
            let initials = crate::storage::queries::derive_initials(&display_name);
            ui.set_my_display_name(display_name.clone().into());
            ui.set_my_initials(initials.into());
            push_log(ui, "INFO", "p2p", &format!("Display name updated to: {}", display_name));
        }

        AppEvent::EndpointTicketUpdated { ticket } => {
            ui.set_my_endpoint_ticket(ticket.clone().into());
            // Extract relay URL from ticket for debug display
            if let Ok(endpoint_ticket) = ticket.parse::<iroh_tickets::endpoint::EndpointTicket>() {
                let relay = endpoint_ticket.endpoint_addr().relay_urls().map(|u| u.to_string()).collect::<Vec<_>>().join(", ");
                ui.set_relay_url(relay.into());
            }
            push_log(ui, "INFO", "p2p", &format!("endpoint ticket updated: {ticket}"));
        }

        AppEvent::MessagesCleared => {
            ui.set_show_confirm_modal(false);
            ui.set_confirm_modal_pin("".into());
            ui.set_confirm_modal_error("".into());
            ui.set_confirm_modal_action("".into());
            clear_active_conversation_messages(ui);
            tracing::info!("messages cleared — modal closed and active conversation models reset");
        }

        AppEvent::ConversationDeleted { target, is_group } => {
            ui.set_show_confirm_modal(false);
            ui.set_confirm_modal_pin("".into());
            ui.set_confirm_modal_error("".into());
            ui.set_confirm_modal_action("".into());
            if active_conversation_matches(ui, &target, is_group) {
                clear_active_conversation_messages(ui);
                reset_active_conversation(ui);
                ui.set_current_screen(0);
            }
            tracing::info!("conversation deleted — modal closed and UI reset");
        }

        AppEvent::ConversationCleared { target, is_group } => {
            ui.set_show_confirm_modal(false);
            ui.set_confirm_modal_pin("".into());
            ui.set_confirm_modal_error("".into());
            ui.set_confirm_modal_action("".into());
            if active_conversation_matches(ui, &target, is_group) {
                clear_active_conversation_messages(ui);
            }
            tracing::info!("conversation history cleared — modal closed");
        }

        AppEvent::UnlockComplete => {
            ui.set_is_locked(false);
            ui.set_lock_pin("".into());
            ui.set_lock_error("".into());
            ui.set_current_screen(0);
            tracing::debug!("unlock complete — returning user into main app");
        }

        AppEvent::AppLocked => {
            ui.set_is_locked(true);
            ui.set_lock_error("".into());
            ui.set_lock_pin("".into());
            tracing::info!("app locked automatically (background/startup)");
        }

        AppEvent::UnlockFailed { error } => {
            ui.set_lock_pin("".into());
            ui.set_lock_error(error.into());
            tracing::warn!("unlock failed — wrong PIN or cooldown active");
        }

        AppEvent::PasswordChanged => {
            ui.set_change_pw_error("".into());
            ui.set_current_screen(5);
            tracing::info!("password changed successfully");
        }

        AppEvent::PasswordChangeFailed { error } => {
            ui.set_change_pw_error(error.into());
            tracing::warn!("password change failed");
        }

        AppEvent::ClearMessagesFailed { error } => {
            tracing::warn!("clear messages failed: {}", error);
            ui.set_confirm_modal_pin("".into());
            ui.set_confirm_modal_error(error.into());
        }

        AppEvent::IdentityDeleted => {
            ui.set_show_confirm_modal(false);
            ui.set_has_identity(false);
            ui.set_setup_step(0);
            ui.set_current_screen(0);
            tracing::info!("identity deleted — resetting app state");
        }

        AppEvent::DeleteIdentityFailed { error } => {
            tracing::warn!("reset application failed: {}", error);
            ui.set_confirm_modal_pin("".into());
            ui.set_confirm_modal_error(error.into());
        }

        AppEvent::PeerVerificationUpdated { peer, verified } => {
            push_log(ui, "INFO", "network", &format!("Peer verification updated: {peer} -> {verified}"));
            let mut active = ui.get_active_conversation();
            if active.id == peer.as_str() {
                active.is_verified = verified;
                ui.set_active_conversation(active);
            }
            tracing::info!(peer = %peer, verified, "UI updated peer verification status");
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
                    is_session_ready: chat.is_session_ready,
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
                    is_session_ready: contact.is_session_ready,
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
            let convo = ui.get_active_conversation();
            if convo.kind != "direct" || convo.id != target {
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
                    is_group_invite: msg.is_group_invite,
                    invite_group_name: msg.invite_group_name.into(),
                    invite_topic_id: msg.invite_topic_id.into(),
                    invite_key: msg.invite_key.into(),
                    invite_is_joined: msg.invite_is_joined,
                })
                .collect();
            ui.set_active_conversation_messages(VecModel::from_slice(&rows));
        }

        AppEvent::GroupConversationLoaded { topic, messages } => {
            let convo = ui.get_active_conversation();
            if convo.kind != "group" || convo.id != topic {
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
                    is_ephemeral: false,
                    ttl_seconds: 0,
                    is_group_invite: false,
                    invite_group_name: "".into(),
                    invite_topic_id: "".into(),
                    invite_key: "".into(),
                    invite_is_joined: false,
                })
                .collect();
            ui.set_active_conversation_messages(VecModel::from_slice(&rows));
        }

        AppEvent::NetworkStatus { direct_peers, relay_peers, is_offline: _ } => {
            ui.set_direct_peers(direct_peers);
            ui.set_relay_peers(relay_peers);
            push_log(ui, "INFO", "network", &format!("status: {direct_peers} direct, {relay_peers} relay"));
        }

        AppEvent::Error { message } => {
            tracing::warn!("backend error surfaced to UI: {}", message);
            push_log(ui, "ERROR", "backend", &message);
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

        AppEvent::Log { level, target, message } => {
            push_log(ui, &level, &target, &message);
        }
    }
}
