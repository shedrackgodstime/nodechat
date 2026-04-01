use slint::VecModel;
use slint::Model;

pub struct ChatPreview {
    pub id: String,
    pub name: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread: i32,
    pub is_group: bool,
    pub is_online: bool,
    pub is_queued: bool,
    pub is_verified: bool,
}

pub struct Message {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub is_mine: bool,
    pub status: String,
    pub is_ephemeral: bool,
    pub ttl_seconds: i32,
}

pub struct Contact {
    pub id: String,
    pub name: String,
    pub node_id: String,
    pub is_verified: bool,
    pub is_online: bool,
}