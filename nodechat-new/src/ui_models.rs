// NodeChat New — UI Data Mapping
// ---------------------------------------------------------
// Maps Rust contract types to Slint UI properties and models.

use slint::{VecModel, Model};
use crate::{AppWindow, ChatPreview, ContactData, MessageData, GroupData, GroupCandidateItem, AppInfo};
use crate::contract::{AppEvent, ChatListItem, ContactListItem, MessageItem, ConversationView, IdentityView, GroupCandidateItem as RustCandidate, AppInfoView};

pub fn apply_event(ui: &AppWindow, event: AppEvent) {
    match event {
        AppEvent::SnapshotReady(snapshot) => {
            apply_identity(ui, snapshot.identity);
            apply_app_info(ui, snapshot.app_info);
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
        AppEvent::MessageAppended { message, .. } => {
            append_message(ui, message);
        }
        AppEvent::GroupCandidatesUpdated(candidates) => {
            apply_group_candidates(ui, candidates);
        }
        AppEvent::StatusNotice(msg) => {
            println!("[UI FEEDBACK] {}", msg);
        }
        AppEvent::UserError(err) => {
            println!("[UI ERROR] {}", err);
        }
        _ => {}
    }
}

fn apply_identity(ui: &AppWindow, identity: IdentityView) {
    ui.set_display_name(identity.display_name.into());
    ui.set_initials(identity.initials.into());
    ui.set_endpoint_ticket(identity.endpoint_ticket.into());
    ui.set_has_identity(identity.has_identity);
    ui.set_is_locked(identity.is_locked);
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

fn apply_chats(ui: &AppWindow, chats: Vec<ChatListItem>) {
    let rows: Vec<ChatPreview> = chats.iter().map(|c| ChatPreview {
        id: c.conversation_id.clone().into(),
        name: c.title.clone().into(),
        initials: c.initials.clone().into(),
        last_message: c.last_message.clone().into(),
        timestamp: c.timestamp.clone().into(),
        unread: c.unread_count,
        is_group: matches!(c.kind, crate::contract::ConversationKind::Group),
        is_online: c.is_online,
        is_relay: c.is_relay,
        is_verified: c.is_verified,
        last_message_status: c.last_message_status.to_string().into(),
        is_last_message_outgoing: c.is_last_message_outgoing,
    }).collect();
    ui.set_chats(VecModel::from_slice(&rows).into());

    // Populate Group List for Contacts Screen
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
        is_online: c.is_online,
        is_session_ready: c.is_session_ready,
        is_relay: c.is_relay,
        is_verified: c.is_verified,
    }).collect();
    ui.set_contacts(VecModel::from_slice(&rows).into());
}

fn apply_conversation(ui: &AppWindow, convo: ConversationView) {
    let mut ctx = ui.get_active_conversation();
    ctx.id = convo.conversation_id.clone().into();
    ctx.kind = convo.kind.to_string().into();
    ctx.title = convo.title.clone().into();
    ctx.initials = convo.initials.clone().into();
    ctx.is_online = convo.is_online;
    ctx.is_relay = convo.is_relay;
    ctx.is_verified = convo.is_verified;
    ctx.is_session_ready = convo.is_session_ready;
    ctx.connection_stage = convo.connection_stage.into();
    ui.set_active_conversation(ctx);
    
    // Also update active-group if it's a group
    if matches!(convo.kind, crate::contract::ConversationKind::Group) {
         ui.set_active_group(GroupData {
             id: convo.conversation_id.into(),
             name: convo.title.into(),
             initials: convo.initials.into(),
             topic: convo.ticket.into(), // Using ticket as topic id for group
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
        // If it's not a VecModel (e.g. empty array property), we need to replace it
        let mut rows: Vec<MessageData> = messages.iter().collect();
        rows.push(map_message(message));
        ui.set_active_messages(VecModel::from_slice(&rows).into());
    }
}

fn apply_group_candidates(ui: &AppWindow, candidates: Vec<RustCandidate>) {
    let rows: Vec<GroupCandidateItem> = candidates.into_iter().map(|c| GroupCandidateItem {
        contact_id: c.contact_id.into(),
        display_name: c.display_name.into(),
        initials: c.initials.into(),
        is_selected: c.is_selected,
        is_online: c.is_online,
    }).collect();
    ui.set_group_candidates(VecModel::from_slice(&rows).into());
}

fn map_message(m: MessageItem) -> MessageData {
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
        invite_key: m.invite_key.into(),
        is_invite_joined: m.invite_is_joined,
        status: m.status.to_string().into(),
    }
}
