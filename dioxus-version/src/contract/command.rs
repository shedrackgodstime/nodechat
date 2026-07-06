#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum Command {
    // Identity
    GenerateIdentity { display_name: String },
    LoadIdentity,

    // Contacts
    AddContact { ticket: String },
    RemoveContact { peer_id: String },

    // Direct messages
    SendDirectMessage { peer_id: String, plaintext: String },
    SendGroupMessage { group_id: String, plaintext: String },

    // Groups
    CreateGroup { name: String, members: Vec<String> },
    JoinGroup { ticket: String },

    // Sync
    SyncRequest { peer_id: String },

    // Snapshot
    RequestSnapshot,
}
