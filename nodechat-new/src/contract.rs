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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Standard,
    System,
    GroupInvite,
    ContactShare,
}

impl MessageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MessageKind::Standard => "standard",
            MessageKind::System => "system",
            MessageKind::GroupInvite => "group_invite",
            MessageKind::ContactShare => "contact_share",
        }
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryScope {
    ActiveConversation,
    AllConversations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityView {
    pub display_name: String,
    pub initials: String,
    pub peer_id: String,
    pub endpoint_ticket: String,
    pub is_locked: bool,
    pub has_identity: bool,
}

impl IdentityView {
    pub fn empty() -> Self {
        Self {
            display_name: String::new(),
            initials: "?".to_string(),
            peer_id: String::new(),
            endpoint_ticket: String::new(),
            is_locked: true,
            has_identity: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppFlags {
    pub direct_peer_count: i32,
    pub relay_peer_count: i32,
    pub is_offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatListItem {
    pub conversation_id: String,
    pub kind: ConversationKind,
    pub title: String,
    pub initials: String,
    pub last_message: String,
    pub last_message_status: MessageStatus,
    pub is_last_message_outgoing: bool,
    pub timestamp: String,
    pub member_count: i32,
    pub unread_count: i32,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_verified: bool,
    pub is_session_ready: bool,
    pub has_queued_messages: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactListItem {
    pub contact_id: String,
    pub peer_id: String,
    pub display_name: String,
    pub initials: String,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_verified: bool,
    pub is_session_ready: bool,
    pub direct_conversation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupCandidateItem {
    pub contact_id: String,
    pub display_name: String,
    pub initials: String,
    pub is_selected: bool,
    pub is_online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationView {
    pub conversation_id: String,
    pub kind: ConversationKind,
    pub title: String,
    pub initials: String,
    pub peer_id: String,
    pub ticket: String,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_verified: bool,
    pub is_session_ready: bool,
    pub connection_stage: String,
    pub member_count: i32,
    pub return_screen: i32,
}

impl ConversationView {
    pub fn empty(kind: ConversationKind) -> Self {
        Self {
            conversation_id: String::new(),
            kind,
            title: String::new(),
            initials: String::new(),
            peer_id: String::new(),
            ticket: String::new(),
            is_online: false,
            is_relay: false,
            is_verified: false,
            is_session_ready: false,
            connection_stage: String::new(),
            member_count: 0,
            return_screen: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageItem {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_name: String,
    pub text: String,
    pub timestamp: String,
    pub is_outgoing: bool,
    pub is_system: bool,
    pub status: MessageStatus,
    pub kind: MessageKind,
    pub invite_group_name: String,
    pub invite_topic_id: String,
    pub invite_key: String,
    pub invite_is_joined: bool,
    pub is_ephemeral: bool,
    pub ttl_seconds: i32,
    pub received_timestamp: String,
    pub is_delayed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfoView {
    pub name: String,
    pub version: String,
    pub version_type: String,
    pub description: String,
    pub website: String,
    pub repo: String,
}

impl AppInfoView {
    pub fn current() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            version_type: "Beta Build".to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            website: env!("CARGO_PKG_HOMEPAGE").to_string(),
            repo: env!("CARGO_PKG_REPOSITORY").to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub identity: IdentityView,
    pub app_info: AppInfoView,
    pub app_flags: AppFlags,
    pub chat_list: Vec<ChatListItem>,
    pub contact_list: Vec<ContactListItem>,
    pub group_candidates: Vec<GroupCandidateItem>,
    pub active_conversation: ConversationView,
    pub active_messages: Vec<MessageItem>,
    pub debug_feed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Refresh,
    LoadConversation {
        conversation_id: String,
    },
    SendMessage {
        conversation_id: String,
        plaintext: String,
    },
    RetryQueuedMessage {
        conversation_id: String,
        message_id: String,
    },
    DeleteConversation {
        conversation_id: String,
        confirmation_pin: Option<String>,
    },
    AddContact {
        ticket_or_peer_id: String,
    },
    CreateGroup {
        name: String,
        description: String,
        member_contact_ids: Vec<String>,
    },
    InviteToGroup {
        group_id: String,
    },
    ToggleGroupCandidate {
        contact_id: String,
    },
    OpenDirectConversation {
        contact_id: String,
    },
    CreateIdentity {
        display_name: String,
        pin: String,
    },
    FinalizeIdentity,
    UnlockApp {
        pin: String,
    },
    ChangePassword {
        current_pin: String,
        new_pin: String,
    },
    ShareContact {
        contact_id: String,
    },
    UpdateDisplayName {
        display_name: String,
    },
    ResetIdentity {
        confirmation_pin: String,
    },
    SetVerification {
        peer_id: String,
        verified: bool,
    },
    AcceptGroupInvite {
        conversation_id: String,
        topic_id: String,
        invite_key: String,
    },
    ClearMessageHistory {
        scope: HistoryScope,
        confirmation_pin: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    SnapshotReady(AppSnapshot),
    IdentityUpdated(IdentityView),
    ChatListUpdated(Vec<ChatListItem>),
    ContactListUpdated(Vec<ContactListItem>),
    ConversationUpdated(ConversationView),
    MessageListReplaced {
        conversation_id: String,
        messages: Vec<MessageItem>,
    },
    MessageAppended {
        conversation_id: String,
        message: MessageItem,
    },
    GroupCandidatesUpdated(Vec<GroupCandidateItem>),
    DebugFeedUpdated(Vec<String>),
    /// A single log line from the tracing subscriber — level is "ERROR"/"WARN"/"INFO"/"DEBUG".
    Log { level: String, message: String },
    StatusNotice(String),
    UserError(String),
    MessageStatusChanged {
        conversation_id: String,
        message_id:      String,
        status:          MessageStatus,
    },
}
