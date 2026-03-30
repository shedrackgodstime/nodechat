use std::path::PathBuf;
use uuid::Uuid;

/// Hexadecimal representation of an Iroh EndpointId (PublicKey)
pub type NodeHex = String;

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
    SendDirectMessage { target: NodeHex, plaintext: String },
    SendFile { target: NodeHex, file_path: PathBuf },
    NotifyReadReceipt { target: NodeHex, message_id: Uuid },
    CreateGroup { name: String },
    SendGroupMessage { topic: TopicId, plaintext: String },
    InviteToGroup { target: NodeHex, topic: TopicId },
    UpdateProfile { display_name: String },
    RefreshIdentity,
    AddContactByTicket { ticket: String },
    Quit, // Clean shutdown signal
}

/// Events broadcast FROM the Backend Worker TO the UI.
#[derive(Debug, Clone)]
pub enum AppEvent {
    IncomingMessage { sender: NodeHex, plaintext: String },
    InternalIncomingMessage { sender: NodeHex, ciphertext: Vec<u8> },
    IncomingFile { sender: NodeHex, file_name: String, path: PathBuf },
    MessageStatusUpdate { id: Uuid, status: MessageStatus },
    IncomingGroupMessage { topic: TopicId, sender: NodeHex, plaintext: String },
    GroupInviteReceived { topic: TopicId, group_name: String },
    IdentityCreated { 
        node_id: String, 
        ticket: String,
        display_name: Option<String>,
    },
    IdentityGenerationFailed(String),
    ErrorMessage(String),
    BackendReady, // Fired when Iroh and SQLite are fully booted
}
