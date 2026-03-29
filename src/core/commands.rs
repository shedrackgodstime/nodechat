use std::path::PathBuf;
use uuid::Uuid;

/// Temporary simplified NodeId for zero-error slate
pub type NodeId = String;

/// Unique identifier for a Gossip Swarm (Group Chat)
pub type TopicId = [u8; 32];

#[derive(Debug, Clone)]
pub enum MessageStatus {
    Queued,
    Sent,
    Delivered,
    Read,
    Failed(String),
}

/// Commands sent FROM the UI TO the Backend Worker.
#[derive(Debug, Clone)]
pub enum Command {
    SendDirectMessage { target: NodeId, plaintext: String },
    SendFile { target: NodeId, file_path: PathBuf },
    NotifyReadReceipt { target: NodeId, message_id: Uuid },
    CreateGroup { name: String },
    SendGroupMessage { topic: TopicId, plaintext: String },
    InviteToGroup { target: NodeId, topic: TopicId },
    Quit, // Clean shutdown signal
}

/// Events broadcast FROM the Backend Worker TO the UI.
#[derive(Debug, Clone)]
pub enum AppEvent {
    IncomingMessage { sender: NodeId, plaintext: String },
    IncomingFile { sender: NodeId, file_name: String, path: PathBuf },
    MessageStatusUpdate { id: Uuid, status: MessageStatus },
    IncomingGroupMessage { topic: TopicId, sender: NodeId, plaintext: String },
    GroupInviteReceived { topic: TopicId, group_name: String },
    BackendReady, // Fired when Iroh and SQLite are fully booted
}
