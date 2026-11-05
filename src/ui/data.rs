//! The shapes the screens show, until the backend fills them.
//!
//! The UI reads and writes signals of these; the transport, storage and
//! crypto layers (not this crate's UI) will produce them. They are plain,
//! `Clone + PartialEq` data so a signal skips equal writes like any other.

use nsh_core::element::{PropEnum, SharedString};

/// A message's state, as the app shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum MessageState {
    /// Written, waiting for a connection.
    #[default]
    Queued,
    /// Handed to the transport.
    Sent,
    /// The peer acknowledged it.
    Delivered,
    /// The peer read it.
    Read,
    /// It could not be delivered.
    Failed,
}

impl MessageState {
    /// Its short label.
    pub fn label(self) -> &'static str {
        match self {
            MessageState::Queued => "Queued",
            MessageState::Sent => "Sent",
            MessageState::Delivered => "Delivered",
            MessageState::Read => "Read",
            MessageState::Failed => "Failed",
        }
    }
}

nsh_core::static_reactive!(MessageState: MessageState);

/// Who wrote a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender {
    /// This device's user.
    Me,
    /// A peer.
    Peer,
}

/// One message in a conversation.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Stable id (the key a list binds by).
    pub id: SharedString,
    /// Its author.
    pub sender: Sender,
    /// Its text.
    pub body: SharedString,
    /// A short time, as shown beside it.
    pub time: SharedString,
    /// Its state (only meaningful for the user's own messages).
    pub state: MessageState,
}

/// A conversation in the list: a peer or a group.
#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    /// Stable id (the key a list binds by, and the route key).
    pub id: SharedString,
    /// The peer's or group's name.
    pub name: SharedString,
    /// The last message's text (the list's second line).
    pub preview: SharedString,
    /// A short time, as shown on the row.
    pub time: SharedString,
    /// Unread messages waiting.
    pub unread: u32,
    /// The peer is reachable now.
    pub online: bool,
    /// A peer's messages (empty: a group; the UI is single-threaded for now).
    pub messages: Vec<Message>,
}

/// A contact: a peer this device knows, with its connection ticket.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    /// Stable id.
    pub id: SharedString,
    /// Its display name.
    pub name: SharedString,
    /// Its ticket or address, as shown.
    pub ticket: SharedString,
    /// It is reachable now.
    pub online: bool,
    /// The user verified it out of band.
    pub verified: bool,
}

/// The app's data, a signal of the surface the screens read (until the
/// backend fills it).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Store {
    /// The conversations, newest first.
    pub conversations: Vec<Conversation>,
    /// The known contacts.
    pub contacts: Vec<Contact>,
    /// This device's display name.
    pub me: SharedString,
    /// This device is reachable.
    pub online: bool,
}

/// Some sample conversations, so the screens have something to lay out while
/// the backend is wired (removed once the store is real).
pub fn sample_store() -> Store {
    fn message(id: &'static str, sender: Sender, body: &'static str, time: &'static str, state: MessageState) -> Message {
        Message {
            id: id.into(),
            sender,
            body: body.into(),
            time: time.into(),
            state,
        }
    }
    fn conversation(
        id: &'static str,
        name: &'static str,
        preview: &'static str,
        time: &'static str,
        unread: u32,
        online: bool,
    ) -> Conversation {
        Conversation {
            id: id.into(),
            name: name.into(),
            preview: preview.into(),
            time: time.into(),
            unread,
            online,
            messages: vec![
                message("m1", Sender::Peer, preview, time, MessageState::Read),
                message("m2", Sender::Me, "Sounds good, see you then.", time, MessageState::Read),
            ],
        }
    }
    Store {
        conversations: vec![
            conversation("ada", "Ada", "I pushed the new build — try it now.", "09:41", 2, true),
            conversation("kai", "Kai (group)", "Tunde: the ticket works on my side.", "08:12", 0, false),
            conversation("mira", "Mira", "Sent you the notes.", "Yesterday", 0, true),
            conversation("jonas", "Jonas", "Let's meet at the usual place.", "Tue", 0, false),
        ],
        contacts: vec![
            Contact {
                id: "ada".into(),
                name: "Ada".into(),
                ticket: "nodechat1q2w…9k".into(),
                online: true,
                verified: true,
            },
            Contact {
                id: "mira".into(),
                name: "Mira".into(),
                ticket: "nodechat5t6y…3m".into(),
                online: true,
                verified: false,
            },
            Contact {
                id: "jonas".into(),
                name: "Jonas".into(),
                ticket: "nodechat8u9i…7n".into(),
                online: false,
                verified: false,
            },
        ],
        me: "You".into(),
        online: true,
    }
}
