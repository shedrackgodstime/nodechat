//! nodechat binary protocol (NC-P2P) — Version 1.0 (Academic Build)
//! -------------------------------------------------------------------
//! This module defines the binary wire format for the NodeChat network.
//! It uses a custom TLV-style (Type-Length-Value) framing to ensure 
//! compatibility and strict validation of incoming P2P signals.

use anyhow::{Result, bail};

/// Magic bytes [0x4E, 0x43, 0x31, 0x48] -> "NC1H"
pub const MAGIC: [u8; 4] = [0x4E, 0x43, 0x31, 0x48];

/// Current Protocol Version
pub const VERSION: u8 = 0x01;

/// Handshake Frame Types
pub const HELLO: u8     = 0x01;
pub const HELLO_ACK: u8 = 0x02;

#[derive(Debug, Clone)]
pub struct HandshakeFrame {
    pub kind: u8,
    pub x25519_public: [u8; 32],
    pub ticket: String,
    pub display_name: String,
}

impl HandshakeFrame {
    /// Serialize frame to bytes for transmission over Iroh.
    pub fn encode(&self) -> Vec<u8> {
        let ticket_bytes = self.ticket.as_bytes();
        let name_bytes = self.display_name.as_bytes();
        
        // Size = Magic(4) + Type(1) + Pubkey(32) + TicketLen(2) + Ticket(N) + NameLen(2) + Name(M)
        let mut buf = Vec::with_capacity(4 + 1 + 32 + 2 + ticket_bytes.len() + 2 + name_bytes.len());
        
        buf.extend_from_slice(&MAGIC);
        buf.push(self.kind);
        buf.extend_from_slice(&self.x25519_public);
        
        buf.extend_from_slice(&(ticket_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(ticket_bytes);
        
        buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(name_bytes);
        
        buf
    }

    /// Deserialize frame from raw Iroh bytes.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 1 + 32 + 2 + 2 {
            bail!("handshake frame too short");
        }
        if data[0..4] != MAGIC {
            bail!("invalid magic header");
        }
        
        let kind = data[4];
        let mut x25519_public = [0u8; 32];
        x25519_public.copy_from_slice(&data[5..37]);
        
        let mut cursor = 37;
        
        let ticket_len = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
        cursor += 2;
        if data.len() < cursor + ticket_len + 2 { bail!("malformed ticket field"); }
        let ticket = String::from_utf8(data[cursor..cursor+ticket_len].to_vec())?;
        cursor += ticket_len;
        
        let name_len = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
        cursor += 2;
        if data.len() < cursor + name_len { bail!("malformed name field"); }
        let display_name = String::from_utf8(data[cursor..cursor+name_len].to_vec())?;
        
        Ok(Self { kind, x25519_public, ticket, display_name })
    }
}

/// Direct Message Frame (NC2D)
/// ---------------------------------------------------------
/// Framing for messages sent over established secure streams.
/// Magic(4) + Kind(1) + MessageId(16) + PayloadLen(4) + Payload(N)

pub const DIRECT_MAGIC: [u8; 4] = [0x4E, 0x43, 0x32, 0x44]; // NC2D
pub const KIND_TEXT: u8    = 0x01;
pub const KIND_RECEIPT: u8 = 0x02;

#[derive(Debug, Clone)]
pub enum DirectFrame {
    Text {
        id: uuid::Uuid,
        content: Vec<u8>,
    },
    Receipt {
        id: uuid::Uuid,
        is_read: bool,
    },
}

impl DirectFrame {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&DIRECT_MAGIC);
        
        match self {
            DirectFrame::Text { id, content } => {
                buf.push(KIND_TEXT);
                buf.extend_from_slice(id.as_bytes());
                buf.extend_from_slice(&(content.len() as u32).to_be_bytes());
                buf.extend_from_slice(content);
            }
            DirectFrame::Receipt { id, is_read } => {
                buf.push(KIND_RECEIPT);
                buf.extend_from_slice(id.as_bytes());
                buf.push(if *is_read { 1 } else { 0 });
            }
        }
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 6 { bail!("direct frame too short"); }
        if data[0..4] != DIRECT_MAGIC { bail!("invalid direct magic"); }
        
        let kind = data[4];
        let mut id_bytes = [0u8; 16];
        if data.len() < 21 { bail!("direct frame missing ID"); }
        id_bytes.copy_from_slice(&data[5..21]);
        let id = uuid::Uuid::from_bytes(id_bytes);
        
        match kind {
            KIND_TEXT => {
                if data.len() < 25 { bail!("direct frame missing length"); }
                let len = u32::from_be_bytes([data[21], data[22], data[23], data[24]]) as usize;
                if data.len() < 25 + len { bail!("truncated direct payload"); }
                Ok(DirectFrame::Text {
                    id,
                    content: data[25..25+len].to_vec(),
                })
            }
            KIND_RECEIPT => {
                let is_read = if data.len() > 21 { data[21] == 1 } else { false };
                Ok(DirectFrame::Receipt { id, is_read })
            }
            _ => bail!("unknown direct kind: {}", kind),
        }
    }
}
/// Group Message Frame (NC3G)
/// ---------------------------------------------------------
/// Framing for messages broadcast over Iroh Gossip.
/// Magic(4) + SenderNodeId(32) + PayloadLen(4) + Payload(N)

pub const GROUP_MAGIC: [u8; 4] = [0x4E, 0x43, 0x33, 0x47]; // NC3G

#[derive(Debug, Clone)]
pub struct GroupFrame {
    pub id: uuid::Uuid,
    pub sender_id: String, // hex
    pub timestamp: i64,
    pub content: Vec<u8>,
}

impl GroupFrame {
    pub fn encode(&self) -> Vec<u8> {
        let sender_bytes = hex::decode(&self.sender_id).unwrap_or_else(|_| vec![0u8; 32]);
        // Size = Magic(4) + Id(16) + Timestamp(8) + SenderNodeId(32) + PayloadLen(4) + Payload(N)
        let mut buf = Vec::with_capacity(4 + 16 + 8 + 32 + 4 + self.content.len());
        buf.extend_from_slice(&GROUP_MAGIC);
        buf.extend_from_slice(self.id.as_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&sender_bytes);
        buf.extend_from_slice(&(self.content.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.content);
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 16 + 8 + 32 + 4 { bail!("group frame too short"); }
        if data[0..4] != GROUP_MAGIC { bail!("invalid group magic"); }
        
        let id = uuid::Uuid::from_slice(&data[4..20])?;
        let timestamp = i64::from_be_bytes([data[20], data[21], data[22], data[23], data[24], data[25], data[26], data[27]]);
        let sender_id = hex::encode(&data[28..60]);
        let len = u32::from_be_bytes([data[60], data[61], data[62], data[63]]) as usize;
        if data.len() < 64 + len { bail!("truncated group payload"); }
        let content = data[64..64+len].to_vec();
        
        Ok(Self { id, sender_id, timestamp, content })
    }
}

/// Sync Frame (NC4S)
/// ---------------------------------------------------------
/// Used for reconciling missing group history when coming online.
/// Magic(4) + Kind(1) + TopicLen(2) + Topic(N) + Data(...)
pub const SYNC_MAGIC: [u8; 4] = [0x4E, 0x43, 0x34, 0x53]; // NC4S
pub const SYNC_QUERY: u8 = 0x01;
pub const SYNC_REPLY: u8 = 0x02;

#[derive(Debug, Clone)]
pub enum SyncFrame {
    Query {
        topic: String,
        after_timestamp: i64,
    },
    Reply {
        topic: String,
        messages: Vec<GroupFrame>,
    }
}

impl SyncFrame {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&SYNC_MAGIC);
        match self {
            SyncFrame::Query { topic, after_timestamp } => {
                buf.push(SYNC_QUERY);
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf.extend_from_slice(&after_timestamp.to_be_bytes());
            }
            SyncFrame::Reply { topic, messages } => {
                buf.push(SYNC_REPLY);
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf.extend_from_slice(&(messages.len() as u16).to_be_bytes());
                for msg in messages {
                    let encoded = msg.encode();
                    buf.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
                    buf.extend_from_slice(&encoded);
                }
            }
        }
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 7 { bail!("sync frame too short"); }
        if data[0..4] != SYNC_MAGIC { bail!("invalid sync magic"); }
        let kind = data[4];
        let topic_len = u16::from_be_bytes([data[5], data[6]]) as usize;
        if data.len() < 7 + topic_len { bail!("truncated sync topic"); }
        let topic = String::from_utf8(data[7..7+topic_len].to_vec())?;
        let mut cursor = 7 + topic_len;

        match kind {
            SYNC_QUERY => {
                if data.len() < cursor + 8 { bail!("truncated sync query"); }
                let after_timestamp = i64::from_be_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3], data[cursor+4], data[cursor+5], data[cursor+6], data[cursor+7]]);
                Ok(SyncFrame::Query { topic, after_timestamp })
            }
            SYNC_REPLY => {
                if data.len() < cursor + 2 { bail!("truncated sync reply count"); }
                let count = u16::from_be_bytes([data[cursor], data[cursor+1]]) as usize;
                cursor += 2;
                let mut messages = Vec::with_capacity(count);
                for _ in 0..count {
                    if data.len() < cursor + 4 { bail!("truncated sync message len"); }
                    let msg_len = u32::from_be_bytes([data[cursor], data[cursor+1], data[cursor+2], data[cursor+3]]) as usize;
                    cursor += 4;
                    if data.len() < cursor + msg_len { bail!("truncated sync message body"); }
                    let msg = GroupFrame::decode(&data[cursor..cursor+msg_len])?;
                    messages.push(msg);
                    cursor += msg_len;
                }
                Ok(SyncFrame::Reply { topic, messages })
            }
            _ => bail!("unknown sync kind: {}", kind),
        }
    }
}
