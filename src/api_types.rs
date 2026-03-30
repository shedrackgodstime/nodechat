//! NodeChat Types - Shared data structures for UI integration
//!
//! These types are designed to be framework-agnostic and can be used
//! with egui, Dioxus, or any other UI framework.

use serde::{Deserialize, Serialize};

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: Vec<u8>,
    pub timestamp: i64,
    pub status: MessageStatus,
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Failed,
}

/// A peer/contact in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub node_id: String,
    pub public_key: Vec<u8>,
    pub alias: Option<String>,
    pub last_seen: i64,
}

/// Local user identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub node_id: String,
    pub display_name: String,
    pub ticket: String,
}

/// Connection ticket for adding peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTicket {
    pub node_id: String,
    pub relay_urls: Vec<String>,
}
