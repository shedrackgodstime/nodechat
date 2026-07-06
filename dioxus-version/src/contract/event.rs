#![allow(dead_code)]

use super::types::*;

#[derive(Debug, Clone)]
pub enum Event {
    // Identity
    IdentityLoaded { identity: IdentityView },
    IdentityCreated { identity: IdentityView },

    // Contacts
    ContactAdded { contact: ContactView },
    ContactRemoved { peer_id: String },

    // Messages
    DirectMessageReceived { message: MessageView },
    GroupMessageReceived { message: MessageView },
    MessageDelivered { message_id: String },
    MessageFailed { message_id: String, reason: String },

    // Groups
    GroupCreated { group: GroupView },
    GroupJoined { group: GroupView },

    // Network
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    NetworkError { message: String },

    // Snapshot
    SnapshotLoaded { snapshot: AppSnapshot },
}
