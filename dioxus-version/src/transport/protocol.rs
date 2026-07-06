use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeFrame {
    Hello {
        public_key: Vec<u8>,
        display_name: String,
    },
    Ack {
        public_key: Vec<u8>,
        display_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageFrame {
    Direct {
        message_id: String,
        sender_id: String,
        ciphertext: Vec<u8>,
        timestamp: i64,
    },
    Group {
        group_id: String,
        message_id: String,
        sender_id: String,
        ciphertext: Vec<u8>,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncFrame {
    Request {
        since: i64,
    },
    Reply {
        messages: Vec<MessageFrame>,
    },
}
