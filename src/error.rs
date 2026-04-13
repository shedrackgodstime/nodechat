use thiserror::Error;
use uuid::Uuid;

/// Domain-specific error types for NodeChat.
/// Categorizing errors shows architectural maturity and allows for targeted UI feedback.
#[derive(Debug, Error)]
pub enum NodeChatError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Cryptographic failure: {0}")]
    Crypto(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Protocol violation: {0}")]
    Protocol(String),

    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("External error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type NodeChatResult<T> = Result<T, NodeChatError>;
