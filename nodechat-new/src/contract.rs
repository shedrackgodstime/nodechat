use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Queued,
    Sent,
    Delivered,
    Read,
}

impl MessageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MessageStatus::Queued => "queued",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
        }
    }
}

impl fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationKind {
    Direct,
    Group,
}

impl ConversationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ConversationKind::Direct => "direct",
            ConversationKind::Group => "group",
        }
    }
}

impl fmt::Display for ConversationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPreview {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread: i32,
    pub is_group: bool,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_queued: bool,
    pub is_verified: bool,
    pub is_session_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub is_mine: bool,
    pub status: MessageStatus,
    pub is_ephemeral: bool,
    pub ttl_seconds: i32,
    pub is_group_invite: bool,
    pub invite_group_name: String,
    pub invite_topic_id: String,
    pub invite_key: String,
    pub invite_is_joined: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMessage {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub is_mine: bool,
    pub sender_name: String,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactDirectoryEntry {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub node_id: String,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_verified: bool,
    pub is_session_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSelectionEntry {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub is_selected: bool,
    pub is_online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationState {
    pub kind: ConversationKind,
    pub id: String,
    pub title: String,
    pub initials: String,
    pub ticket: String,
    pub is_online: bool,
    pub is_session_ready: bool,
    pub is_verified: bool,
    pub connection_stage: String,
    pub member_count: String,
    pub return_screen: i32,
}

impl ConversationState {
    pub fn empty(kind: ConversationKind) -> Self {
        Self {
            kind,
            id: String::new(),
            title: String::new(),
            initials: String::new(),
            ticket: String::new(),
            is_online: false,
            is_session_ready: false,
            is_verified: false,
            connection_stage: String::new(),
            member_count: "0".to_string(),
            return_screen: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityCard {
    pub display_name: String,
    pub initials: String,
    pub node_id: String,
    pub endpoint_ticket: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub identity: Option<IdentityCard>,
    pub chats: Vec<ChatPreview>,
    pub contacts: Vec<ContactDirectoryEntry>,
    pub group_candidates: Vec<GroupSelectionEntry>,
    pub active_conversation: ConversationState,
    pub direct_messages: Vec<ChatMessage>,
    pub group_messages: Vec<GroupMessage>,
    pub debug_logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Refresh,
    LoadConversation {
        target: String,
        is_group: bool,
    },
    SendDirectMessage {
        target: String,
        plaintext: String,
    },
    SendGroupMessage {
        topic: String,
        plaintext: String,
    },
    ToggleVerified {
        node_id: String,
        verified: bool,
    },
    ToggleGroupMemberSelection {
        peer_id: String,
    },
    AddContact {
        ticket_or_id: String,
    },
    CreateGroup {
        name: String,
    },
    CreateIdentity {
        name: String,
        pin: String,
    },
    FinaliseIdentity,
    UnlockApp {
        pin: String,
    },
    ChangePassword {
        current_pin: String,
        new_pin: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    SnapshotReady(AppSnapshot),
    ChatsUpdated(Vec<ChatPreview>),
    ContactsUpdated(Vec<ContactDirectoryEntry>),
    ConversationLoaded(ConversationState),
    DirectMessageAppended {
        conversation_id: String,
        message: ChatMessage,
    },
    GroupMessageAppended {
        topic_id: String,
        message: GroupMessage,
    },
    IdentityUpdated(IdentityCard),
    Status(String),
    Error(String),
}
