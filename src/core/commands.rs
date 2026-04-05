//! Channel boundary types — the ONLY messages that cross the UI/backend boundary.
//!
//! `Command` flows UI → `NodeChatWorker` via `mpsc::Sender<Command>`.
//! `AppEvent` flows `NodeChatWorker` → Slint via `slint::invoke_from_event_loop`.
//!
//! NodeIds and TopicIds are plain `String` here; iroh types stay inside `src/p2p/`.
//! See ARCHITECTURE.md §3.5 and RULES.md A-01.

use std::path::PathBuf;
use uuid::Uuid;

/// A chat row as shown in the home list.
///
/// This crosses the backend/UI boundary so the Slint layer can render
/// the list without reading SQLite directly.
#[derive(Debug, Clone)]
pub struct ChatPreviewData {
    /// Hex-encoded peer node ID or group topic ID.
    pub id: String,
    /// Display name shown in the list.
    pub name: String,
    /// Short initials for the avatar.
    pub initials: String,
    /// Preview text for the latest message or queue state.
    pub last_message: String,
    /// Timestamp label shown in the row.
    pub timestamp: String,
    /// Unread badge count.
    pub unread: i32,
    /// `true` if this row represents a group chat.
    pub is_group: bool,
    /// `true` if the peer is currently reachable.
    pub is_online: bool,
    /// `true` if the peer is currently using relay routing.
    pub is_relay: bool,
    /// `true` if the row has queued outbound messages.
    pub is_queued: bool,
    /// `true` if the peer has been key-verified.
    pub is_verified: bool,
}

/// A direct-chat message row rendered in the active conversation view.
#[derive(Debug, Clone)]
pub struct ChatMessageData {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub is_mine: bool,
    pub status: String,
    pub is_ephemeral: bool,
    pub ttl_seconds: i32,
    pub is_group_invite: bool,
    pub invite_group_name: String,
    pub invite_topic_id: String,
    pub invite_key: String,
    pub invite_is_joined: bool,
}

/// A group-chat message row rendered in the active conversation view.
#[derive(Debug, Clone)]
pub struct GroupMessageData {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub is_mine: bool,
    pub sender_name: String,
    pub status: String,
}

/// A contact row rendered in the contacts directory page.
#[derive(Debug, Clone)]
pub struct ContactDirectoryData {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub node_id: String,
    pub is_online: bool,
    pub is_relay: bool,
    pub is_verified: bool,
}

/// A selectable peer row rendered in the group member picker.
#[derive(Debug, Clone)]
pub struct GroupSelectionData {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub is_selected: bool,
    pub is_online: bool,
}

/// Delivery status of a single message.
///
/// Advances forward only: `Queued → Sent → Delivered → Read`.
/// Backward transitions are rejected in `src/storage/queries.rs` (RULES.md DB-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageStatus {
    /// Stored locally; recipient was unreachable at send time.
    Queued,
    /// Dispatched from this device; no delivery confirmation yet.
    Sent,
    /// Confirmed received by the recipient node.
    Delivered,
    /// Recipient has opened the conversation and read the message.
    Read,
}

impl MessageStatus {
    /// Returns `true` if transitioning from `self` to `next` is a valid forward move.
    pub fn can_advance_to(&self, next: &MessageStatus) -> bool {
        matches!(
            (self, next),
            (MessageStatus::Queued, MessageStatus::Sent)
                | (MessageStatus::Sent, MessageStatus::Delivered)
                | (MessageStatus::Delivered, MessageStatus::Read)
        )
    }

    /// Serialise to the string value stored in SQLite.
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Queued    => "queued",
            MessageStatus::Sent      => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read      => "read",
        }
    }

    /// Deserialise from the SQLite string value.
    ///
    /// # Errors
    /// Returns an error for any unrecognised status string.
    pub fn from_db_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "queued"    => Ok(MessageStatus::Queued),
            "sent"      => Ok(MessageStatus::Sent),
            "delivered" => Ok(MessageStatus::Delivered),
            "read"      => Ok(MessageStatus::Read),
            other       => anyhow::bail!("unknown message status in database: {:?}", other),
        }
    }
}

/// Commands sent FROM the Slint UI TO `NodeChatWorker` via `mpsc::Sender<Command>`.
///
/// Each variant has exactly one handler in `NodeChatWorker::handle_command` (RULES.md A-04).
#[derive(Debug)]
pub enum Command {
    /// Encrypt and send a direct 1:1 message to a peer.
    SendDirectMessage {
        /// Hex-encoded NodeId of the intended recipient.
        target: String,
        /// Unencrypted message body — the backend encrypts before transmitting.
        plaintext: String,
    },

    /// Send a file to a peer via direct P2P transfer.
    SendFile {
        /// Hex-encoded NodeId of the intended recipient.
        target: String,
        /// Absolute path to the source file on disk.
        file_path: PathBuf,
    },

    /// Inform the backend that the local user has read messages in a conversation.
    NotifyReadReceipt {
        /// Hex-encoded NodeId of the peer whose messages were read.
        target: String,
        /// ID of the most recent message the user has seen.
        message_id: Uuid,
    },

    /// Create a new group on this device; this node is the group founder.
    CreateGroup {
        /// Human-readable group name shown in the group header.
        name: String,
    },

    /// Accept a group invitation and store the group locally.
    AcceptGroupInvite {
        /// Hex-encoded TopicId of the group.
        topic: String,
        /// Human-readable group name.
        group_name: String,
        /// Hex-encoded symmetric key.
        symmetric_key: String,
    },

    /// Toggle whether a peer is selected for a new group.
    ToggleGroupMemberSelection {
        /// Hex-encoded NodeId of the peer.
        peer_id: String,
    },

    /// Ensure identity creation flow is triggered.
    /// Create a new local identity with the given display name and pin.
    CreateIdentity {
        name: String,
        pin: String,
    },

    /// Finalise the setup overlay and go to chat list
    FinaliseIdentity,

    /// Encrypt and broadcast a message to a group gossip swarm.
    SendGroupMessage {
        /// Hex-encoded TopicId of the target group.
        topic: String,
        /// Unencrypted message body — the backend encrypts before broadcasting.
        plaintext: String,
    },

    /// Invite a peer to a group by sending them the topic + symmetric key via 1:1 E2EE.
    InviteToGroup {
        /// Hex-encoded NodeId of the peer to invite.
        target: String,
        /// Hex-encoded TopicId of the group.
        topic: String,
    },

    /// Mark a contact as key-verified after the user completes the safety number check.
    MarkVerified {
        /// Hex-encoded NodeId of the peer to mark as verified.
        node_id: String,
    },
    /// Add a new contact by Iroh EndpointTicket or discovery-capable NodeId.
    AddContact {
        /// The Iroh ticket string (base32) or raw NodeId.
        ticket_or_id: String,
    },
    /// Clear all messages from the local database.
    ClearMessages {
        /// The vault PIN for verification.
        pin: String,
    },
    /// Clear only the messages in a single conversation.
    ClearConversationHistory {
        /// Hex-encoded NodeId or TopicId of the conversation to clear.
        target: String,
        /// `true` if this is a group conversation.
        is_group: bool,
        /// The vault PIN for verification.
        pin: String,
    },
    /// Delete a single conversation and its local history.
    DeleteConversation {
        /// Hex-encoded NodeId or TopicId of the conversation to remove.
        target: String,
        /// `true` if this is a group conversation.
        is_group: bool,
        /// The vault PIN for verification.
        pin: String,
    },
    /// Retry queued direct messages for a peer immediately.
    RetryQueuedMessages {
        /// Hex-encoded NodeId of the direct peer.
        target: String,
    },
    /// Delete the local identity and all associated data.
    DeleteIdentity {
        /// The vault PIN for verification.
        pin: String,
    },
    /// Wipes everything without PIN verification. EMERGENCY ONLY.
    ForceDeleteIdentity,
    /// Unlock the app shell after a returning-user gate is dismissed.
    UnlockApp {
        /// The characters/digits entered by the user.
        pin: String,
    },

    /// Change the stored access PIN after verifying the current one.
    ChangePassword {
        /// The currently-stored PIN to verify identity.
        current_pin: String,
        /// The new PIN to hash and persist.
        new_pin: String,
    },

    /// Notify the backend that the app moved to foreground/background.
    SetAppForeground {
        /// `true` when the app is visible and active to the user.
        foreground: bool,
    },

    /// Push current endpoint/share info to the UI after startup wiring completes.
    RefreshLocalInfo,

    /// Load the selected conversation thread into the active chat pane.
    LoadConversation {
        /// Hex-encoded NodeId or TopicId.
        target: String,
        /// `true` if this is a group conversation.
        is_group: bool,
    },
    /// Update the local display name in the identity vault.
    UpdateDisplayName {
        /// New human-readable name.
        name: String,
    },
}

/// Events pushed FROM `NodeChatWorker` TO the Slint UI via `slint::invoke_from_event_loop`.
///
/// Every variant MUST be handled in `src/ui/models::apply_event` — no `_ => {}` catch-alls
/// that silently drop events (RULES.md R-03).
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A new decrypted 1:1 direct message is ready to display.
    IncomingMessage {
        /// Hex-encoded NodeId of the sender.
        sender: String,
        /// Unique message identifier (used for status tracking).
        id: Uuid,
        /// Decrypted plaintext content.
        plaintext: String,
        /// UTC Unix timestamp in seconds.
        timestamp: i64,
    },

    /// A new decrypted group message is ready to display.
    IncomingGroupMessage {
        /// Hex-encoded TopicId of the originating group.
        topic: String,
        /// Hex-encoded NodeId of the sender within the group.
        sender: String,
        /// Unique message identifier.
        id: Uuid,
        /// Decrypted plaintext content.
        plaintext: String,
        /// UTC Unix timestamp in seconds.
        timestamp: i64,
    },

    /// A file has been received from a peer and saved locally.
    IncomingFile {
        /// Hex-encoded NodeId of the sender.
        sender: String,
        /// Original filename as provided by the sender.
        file_name: String,
        /// Local path where the file was saved.
        path: PathBuf,
    },

    /// The delivery status of an outgoing message has changed.
    MessageStatusUpdate {
        /// The message whose status changed.
        id: Uuid,
        /// The conversation this message belongs to.
        target: String,
        /// `true` if the message belongs to a group conversation.
        is_group: bool,
        /// The new delivery status.
        status: MessageStatus,
    },

    /// A peer has sent a group invitation via the 1:1 E2EE channel.
    GroupInviteReceived {
        /// Hex-encoded TopicId of the offered group.
        topic: String,
        /// Group name as supplied by the inviter.
        group_name: String,
    },

    /// A peer's network reachability status has changed.
    PeerOnlineStatus {
        /// Hex-encoded NodeId of the peer.
        peer: String,
        /// `true` if the peer is currently reachable.
        online: bool,
        /// `true` if the connection is routed through a DERP relay (reduced privacy).
        via_relay: bool,
    },

    /// Direct-chat handshake progress for the currently active peer.
    PeerHandshakeStage {
        /// Hex-encoded NodeId of the peer.
        peer: String,
        /// Human-readable progress label.
        stage: String,
    },

    /// Current global network health and peer counts.
    NetworkStatus {
        /// Number of peers reached via direct P2P (QUIC).
        direct_peers: i32,
        /// Number of peers reached via DERP relay.
        relay_peers: i32,
        /// `true` if this node has no connection to any relay/peer.
        is_offline: bool,
    },

    /// Setup is complete, UI should hide onboarding overlay.
    SetupComplete,

    /// Identity generated, move UI to step 2.
    IdentityGenerated {
        display_name: String,
        node_id: String,
    },

    /// Shareable endpoint ticket for the local identity / discovery flow.
    EndpointTicketUpdated {
        /// Ticket string created from the local endpoint address.
        ticket: String,
    },

    /// All local messages have been cleared.
    MessagesCleared,

    /// Clear messages failed (e.g. wrong PIN).
    ClearMessagesFailed { error: String },

    /// The local identity and all data have been wiped.
    IdentityDeleted,

    /// Reset failed (e.g. wrong PIN).
    DeleteIdentityFailed { error: String },

    /// A single conversation's messages have been cleared.
    ConversationCleared {
        /// Hex-encoded NodeId or TopicId that was cleared.
        target: String,
        /// `true` if the cleared conversation was a group.
        is_group: bool,
    },

    /// A single conversation has been removed from the local database.
    ConversationDeleted {
        /// Hex-encoded NodeId or TopicId that was removed.
        target: String,
        /// `true` if the removed conversation was a group.
        is_group: bool,
    },

    /// Contact details for the currently selected peer.
    PeerContactDetails {
        /// Hex-encoded NodeId of the peer.
        peer: String,
        /// Shared display name learned from the peer handshake.
        display_name: String,
        /// Imported ticket or dial hint used to reach the peer.
        endpoint_ticket: String,
        /// `true` if the peer is key-verified.
        verified: bool,
    },

    /// The backend explicitly finalized an application unlock event.
    UnlockComplete,

    /// The application was locked (e.g. on background or startup).
    AppLocked,

    /// The application unlock event failed.
    UnlockFailed {
        /// Error message describing why unlock failed.
        error: String,
    },

    /// A peer was successfully key-verified.
    PeerVerified {
        /// Hex-encoded NodeId of the verified peer.
        peer: String,
    },

    /// Password was updated successfully.
    PasswordChanged,

    /// Password change failed (wrong current PIN or other error).
    PasswordChangeFailed {
        error: String,
    },

    /// The home chat list has changed and should be re-rendered.
    ChatsUpdated {
        /// Rows to display in the chat list.
        chats: Vec<ChatPreviewData>,
    },

    /// The contacts directory has changed and should be re-rendered.
    ContactsUpdated {
        /// Rows to display in the contacts directory.
        contacts: Vec<ContactDirectoryData>,
    },

    /// The group member picker has changed and should be re-rendered.
    GroupSelectionUpdated {
        /// Rows to display in the picker.
        contacts: Vec<GroupSelectionData>,
        /// Number of currently selected peers.
        selected_count: i32,
    },

    /// The active direct conversation should be re-rendered.
    DirectConversationLoaded {
        /// Hex-encoded peer NodeId.
        target: String,
        /// Rows to display in the active thread.
        messages: Vec<ChatMessageData>,
    },

    /// The active group conversation should be re-rendered.
    GroupConversationLoaded {
        /// Hex-encoded TopicId.
        topic: String,
        /// Rows to display in the active thread.
        messages: Vec<GroupMessageData>,
    },

    /// A non-fatal backend error to surface to the user in plain English (RULES.md E-03).
    Error {
        /// Human-readable, user-facing description of what went wrong.
        message: String,
    },
}
