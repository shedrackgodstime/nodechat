use dioxus::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub text: String,
    pub timestamp: String,
    pub is_outgoing: bool,
    pub is_system: bool,
    pub is_invite: bool,
    pub invite_group_name: String,
    pub invite_desc: String,
    pub invite_topic_id: String,
    pub invite_key: String,
    pub invite_joined: bool,
    pub status: String, // "queued", "sent", "delivered", "read"
    pub is_contact_share: bool,
    pub share_name: String,
    pub share_node_id: String,
    pub share_ticket: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread: i32,
    pub is_group: bool,
    pub is_online: bool,
    pub is_verified: bool,
    pub has_queued: bool,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub node_id: String,
    pub is_online: bool,
    pub is_verified: bool,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub has_identity: Signal<bool>,
    pub onboarding_step: Signal<i32>,
    pub display_name_input: Signal<String>,
    pub temp_name_input: Signal<String>,
    pub settings_name_draft: Signal<String>,
    pub show_copied_toast: Signal<bool>,
    pub show_clear_confirm: Signal<bool>,
    pub show_reset_confirm: Signal<bool>,
    pub info_clear_confirm: Signal<bool>,
    pub info_remove_confirm: Signal<bool>,
    pub is_locked: Signal<bool>,
    pub pin_input: Signal<String>,
    pub pin_error: Signal<bool>,
    pub current_tab: Signal<String>,
    pub previous_tab: Signal<String>,
    pub active_chat_id: Signal<Option<String>>,
    pub search_query: Signal<String>,
    pub show_create_group: Signal<bool>,
    pub show_add_contact: Signal<bool>,
    pub show_info_panel: Signal<bool>,
    pub logs: Signal<Vec<String>>,
    pub message_input: Signal<String>,
    pub contact_ticket_input: Signal<String>,
    pub group_name_input: Signal<String>,
    pub group_desc_input: Signal<String>,
    pub join_ticket_input: Signal<String>,
    pub selected_group_contacts: Signal<Vec<String>>,
    pub mobile_show_chat: Signal<bool>,
    pub chats: Signal<Vec<Chat>>,
    pub contacts: Signal<Vec<Contact>>,
}
