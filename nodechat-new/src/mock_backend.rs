use std::collections::HashMap;

use crate::contract::{
    AppEvent, AppSnapshot, ChatMessage, ChatPreview, Command, ContactDirectoryEntry,
    ConversationKind, ConversationState, GroupMessage, GroupSelectionEntry, IdentityCard,
    MessageStatus,
};

pub struct MockBackend {
    snapshot: AppSnapshot,
    direct_threads: HashMap<String, Vec<ChatMessage>>,
    group_threads: HashMap<String, Vec<GroupMessage>>,
    next_message_id: u64,
}

impl MockBackend {
    pub fn new() -> Self {
        let identity = IdentityCard {
            display_name: "Shedrack".to_string(),
            initials: "S".to_string(),
            node_id: "b3f5a1d8c1e24f7a8d6e9c0f5a2b1c3d4e5f60718293a4b5c6d7e8f901234567".to_string(),
            endpoint_ticket: "nch-0-demo-ticket".to_string(),
            is_locked: false,
        };

        let chats = vec![
            ChatPreview {
                id: "7f1f3b2a9d6e4c8b".to_string(),
                name: "Ada".to_string(),
                initials: "A".to_string(),
                last_message: "I can test the new layout once it is ready.".to_string(),
                timestamp: "09:41".to_string(),
                unread: 2,
                is_group: false,
                is_online: true,
                is_relay: false,
                is_queued: false,
                is_verified: true,
                is_session_ready: true,
            },
            ChatPreview {
                id: "a0d9e5c48b7f2a10".to_string(),
                name: "Project Team".to_string(),
                initials: "P".to_string(),
                last_message: "Let us keep the interface consistent across desktop.".to_string(),
                timestamp: "08:12".to_string(),
                unread: 0,
                is_group: true,
                is_online: true,
                is_relay: false,
                is_queued: false,
                is_verified: true,
                is_session_ready: true,
            },
        ];

        let contacts = vec![
            ContactDirectoryEntry {
                id: "contact-ada".to_string(),
                name: "Ada".to_string(),
                initials: "A".to_string(),
                node_id: "7f1f3b2a9d6e4c8b".to_string(),
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
            },
            ContactDirectoryEntry {
                id: "contact-isaac".to_string(),
                name: "Isaac".to_string(),
                initials: "I".to_string(),
                node_id: "5c2d9a8b7f1e4c30".to_string(),
                is_online: false,
                is_relay: true,
                is_verified: false,
                is_session_ready: false,
            },
            ContactDirectoryEntry {
                id: "contact-zainab".to_string(),
                name: "Zainab".to_string(),
                initials: "Z".to_string(),
                node_id: "e4b9a0f17c6d2e88".to_string(),
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
            },
        ];

        let group_candidates = vec![
            GroupSelectionEntry {
                id: "contact-ada".to_string(),
                name: "Ada".to_string(),
                initials: "A".to_string(),
                is_selected: true,
                is_online: true,
            },
            GroupSelectionEntry {
                id: "contact-isaac".to_string(),
                name: "Isaac".to_string(),
                initials: "I".to_string(),
                is_selected: false,
                is_online: false,
            },
            GroupSelectionEntry {
                id: "contact-zainab".to_string(),
                name: "Zainab".to_string(),
                initials: "Z".to_string(),
                is_selected: true,
                is_online: true,
            },
        ];

        let direct_messages = vec![
            ChatMessage {
                id: "msg-1".to_string(),
                text: "Can you review the mobile layout shell?".to_string(),
                timestamp: "09:32".to_string(),
                is_mine: false,
                status: MessageStatus::Delivered,
                is_ephemeral: false,
                ttl_seconds: 0,
                is_group_invite: false,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
            },
            ChatMessage {
                id: "msg-2".to_string(),
                text: "Yes. I will keep the shell clean and reusable.".to_string(),
                timestamp: "09:36".to_string(),
                is_mine: true,
                status: MessageStatus::Sent,
                is_ephemeral: false,
                ttl_seconds: 0,
                is_group_invite: false,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
            },
        ];

        let group_messages = vec![
            GroupMessage {
                id: "gmsg-1".to_string(),
                text: "Keep the UI split into shell, component, and screen layers.".to_string(),
                timestamp: "08:55".to_string(),
                is_mine: false,
                sender_name: "Project Team".to_string(),
                status: MessageStatus::Delivered,
            },
            GroupMessage {
                id: "gmsg-2".to_string(),
                text: "Mock backend is ready for the new UI build.".to_string(),
                timestamp: "09:05".to_string(),
                is_mine: true,
                sender_name: "You".to_string(),
                status: MessageStatus::Sent,
            },
        ];

        let active_conversation = ConversationState {
            kind: ConversationKind::Direct,
            id: "7f1f3b2a9d6e4c8b".to_string(),
            title: "Ada".to_string(),
            initials: "A".to_string(),
            ticket: "nch-0-demo-ticket-ada".to_string(),
            is_online: true,
            is_session_ready: true,
            is_verified: true,
            connection_stage: "session ready".to_string(),
            member_count: "1".to_string(),
            return_screen: 0,
        };

        let snapshot = AppSnapshot {
            identity: Some(identity),
            chats,
            contacts,
            group_candidates,
            active_conversation,
            direct_messages,
            group_messages,
            debug_logs: vec![
                "[INFO] mock backend seeded with demo data".to_string(),
                "[INFO] ready for UI integration".to_string(),
            ],
        };

        let mut direct_threads = HashMap::new();
        direct_threads.insert(
            "7f1f3b2a9d6e4c8b".to_string(),
            snapshot.direct_messages.clone(),
        );

        let mut group_threads = HashMap::new();
        group_threads.insert(
            "a0d9e5c48b7f2a10".to_string(),
            snapshot.group_messages.clone(),
        );

        Self {
            snapshot,
            direct_threads,
            group_threads,
            next_message_id: 3,
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.snapshot.clone()
    }

    pub fn handle_command(&mut self, command: Command) -> Vec<AppEvent> {
        match command {
            Command::Refresh => vec![AppEvent::SnapshotReady(self.snapshot())],
            Command::LoadConversation { target, is_group } => {
                let conversation = self.load_conversation(&target, is_group);
                vec![
                    AppEvent::SnapshotReady(self.snapshot()),
                    AppEvent::ConversationLoaded(conversation.clone()),
                    AppEvent::Status(format!(
                        "loaded {} conversation {}",
                        conversation.kind, conversation.id
                    )),
                ]
            }
            Command::SendDirectMessage { target, plaintext } => {
                let message = self.append_direct_message(&target, plaintext, true);
                vec![
                    AppEvent::DirectMessageAppended {
                        conversation_id: target,
                        message,
                    },
                    AppEvent::ChatsUpdated(self.snapshot.chats.clone()),
                ]
            }
            Command::SendGroupMessage { topic, plaintext } => {
                let message = self.append_group_message(&topic, plaintext, true);
                vec![
                    AppEvent::GroupMessageAppended {
                        topic_id: topic,
                        message,
                    },
                    AppEvent::ChatsUpdated(self.snapshot.chats.clone()),
                ]
            }
            Command::ToggleVerified { node_id, verified } => {
                self.update_verified(&node_id, verified);
                vec![
                    AppEvent::ContactsUpdated(self.snapshot.contacts.clone()),
                    AppEvent::ChatsUpdated(self.snapshot.chats.clone()),
                    AppEvent::Status(format!("verification for {} set to {}", node_id, verified)),
                ]
            }
            Command::ToggleGroupMemberSelection { peer_id } => {
                self.toggle_group_candidate(&peer_id);
                vec![AppEvent::Status(format!(
                    "group member selection toggled for {}",
                    peer_id
                ))]
            }
            Command::AddContact { ticket_or_id } => {
                let added = self.add_contact(ticket_or_id);
                vec![
                    AppEvent::ContactsUpdated(self.snapshot.contacts.clone()),
                    AppEvent::ChatsUpdated(self.snapshot.chats.clone()),
                    AppEvent::Status(format!("contact added: {}", added.name)),
                ]
            }
            Command::CreateGroup { name } => {
                let group = self.create_group(name);
                vec![
                    AppEvent::ChatsUpdated(self.snapshot.chats.clone()),
                    AppEvent::Status(format!("group created: {}", group.name)),
                ]
            }
            Command::CreateIdentity { name, pin: _ } => {
                let identity = self.create_identity(name);
                vec![AppEvent::IdentityUpdated(identity)]
            }
            Command::FinaliseIdentity => vec![AppEvent::Status(
                "identity finalised and app ready".to_string(),
            )],
            Command::UnlockApp { pin } => {
                let status = if pin.is_empty() {
                    AppEvent::Error("unlock pin cannot be empty".to_string())
                } else {
                    AppEvent::Status("mock unlock complete".to_string())
                };
                vec![status]
            }
            Command::ChangePassword {
                current_pin,
                new_pin,
            } => {
                if current_pin.is_empty() || new_pin.is_empty() {
                    vec![AppEvent::Error(
                        "both current and new PIN are required".to_string(),
                    )]
                } else {
                    vec![AppEvent::Status(
                        "password change accepted by mock backend".to_string(),
                    )]
                }
            }
        }
    }

    pub fn load_conversation(&mut self, target: &str, is_group: bool) -> ConversationState {
        let conversation = if is_group {
            self.group_preview(target)
        } else {
            self.direct_preview(target)
        };
        self.snapshot.active_conversation = conversation.clone();
        if is_group {
            self.snapshot.group_messages = self
                .group_threads
                .get(target)
                .cloned()
                .unwrap_or_default();
        } else {
            self.snapshot.direct_messages = self
                .direct_threads
                .get(target)
                .cloned()
                .unwrap_or_default();
        }
        conversation
    }

    fn direct_preview(&self, target: &str) -> ConversationState {
        if let Some(chat) = self.snapshot.chats.iter().find(|chat| chat.id == target) {
            ConversationState {
                kind: ConversationKind::Direct,
                id: chat.id.clone(),
                title: chat.name.clone(),
                initials: chat.initials.clone(),
                ticket: format!("ticket-for-{}", chat.id),
                is_online: chat.is_online,
                is_session_ready: chat.is_session_ready,
                is_verified: chat.is_verified,
                connection_stage: if chat.is_session_ready {
                    "session ready".to_string()
                } else {
                    "awaiting handshake".to_string()
                },
                member_count: "1".to_string(),
                return_screen: 0,
            }
        } else {
            ConversationState::empty(ConversationKind::Direct)
        }
    }

    fn group_preview(&self, target: &str) -> ConversationState {
        if let Some(chat) = self.snapshot.chats.iter().find(|chat| chat.id == target) {
            ConversationState {
                kind: ConversationKind::Group,
                id: chat.id.clone(),
                title: chat.name.clone(),
                initials: chat.initials.clone(),
                ticket: String::new(),
                is_online: chat.is_online,
                is_session_ready: chat.is_session_ready,
                is_verified: chat.is_verified,
                connection_stage: "group ready".to_string(),
                member_count: "4".to_string(),
                return_screen: 0,
            }
        } else {
            ConversationState::empty(ConversationKind::Group)
        }
    }

    fn append_direct_message(
        &mut self,
        target: &str,
        plaintext: String,
        is_mine: bool,
    ) -> ChatMessage {
        let message = ChatMessage {
            id: format!("msg-{}", self.next_message_id),
            text: plaintext.clone(),
            timestamp: self.next_timestamp(),
            is_mine,
            status: MessageStatus::Sent,
            is_ephemeral: false,
            ttl_seconds: 0,
            is_group_invite: false,
            invite_group_name: String::new(),
            invite_topic_id: String::new(),
            invite_key: String::new(),
            invite_is_joined: false,
        };
        self.next_message_id += 1;

        self.direct_threads
            .entry(target.to_string())
            .or_default()
            .push(message.clone());

        if let Some(active) = self
            .snapshot
            .chats
            .iter_mut()
            .find(|chat| chat.id == target)
        {
            active.last_message = plaintext;
            active.timestamp = message.timestamp.clone();
            active.unread = 0;
            active.is_queued = false;
        }

        self.snapshot.direct_messages =
            self.direct_threads.get(target).cloned().unwrap_or_default();

        message
    }

    fn append_group_message(
        &mut self,
        topic: &str,
        plaintext: String,
        is_mine: bool,
    ) -> GroupMessage {
        let message = GroupMessage {
            id: format!("gmsg-{}", self.next_message_id),
            text: plaintext.clone(),
            timestamp: self.next_timestamp(),
            is_mine,
            sender_name: if is_mine {
                "You".to_string()
            } else {
                "Group member".to_string()
            },
            status: MessageStatus::Sent,
        };
        self.next_message_id += 1;

        self.group_threads
            .entry(topic.to_string())
            .or_default()
            .push(message.clone());

        if let Some(active) = self.snapshot.chats.iter_mut().find(|chat| chat.id == topic) {
            active.last_message = plaintext;
            active.timestamp = message.timestamp.clone();
        }

        self.snapshot.group_messages = self.group_threads.get(topic).cloned().unwrap_or_default();

        message
    }

    fn update_verified(&mut self, node_id: &str, verified: bool) {
        if let Some(contact) = self
            .snapshot
            .contacts
            .iter_mut()
            .find(|contact| contact.node_id == node_id)
        {
            contact.is_verified = verified;
        }

        if let Some(chat) = self
            .snapshot
            .chats
            .iter_mut()
            .find(|chat| chat.id == node_id)
        {
            chat.is_verified = verified;
        }
    }

    fn toggle_group_candidate(&mut self, peer_id: &str) {
        if let Some(candidate) = self
            .snapshot
            .group_candidates
            .iter_mut()
            .find(|candidate| candidate.id == peer_id)
        {
            candidate.is_selected = !candidate.is_selected;
        }
    }

    fn add_contact(&mut self, ticket_or_id: String) -> ContactDirectoryEntry {
        let id = ticket_or_id.clone();
        let name = Self::display_name_from_id(&ticket_or_id);
        let initials = Self::initials(&name);
        let contact = ContactDirectoryEntry {
            id: format!("contact-{}", id),
            name: name.clone(),
            initials: initials.clone(),
            node_id: id.clone(),
            is_online: false,
            is_relay: false,
            is_verified: false,
            is_session_ready: false,
        };

        self.snapshot.contacts.push(contact.clone());
        self.snapshot.chats.push(ChatPreview {
            id,
            name,
            initials,
            last_message: "New contact added in mock backend".to_string(),
            timestamp: self.next_timestamp(),
            unread: 0,
            is_group: false,
            is_online: false,
            is_relay: false,
            is_queued: false,
            is_verified: false,
            is_session_ready: false,
        });

        contact
    }

    fn create_group(&mut self, name: String) -> ChatPreview {
        let topic_id = format!("group-{}", Self::slug(&name));
        let initials = Self::initials(&name);
        let chat = ChatPreview {
            id: topic_id.clone(),
            name: name.clone(),
            initials,
            last_message: "Group created in mock backend".to_string(),
            timestamp: self.next_timestamp(),
            unread: 0,
            is_group: true,
            is_online: true,
            is_relay: false,
            is_queued: false,
            is_verified: true,
            is_session_ready: true,
        };
        self.snapshot.chats.push(chat.clone());
        self.group_threads.entry(topic_id).or_default();
        chat
    }

    fn create_identity(&mut self, name: String) -> IdentityCard {
        let identity = IdentityCard {
            display_name: name.clone(),
            initials: Self::initials(&name),
            node_id: self
                .snapshot
                .identity
                .as_ref()
                .map(|identity| identity.node_id.clone())
                .unwrap_or_else(|| "mock-node-id".to_string()),
            endpoint_ticket: self
                .snapshot
                .identity
                .as_ref()
                .map(|identity| identity.endpoint_ticket.clone())
                .unwrap_or_else(|| "mock-ticket".to_string()),
            is_locked: false,
        };
        self.snapshot.identity = Some(identity.clone());
        identity
    }

    fn next_timestamp(&self) -> String {
        let minutes = self.next_message_id % 60;
        let hours = 9 + (self.next_message_id / 60) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }

    fn slug(value: &str) -> String {
        value
            .trim()
            .to_lowercase()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn initials(value: &str) -> String {
        let mut chars = value.chars().filter(|ch| ch.is_alphabetic());
        match chars.next() {
            Some(first) => first.to_uppercase().collect(),
            None => "?".to_string(),
        }
    }

    fn display_name_from_id(value: &str) -> String {
        let cleaned = value
            .trim()
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|segment| !segment.is_empty())
            .next()
            .unwrap_or("Contact");
        let mut chars = cleaned.chars();
        match chars.next() {
            Some(first) => {
                let mut text = first.to_uppercase().collect::<String>();
                text.push_str(chars.as_str().to_lowercase().as_str());
                text
            }
            None => "Contact".to_string(),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_contains_demo_data() {
        let backend = MockBackend::new();
        let snapshot = backend.snapshot();
        assert!(!snapshot.chats.is_empty());
        assert!(!snapshot.contacts.is_empty());
        assert!(snapshot.identity.is_some());
    }

    #[test]
    fn direct_message_updates_thread_and_preview() {
        let mut backend = MockBackend::new();
        let events = backend.handle_command(Command::SendDirectMessage {
            target: "7f1f3b2a9d6e4c8b".to_string(),
            plaintext: "Hello from the mock backend".to_string(),
        });

        assert!(matches!(events[0], AppEvent::DirectMessageAppended { .. }));
        assert!(
            backend
                .snapshot()
                .chats
                .iter()
                .any(|chat| chat.last_message == "Hello from the mock backend")
        );
    }
}
