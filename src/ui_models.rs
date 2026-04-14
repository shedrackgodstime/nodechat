//! Maps backend contract types into the Slint models consumed by the UI.

use slint::{VecModel, Model};
use crate::{AppWindow, ChatPreview, ContactData, MessageData, GroupData, GroupCandidateItem, AppInfo};
use crate::contract::{AppEvent, ChatListItem, ContactListItem, MessageItem, ConversationView, IdentityView, GroupCandidateItem as RustCandidate, AppInfoView};

pub fn apply_event(ui: &AppWindow, event: AppEvent) {
    match event {
        AppEvent::SnapshotReady(snapshot) => {
            apply_identity(ui, snapshot.identity);
            apply_app_info(ui, snapshot.app_info);
            apply_app_flags(ui, snapshot.app_flags);
            apply_chats(ui, snapshot.chat_list);
            apply_contacts(ui, snapshot.contact_list);
            apply_conversation(ui, snapshot.active_conversation);
            apply_messages(ui, snapshot.active_messages);
            apply_group_candidates(ui, snapshot.group_candidates);
        }
        AppEvent::IdentityUpdated(identity) => {
            apply_identity(ui, identity);
        }
        AppEvent::ChatListUpdated(chats) => {
            apply_chats(ui, chats);
        }
        AppEvent::ContactListUpdated(contacts) => {
            apply_contacts(ui, contacts);
        }
        AppEvent::ConversationUpdated(convo) => {
            apply_conversation(ui, convo);
        }
        AppEvent::MessageListReplaced { messages, .. } => {
            apply_messages(ui, messages);
        }
        AppEvent::MessageAppended { conversation_id, message } => {
            if ui.get_active_conversation().id == conversation_id {
                append_message(ui, message);
            }
        }
        AppEvent::GroupCandidatesUpdated(candidates) => {
            apply_group_candidates(ui, candidates);
        }
        AppEvent::DebugFeedUpdated(lines) => {
            let text = lines.join("\n");
            ui.set_debug_logs(text.into());
        }
        AppEvent::Log { level: _, message } => {
            // Keep the in-app debug console bounded so repeated logs do not grow without limit.
            let current = ui.get_debug_logs().to_string();
            let mut lines: Vec<String> = current.lines().map(|s| s.to_string()).collect();
            lines.push(message);
            let limit = 500;
            if lines.len() > limit {
                lines.drain(0..lines.len() - limit);
            }
            ui.set_debug_logs(lines.join("\n").into());
        }
        AppEvent::StatusNotice(msg) => {
            eprintln!("[STATUS] {}", msg);
            ui.set_global_notice_text(msg.into());
            ui.set_global_notice_is_error(false);
            ui.set_global_notice_visible(true);
        }
        AppEvent::UserError(err) => {
            eprintln!("[ERROR] {}", err);
            ui.set_global_notice_text(err.into());
            ui.set_global_notice_is_error(true);
            ui.set_global_notice_visible(true);
        }
        AppEvent::MessageStatusChanged { conversation_id, message_id, status } => {
            update_message_status(ui, &conversation_id, &message_id, status);
        }
        AppEvent::OperationSuccess(slug) => {
            ui.set_operation_success_slug(slug.into());
            ui.invoke_do_operation_success();
        }
    }
}

fn update_message_status(ui: &AppWindow, conversation_id: &str, message_id: &str, status: crate::contract::MessageStatus) {
    // Update the visible message list first so delivery state changes appear immediately.
    let active = ui.get_active_conversation();
    if active.id == conversation_id {
        let model = ui.get_active_messages();
        for i in 0..model.row_count() {
            if let Some(mut msg) = model.row_data(i) {
                if msg.id == message_id {
                    msg.status = status.to_string().into();
                    model.set_row_data(i, msg);
                    break;
                }
            }
        }
    }

    // Mirror the same status in the chat preview row.
    let chats_model = ui.get_chats();
    for i in 0..chats_model.row_count() {
        if let Some(mut chat) = chats_model.row_data(i) {
            if chat.id == conversation_id {
                chat.last_message_status = status.to_string().into();
                // Queue badges are refreshed from a full chat list rebuild when needed.
                chats_model.set_row_data(i, chat);
                break;
            }
        }
    }
}

fn apply_identity(ui: &AppWindow, identity: IdentityView) {
    ui.set_display_name(identity.display_name.into());
    ui.set_initials(identity.initials.into());
    ui.set_endpoint_ticket(identity.endpoint_ticket.into());
    ui.set_has_identity(identity.has_identity);
    ui.set_is_locked(identity.is_locked);
    ui.set_has_password(identity.has_password);
}

fn apply_app_info(ui: &AppWindow, info: AppInfoView) {
    ui.set_app_info(AppInfo {
        name: info.name.into(),
        version: info.version.into(),
        version_type: info.version_type.into(),
        description: info.description.into(),
        website: info.website.into(),
        repo: info.repo.into(),
    });
}

fn apply_app_flags(ui: &AppWindow, flags: crate::contract::AppFlags) {
    ui.set_direct_peers(flags.direct_peer_count);
    ui.set_relay_peers(flags.relay_peer_count);
    ui.set_is_offline(flags.is_offline);
}

fn apply_chats(ui: &AppWindow, chats: Vec<ChatListItem>) {
    let direct_online = chats
        .iter()
        .filter(|c| matches!(c.kind, crate::contract::ConversationKind::Direct) && c.is_online && !c.is_relay)
        .count() as i32;
    let relay_online = chats
        .iter()
        .filter(|c| matches!(c.kind, crate::contract::ConversationKind::Direct) && c.is_online && c.is_relay)
        .count() as i32;
    let group_online = chats
        .iter()
        .filter(|c| matches!(c.kind, crate::contract::ConversationKind::Group) && c.is_online)
        .count() as i32;

    let rows: Vec<ChatPreview> = chats.iter().map(|c| ChatPreview {
        id: c.conversation_id.clone().into(),
        name: c.title.clone().into(),
        initials: c.initials.clone().into(),
        last_message: c.last_message.replace('\n', " ").replace('\r', "").trim().into(),
        timestamp: c.timestamp.clone().into(),
        unread: c.unread_count,
        is_group: matches!(c.kind, crate::contract::ConversationKind::Group),
        is_online: c.is_online,
        is_relay: c.is_relay,
        is_verified: c.is_verified,
        is_session_ready: c.is_session_ready,
        has_queued_messages: c.has_queued_messages,
        last_message_status: c.last_message_status.to_string().into(),
        is_last_message_outgoing: c.is_last_message_outgoing,
        member_count: c.member_count,
    }).collect();
    ui.set_chats(VecModel::from_slice(&rows).into());
    ui.set_direct_peers(direct_online);
    ui.set_relay_peers(relay_online);
    ui.set_is_offline(direct_online == 0 && relay_online == 0 && group_online == 0);

    // Reuse the chat snapshot to populate the group picker shown from the contacts screen.
    let groups: Vec<GroupData> = chats.into_iter()
        .filter(|c| matches!(c.kind, crate::contract::ConversationKind::Group))
        .map(|c| GroupData {
            id: c.conversation_id.into(),
            name: c.title.into(),
            initials: c.initials.into(),
            topic: "".into(),
            secret_key: "".into(),
            member_count: c.member_count,
        })
        .collect();
    ui.set_groups(VecModel::from_slice(&groups).into());
}

fn apply_contacts(ui: &AppWindow, contacts: Vec<ContactListItem>) {
    let rows: Vec<ContactData> = contacts.into_iter().map(|c| ContactData {
        id: c.contact_id.into(),
        name: c.display_name.into(),
        initials: c.initials.into(),
        node_id: c.peer_id.into(),
        ticket: "".into(), // Summary list doesn't include ticket
        is_online: c.is_online,
        is_session_ready: c.is_session_ready,
        is_relay: c.is_relay,
        is_verified: c.is_verified,
        direct_conversation_id: c.direct_conversation_id.into(),
    }).collect();
    ui.set_contacts(VecModel::from_slice(&rows).into());
}

fn apply_conversation(ui: &AppWindow, convo: ConversationView) {
    let mut ctx = ui.get_active_conversation();
    ctx.id = convo.conversation_id.clone().into();
    ctx.kind = convo.kind.to_string().into();
    ctx.title = convo.title.clone().into();
    ctx.initials = convo.initials.clone().into();
    ctx.node_id = convo.peer_id.clone().into();
    ctx.ticket = convo.ticket.clone().into();
    ctx.is_online = convo.is_online;
    ctx.is_relay = convo.is_relay;
    ctx.is_verified = convo.is_verified;
    ctx.is_session_ready = convo.is_session_ready;
    ctx.connection_stage = convo.connection_stage.into();
    ui.set_active_conversation(ctx);
    
    // Keep the group-specific state in sync for invite and membership flows.
    if matches!(convo.kind, crate::contract::ConversationKind::Group) {
         ui.set_active_group(GroupData {
             id: convo.conversation_id.into(),
             name: convo.title.into(),
             initials: convo.initials.into(),
            topic: convo.ticket.into(), // Reuses the shared conversation identifier slot for groups.
             secret_key: "".into(),
             member_count: convo.member_count,
         });
    }
}

fn apply_messages(ui: &AppWindow, messages: Vec<MessageItem>) {
    let rows: Vec<MessageData> = messages.into_iter().map(map_message).collect();
    ui.set_active_messages(VecModel::from_slice(&rows).into());
}

fn append_message(ui: &AppWindow, message: MessageItem) {
    let messages = ui.get_active_messages();
    if let Some(model) = messages.as_any().downcast_ref::<VecModel<MessageData>>() {
        model.push(map_message(message));
    } else {
        // Replace the backing model when Slint is still holding the default empty array.
        let mut rows: Vec<MessageData> = messages.iter().collect();
        rows.push(map_message(message));
        ui.set_active_messages(VecModel::from_slice(&rows).into());
    }
}

fn apply_group_candidates(ui: &AppWindow, candidates: Vec<RustCandidate>) {
    let mut selected_count = 0;
    let rows: Vec<GroupCandidateItem> = candidates.into_iter().map(|c| {
        if c.is_selected { selected_count += 1; }
        GroupCandidateItem {
            contact_id: c.contact_id.into(),
            display_name: c.display_name.into(),
            initials: c.initials.into(),
            is_selected: c.is_selected,
            is_online: c.is_online,
        }
    }).collect();
    ui.set_group_candidates(VecModel::from_slice(&rows).into());
    ui.set_group_candidates_selected(selected_count);
}

fn map_message(m: MessageItem) -> MessageData {
    let mut is_contact_share = matches!(m.kind, crate::contract::MessageKind::ContactShare);
    let mut share_name = String::new();
    let mut share_node_id = String::new();
    let mut share_ticket = String::new();

    if is_contact_share {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&m.text) {
            share_name = value.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            share_node_id = value.get("node_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            share_ticket = value.get("ticket").and_then(|v| v.as_str()).unwrap_or("").to_string();
        } else {
            is_contact_share = false;
        }
    }

    let mut invite_desc = String::new();
    if matches!(m.kind, crate::contract::MessageKind::GroupInvite) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&m.text) {
             invite_desc = value.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        }
    }

    MessageData {
        id: m.message_id.into(),
        sender: m.sender_name.into(),
        text: m.text.into(),
        timestamp: m.timestamp.into(),
        is_outgoing: m.is_outgoing,
        is_system: m.is_system,
        is_invite: matches!(m.kind, crate::contract::MessageKind::GroupInvite),
        invite_topic: m.invite_topic_id.into(),
        invite_name: m.invite_group_name.into(),
        invite_desc: invite_desc.into(),
        invite_key: m.invite_key.into(),
        is_invite_joined: m.invite_is_joined,
        status: m.status.to_string().into(),
        received_timestamp: m.received_timestamp.into(),
        is_delayed: m.is_delayed,
        is_contact_share,
        share_name: share_name.into(),
        share_node_id: share_node_id.into(),
        share_ticket: share_ticket.into(),
    }
}
