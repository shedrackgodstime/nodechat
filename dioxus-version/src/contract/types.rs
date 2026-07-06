#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentityView {
    pub id: String,
    pub display_name: String,
    pub public_key: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactView {
    pub id: String,
    pub display_name: String,
    pub public_key: String,
    pub ticket: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageView {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub ciphertext: String,
    pub timestamp: i64,
    pub status: String,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageKind {
    Direct,
    Group,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupView {
    pub id: String,
    pub name: String,
    pub topic_id: String,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatPreview {
    pub id: String,
    pub name: String,
    pub last_message: String,
    pub timestamp: i64,
    pub unread: i32,
    pub is_group: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppSnapshot {
    pub identity: Option<IdentityView>,
    pub contacts: Vec<ContactView>,
    pub groups: Vec<GroupView>,
    pub chats: Vec<ChatPreview>,
}
