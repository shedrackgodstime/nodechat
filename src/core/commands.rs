//! Channel boundary types — the ONLY messages that cross the UI/backend boundary.
//!
//! `Command` flows UI → `NodeChatWorker` via `mpsc::Sender<Command>`.
//! `AppEvent` flows `NodeChatWorker` → Slint via `slint::invoke_from_event_loop`.
//!
//! NodeIds and TopicIds are plain `String` here; iroh types stay inside `src/p2p/`.
//! See ARCHITECTURE.md §3.5 and RULES.md A-01.

use std::path::PathBuf;
use uuid::Uuid;

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

    /// Ensure identity creation flow is triggered.
    CreateIdentity {
        /// The chosen display name
        name: String,
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
    /// Clear all messages from the local database.
    ClearMessages,
    /// Delete the local identity and all associated data.
    DeleteIdentity,
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

    /// Setup is complete, UI should hide onboarding overlay.
    SetupComplete,

    /// Identity generated, move UI to step 2.
    IdentityGenerated {
        display_name: String,
        node_id: String,
    },

    /// All local messages have been cleared.
    MessagesCleared,

    /// A non-fatal backend error to surface to the user in plain English (RULES.md E-03).
    Error {
        /// Human-readable, user-facing description of what went wrong.
        message: String,
    },
}
