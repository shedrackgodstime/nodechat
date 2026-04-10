use std::collections::HashMap;

use crate::contract::{
    AppEvent, AppFlags, AppInfoView, AppSnapshot, ChatListItem, Command, ContactListItem, ConversationKind,
    ConversationView, GroupCandidateItem, HistoryScope, IdentityView, MessageItem, MessageKind,
    MessageStatus,
};

pub struct MockBackend {
    snapshot: AppSnapshot,
    threads: HashMap<String, Vec<MessageItem>>,
    next_message_id: u64,
}

impl MockBackend {
    pub fn new() -> Self {
        let ada_peer_id = "7f1f3b2a9d6e4c8b".to_string();
        let ada_conversation_id = Self::direct_conversation_id(&ada_peer_id);
        let project_team_conversation_id = "group-project-team".to_string();

        let identity = IdentityView {
            display_name: "Shedrack".to_string(),
            initials: "S".to_string(),
            peer_id: "b3f5a1d8c1e24f7a8d6e9c0f5a2b1c3d4e5f60718293a4b5c6d7e8f901234567"
                .to_string(),
            endpoint_ticket: "nch-0-demo-ticket".to_string(),
            is_locked: false,
            has_identity: true,
        };

        let app_info = AppInfoView::current();

        let contact_list = vec![
            ContactListItem {
                contact_id: "contact-ada".to_string(),
                peer_id: ada_peer_id.clone(),
                display_name: "Ada".to_string(),
                initials: "A".to_string(),
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
                direct_conversation_id: ada_conversation_id.clone(),
            },
            ContactListItem {
                contact_id: "contact-isaac".to_string(),
                peer_id: "5c2d9a8b7f1e4c30".to_string(),
                display_name: "Isaac".to_string(),
                initials: "I".to_string(),
                is_online: false,
                is_relay: true,
                is_verified: false,
                is_session_ready: false,
                direct_conversation_id: Self::direct_conversation_id("5c2d9a8b7f1e4c30"),
            },
            ContactListItem {
                contact_id: "contact-zainab".to_string(),
                peer_id: "e4b9a0f17c6d2e88".to_string(),
                display_name: "Zainab".to_string(),
                initials: "Z".to_string(),
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
                direct_conversation_id: Self::direct_conversation_id("e4b9a0f17c6d2e88"),
            },
        ];

        let chat_list = vec![
            ChatListItem {
                conversation_id: ada_conversation_id.clone(),
                kind: ConversationKind::Direct,
                title: "Ada".to_string(),
                initials: "A".to_string(),
                last_message: "I can test the new layout once it is ready.".to_string(),
                timestamp: "09:41".to_string(),
                unread_count: 2,
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
                has_queued_messages: false,
            },
            ChatListItem {
                conversation_id: project_team_conversation_id.clone(),
                kind: ConversationKind::Group,
                title: "Project Team".to_string(),
                initials: "P".to_string(),
                last_message: "Let us keep the interface consistent across desktop.".to_string(),
                timestamp: "08:12".to_string(),
                unread_count: 0,
                is_online: true,
                is_relay: false,
                is_verified: true,
                is_session_ready: true,
                has_queued_messages: false,
            },
        ];

        let group_candidates = vec![
            GroupCandidateItem {
                contact_id: "contact-ada".to_string(),
                display_name: "Ada".to_string(),
                initials: "A".to_string(),
                is_selected: true,
                is_online: true,
            },
            GroupCandidateItem {
                contact_id: "contact-isaac".to_string(),
                display_name: "Isaac".to_string(),
                initials: "I".to_string(),
                is_selected: false,
                is_online: false,
            },
            GroupCandidateItem {
                contact_id: "contact-zainab".to_string(),
                display_name: "Zainab".to_string(),
                initials: "Z".to_string(),
                is_selected: true,
                is_online: true,
            },
        ];

        let direct_messages = vec![
            MessageItem {
                message_id: "msg-1".to_string(),
                conversation_id: ada_conversation_id.clone(),
                sender_name: "Ada".to_string(),
                text: "Can you review the mobile layout shell?".to_string(),
                timestamp: "09:32".to_string(),
                is_outgoing: false,
                is_system: false,
                status: MessageStatus::Delivered,
                kind: MessageKind::Standard,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
                is_ephemeral: false,
                ttl_seconds: 0,
            },
            MessageItem {
                message_id: "msg-2".to_string(),
                conversation_id: ada_conversation_id.clone(),
                sender_name: "You".to_string(),
                text: "Yes. I will keep the shell clean and reusable.".to_string(),
                timestamp: "09:36".to_string(),
                is_outgoing: true,
                is_system: false,
                status: MessageStatus::Sent,
                kind: MessageKind::Standard,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
                is_ephemeral: false,
                ttl_seconds: 0,
            },
        ];

        let group_messages = vec![
            MessageItem {
                message_id: "gmsg-1".to_string(),
                conversation_id: project_team_conversation_id.clone(),
                sender_name: "Project Team".to_string(),
                text: "Keep the UI split into shell, component, and screen layers.".to_string(),
                timestamp: "08:55".to_string(),
                is_outgoing: false,
                is_system: false,
                status: MessageStatus::Delivered,
                kind: MessageKind::Standard,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
                is_ephemeral: false,
                ttl_seconds: 0,
            },
            MessageItem {
                message_id: "gmsg-2".to_string(),
                conversation_id: project_team_conversation_id.clone(),
                sender_name: "You".to_string(),
                text: "Mock backend is ready for the new UI build.".to_string(),
                timestamp: "09:05".to_string(),
                is_outgoing: true,
                is_system: false,
                status: MessageStatus::Sent,
                kind: MessageKind::Standard,
                invite_group_name: String::new(),
                invite_topic_id: String::new(),
                invite_key: String::new(),
                invite_is_joined: false,
                is_ephemeral: false,
                ttl_seconds: 0,
            },
        ];

        let mut threads = HashMap::new();
        threads.insert(ada_conversation_id.clone(), direct_messages.clone());
        threads.insert(project_team_conversation_id.clone(), group_messages);

        let active_conversation = ConversationView {
            conversation_id: ada_conversation_id.clone(),
            kind: ConversationKind::Direct,
            title: "Ada".to_string(),
            initials: "A".to_string(),
            peer_id: ada_peer_id,
            ticket: "nch-0-demo-ticket-ada".to_string(),
            is_online: true,
            is_relay: false,
            is_verified: true,
            is_session_ready: true,
            connection_stage: "session ready".to_string(),
            member_count: 1,
            return_screen: 0,
        };

        let snapshot = AppSnapshot {
            identity,
            app_info,
            app_flags: AppFlags {
                direct_peer_count: 2,
                relay_peer_count: 1,
                is_offline: false,
            },
            chat_list,
            contact_list,
            group_candidates,
            active_conversation,
            active_messages: direct_messages,
            debug_feed: vec![
                "[INFO] mock backend seeded with demo data".to_string(),
                "[INFO] ready for UI integration".to_string(),
            ],
        };

        Self {
            snapshot,
            threads,
            next_message_id: 3,
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.snapshot.clone()
    }

    pub fn handle_command(&mut self, command: Command) -> Vec<AppEvent> {
        match command {
            Command::Refresh => vec![AppEvent::SnapshotReady(self.snapshot())],
            Command::LoadConversation { conversation_id } => {
                let conversation = self.load_conversation(&conversation_id);
                if conversation.conversation_id.is_empty() {
                    return vec![AppEvent::UserError(format!(
                        "conversation not found: {}",
                        conversation_id
                    ))];
                }

                vec![
                    AppEvent::ConversationUpdated(conversation.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id: conversation.conversation_id.clone(),
                        messages: self.snapshot.active_messages.clone(),
                    },
                    AppEvent::StatusNotice(format!(
                        "loaded {} conversation {}",
                        conversation.kind, conversation.conversation_id
                    )),
                ]
            }
            Command::SendMessage {
                conversation_id,
                plaintext,
            } => {
                let Some(message) = self.append_message(&conversation_id, plaintext, true) else {
                    return vec![AppEvent::UserError(format!(
                        "conversation not found: {}",
                        conversation_id
                    ))];
                };

                vec![
                    AppEvent::MessageAppended {
                        conversation_id: conversation_id.clone(),
                        message,
                    },
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                ]
            }
            Command::RetryQueuedMessage {
                conversation_id,
                message_id,
            } => {
                if self.mark_message_sent(&conversation_id, &message_id) {
                    vec![
                        AppEvent::MessageListReplaced {
                            conversation_id,
                            messages: self.snapshot.active_messages.clone(),
                        },
                        AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                        AppEvent::StatusNotice("queued message retried".to_string()),
                    ]
                } else {
                    vec![AppEvent::UserError("queued message not found".to_string())]
                }
            }
            Command::DeleteConversation {
                conversation_id,
                confirmation_pin: _,
            } => {
                if self.delete_conversation(&conversation_id) {
                    vec![
                        AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                        AppEvent::ConversationUpdated(
                            self.snapshot.active_conversation.clone(),
                        ),
                        AppEvent::MessageListReplaced {
                            conversation_id: self
                                .snapshot
                                .active_conversation
                                .conversation_id
                                .clone(),
                            messages: self.snapshot.active_messages.clone(),
                        },
                        AppEvent::StatusNotice("conversation deleted".to_string()),
                    ]
                } else {
                    vec![AppEvent::UserError("conversation not found".to_string())]
                }
            }
            Command::AddContact { ticket_or_peer_id } => {
                let added = self.add_contact(ticket_or_peer_id);
                vec![
                    AppEvent::ContactListUpdated(self.snapshot.contact_list.clone()),
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                    AppEvent::StatusNotice(format!(
                        "contact added: {}",
                        added.display_name
                    )),
                ]
            }
            Command::CreateGroup {
                name,
                member_contact_ids,
            } => {
                let group = self.create_group(name, member_contact_ids.len() as i32);
                vec![
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                    AppEvent::StatusNotice(format!("group created: {}", group.title)),
                ]
            }
            Command::ToggleGroupCandidate { contact_id } => {
                self.toggle_group_candidate(&contact_id);
                vec![
                    AppEvent::GroupCandidatesUpdated(self.snapshot.group_candidates.clone()),
                    AppEvent::StatusNotice(format!(
                        "group member selection toggled for {}",
                        contact_id
                    )),
                ]
            }
            Command::OpenDirectConversation { contact_id } => {
                let Some(conversation_id) = self.direct_conversation_for_contact(&contact_id) else {
                    return vec![AppEvent::UserError(format!(
                        "contact not found: {}",
                        contact_id
                    ))];
                };
                let conversation = self.load_conversation(&conversation_id);
                vec![
                    AppEvent::ConversationUpdated(conversation.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id,
                        messages: self.snapshot.active_messages.clone(),
                    },
                ]
            }
            Command::CreateIdentity { display_name, pin: _ } => {
                let identity = self.create_identity(display_name);
                vec![AppEvent::IdentityUpdated(identity)]
            }
            Command::FinalizeIdentity => vec![AppEvent::StatusNotice(
                "identity finalised and app ready".to_string(),
            )],
            Command::UnlockApp { pin } => {
                if pin.is_empty() {
                    vec![AppEvent::UserError("unlock pin cannot be empty".to_string())]
                } else {
                    self.snapshot.identity.is_locked = false;
                    vec![
                        AppEvent::IdentityUpdated(self.snapshot.identity.clone()),
                        AppEvent::StatusNotice("mock unlock complete".to_string()),
                    ]
                }
            }
            Command::ChangePassword {
                current_pin,
                new_pin,
            } => {
                if current_pin.is_empty() || new_pin.is_empty() {
                    vec![AppEvent::UserError(
                        "both current and new PIN are required".to_string(),
                    )]
                } else {
                    vec![AppEvent::StatusNotice(
                        "password change accepted by mock backend".to_string(),
                    )]
                }
            }
            Command::UpdateDisplayName { display_name } => {
                if display_name.trim().is_empty() {
                    vec![AppEvent::UserError(
                        "display name cannot be empty".to_string(),
                    )]
                } else {
                    self.update_display_name(display_name);
                    vec![AppEvent::IdentityUpdated(self.snapshot.identity.clone())]
                }
            }
            Command::ResetIdentity { confirmation_pin } => {
                if confirmation_pin.is_empty() {
                    vec![AppEvent::UserError(
                        "confirmation PIN is required".to_string(),
                    )]
                } else {
                    self.reset_identity();
                    vec![
                        AppEvent::IdentityUpdated(self.snapshot.identity.clone()),
                        AppEvent::StatusNotice("identity reset in mock backend".to_string()),
                    ]
                }
            }
            Command::SetVerification { peer_id, verified } => {
                self.update_verified(&peer_id, verified);
                vec![
                    AppEvent::ContactListUpdated(self.snapshot.contact_list.clone()),
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                    AppEvent::ConversationUpdated(self.snapshot.active_conversation.clone()),
                    AppEvent::StatusNotice(format!(
                        "verification for {} set to {}",
                        peer_id, verified
                    )),
                ]
            }
            Command::AcceptGroupInvite {
                conversation_id,
                topic_id,
                invite_key: _,
            } => {
                let target_conversation_id = if self
                    .snapshot
                    .chat_list
                    .iter()
                    .any(|item| item.conversation_id == conversation_id)
                {
                    conversation_id
                } else {
                    let chat = self.create_group(
                        format!("Joined {}", topic_id),
                        self.selected_group_candidates() + 1,
                    );
                    chat.conversation_id
                };

                vec![
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                    AppEvent::StatusNotice(format!(
                        "group invite accepted for {}",
                        target_conversation_id
                    )),
                ]
            }
            Command::ClearMessageHistory {
                scope,
                confirmation_pin: _,
            } => {
                self.clear_history(scope);
                vec![
                    AppEvent::ChatListUpdated(self.snapshot.chat_list.clone()),
                    AppEvent::MessageListReplaced {
                        conversation_id: self
                            .snapshot
                            .active_conversation
                            .conversation_id
                            .clone(),
                        messages: self.snapshot.active_messages.clone(),
                    },
                    AppEvent::StatusNotice("message history cleared".to_string()),
                ]
            }
        }
    }

    pub fn load_conversation(&mut self, conversation_id: &str) -> ConversationView {
        let conversation = self.conversation_view(conversation_id);
        self.snapshot.active_conversation = conversation.clone();
        self.snapshot.active_messages = self
            .threads
            .get(conversation_id)
            .cloned()
            .unwrap_or_default();
        conversation
    }

    fn conversation_view(&self, conversation_id: &str) -> ConversationView {
        let Some(chat) = self
            .snapshot
            .chat_list
            .iter()
            .find(|chat| chat.conversation_id == conversation_id)
        else {
            return ConversationView::empty(ConversationKind::Direct);
        };

        match chat.kind {
            ConversationKind::Direct => {
                let contact = self
                    .snapshot
                    .contact_list
                    .iter()
                    .find(|contact| contact.direct_conversation_id == conversation_id);
                ConversationView {
                    conversation_id: chat.conversation_id.clone(),
                    kind: chat.kind,
                    title: chat.title.clone(),
                    initials: chat.initials.clone(),
                    peer_id: contact
                        .map(|contact| contact.peer_id.clone())
                        .unwrap_or_default(),
                    ticket: format!("ticket-for-{}", chat.conversation_id),
                    is_online: chat.is_online,
                    is_relay: chat.is_relay,
                    is_verified: chat.is_verified,
                    is_session_ready: chat.is_session_ready,
                    connection_stage: if chat.is_session_ready {
                        "session ready".to_string()
                    } else {
                        "awaiting handshake".to_string()
                    },
                    member_count: 1,
                    return_screen: 0,
                }
            }
            ConversationKind::Group => ConversationView {
                conversation_id: chat.conversation_id.clone(),
                kind: chat.kind,
                title: chat.title.clone(),
                initials: chat.initials.clone(),
                peer_id: String::new(),
                ticket: String::new(),
                is_online: chat.is_online,
                is_relay: chat.is_relay,
                is_verified: chat.is_verified,
                is_session_ready: chat.is_session_ready,
                connection_stage: "group ready".to_string(),
                member_count: (self.selected_group_candidates() + 1).max(1),
                return_screen: 0,
            },
        }
    }

    fn append_message(
        &mut self,
        conversation_id: &str,
        plaintext: String,
        is_outgoing: bool,
    ) -> Option<MessageItem> {
        let chat = self
            .snapshot
            .chat_list
            .iter()
            .find(|chat| chat.conversation_id == conversation_id)?
            .clone();

        let message = MessageItem {
            message_id: format!("msg-{}", self.next_message_id),
            conversation_id: conversation_id.to_string(),
            sender_name: if is_outgoing {
                "You".to_string()
            } else {
                chat.title.clone()
            },
            text: plaintext.clone(),
            timestamp: self.next_timestamp(),
            is_outgoing,
            is_system: false,
            status: if is_outgoing {
                MessageStatus::Sent
            } else {
                MessageStatus::Delivered
            },
            kind: MessageKind::Standard,
            invite_group_name: String::new(),
            invite_topic_id: String::new(),
            invite_key: String::new(),
            invite_is_joined: false,
            is_ephemeral: false,
            ttl_seconds: 0,
        };
        self.next_message_id += 1;

        self.threads
            .entry(conversation_id.to_string())
            .or_default()
            .push(message.clone());

        if let Some(active) = self
            .snapshot
            .chat_list
            .iter_mut()
            .find(|chat| chat.conversation_id == conversation_id)
        {
            active.last_message = plaintext;
            active.timestamp = message.timestamp.clone();
            active.unread_count = 0;
            active.has_queued_messages = false;
        }

        if self.snapshot.active_conversation.conversation_id == conversation_id {
            self.snapshot.active_messages = self
                .threads
                .get(conversation_id)
                .cloned()
                .unwrap_or_default();
        }

        Some(message)
    }

    fn mark_message_sent(&mut self, conversation_id: &str, message_id: &str) -> bool {
        let Some(thread) = self.threads.get_mut(conversation_id) else {
            return false;
        };

        let Some(message) = thread.iter_mut().find(|message| message.message_id == message_id) else {
            return false;
        };

        message.status = MessageStatus::Sent;
        if let Some(active) = self
            .snapshot
            .chat_list
            .iter_mut()
            .find(|chat| chat.conversation_id == conversation_id)
        {
            active.has_queued_messages = false;
        }
        if self.snapshot.active_conversation.conversation_id == conversation_id {
            self.snapshot.active_messages = thread.clone();
        }
        true
    }

    fn update_verified(&mut self, peer_id: &str, verified: bool) {
        if let Some(contact) = self
            .snapshot
            .contact_list
            .iter_mut()
            .find(|contact| contact.peer_id == peer_id)
        {
            contact.is_verified = verified;

            if let Some(chat) = self
                .snapshot
                .chat_list
                .iter_mut()
                .find(|chat| chat.conversation_id == contact.direct_conversation_id)
            {
                chat.is_verified = verified;
            }
        }

        if self.snapshot.active_conversation.peer_id == peer_id {
            self.snapshot.active_conversation.is_verified = verified;
        }
    }

    fn toggle_group_candidate(&mut self, contact_id: &str) {
        if let Some(candidate) = self
            .snapshot
            .group_candidates
            .iter_mut()
            .find(|candidate| candidate.contact_id == contact_id)
        {
            candidate.is_selected = !candidate.is_selected;
        }
    }

    fn add_contact(&mut self, ticket_or_peer_id: String) -> ContactListItem {
        let peer_id = ticket_or_peer_id.clone();
        let display_name = Self::display_name_from_id(&ticket_or_peer_id);
        let initials = Self::initials(&display_name);
        let conversation_id = Self::direct_conversation_id(&peer_id);

        let contact = ContactListItem {
            contact_id: format!("contact-{}", peer_id),
            peer_id: peer_id.clone(),
            display_name: display_name.clone(),
            initials: initials.clone(),
            is_online: false,
            is_relay: false,
            is_verified: false,
            is_session_ready: false,
            direct_conversation_id: conversation_id.clone(),
        };

        self.snapshot.contact_list.push(contact.clone());
        self.snapshot.chat_list.push(ChatListItem {
            conversation_id: conversation_id.clone(),
            kind: ConversationKind::Direct,
            title: display_name,
            initials,
            last_message: "New contact added in mock backend".to_string(),
            timestamp: self.next_timestamp(),
            unread_count: 0,
            is_online: false,
            is_relay: false,
            is_verified: false,
            is_session_ready: false,
            has_queued_messages: false,
        });
        self.threads.entry(conversation_id).or_default();
        self.recompute_app_flags();

        contact
    }

    fn create_group(&mut self, name: String, member_count: i32) -> ChatListItem {
        let conversation_id = format!("group-{}", Self::slug(&name));
        let chat = ChatListItem {
            conversation_id: conversation_id.clone(),
            kind: ConversationKind::Group,
            title: name.clone(),
            initials: Self::initials(&name),
            last_message: "Group created in mock backend".to_string(),
            timestamp: self.next_timestamp(),
            unread_count: 0,
            is_online: true,
            is_relay: false,
            is_verified: true,
            is_session_ready: true,
            has_queued_messages: false,
        };
        self.snapshot.chat_list.push(chat.clone());
        self.threads.entry(conversation_id.clone()).or_default();
        if self.snapshot.active_conversation.conversation_id == conversation_id {
            self.snapshot.active_conversation.member_count = member_count;
        }
        chat
    }

    fn create_identity(&mut self, display_name: String) -> IdentityView {
        let identity = IdentityView {
            display_name: display_name.clone(),
            initials: Self::initials(&display_name),
            peer_id: if self.snapshot.identity.peer_id.is_empty() {
                "mock-node-id".to_string()
            } else {
                self.snapshot.identity.peer_id.clone()
            },
            endpoint_ticket: if self.snapshot.identity.endpoint_ticket.is_empty() {
                "mock-ticket".to_string()
            } else {
                self.snapshot.identity.endpoint_ticket.clone()
            },
            is_locked: false,
            has_identity: true,
        };
        self.snapshot.identity = identity.clone();
        identity
    }

    fn update_display_name(&mut self, display_name: String) {
        self.snapshot.identity.display_name = display_name.clone();
        self.snapshot.identity.initials = Self::initials(&display_name);
    }

    fn reset_identity(&mut self) {
        self.snapshot.identity = IdentityView::empty();
    }

    fn clear_history(&mut self, scope: HistoryScope) {
        match scope {
            HistoryScope::ActiveConversation => {
                let conversation_id = self.snapshot.active_conversation.conversation_id.clone();
                if let Some(thread) = self.threads.get_mut(&conversation_id) {
                    thread.clear();
                }
                self.snapshot.active_messages.clear();
                if let Some(chat) = self
                    .snapshot
                    .chat_list
                    .iter_mut()
                    .find(|chat| chat.conversation_id == conversation_id)
                {
                    chat.last_message = "No messages yet".to_string();
                    chat.unread_count = 0;
                }
            }
            HistoryScope::AllConversations => {
                for thread in self.threads.values_mut() {
                    thread.clear();
                }
                for chat in &mut self.snapshot.chat_list {
                    chat.last_message = "No messages yet".to_string();
                    chat.unread_count = 0;
                }
                self.snapshot.active_messages.clear();
            }
        }
    }

    fn delete_conversation(&mut self, conversation_id: &str) -> bool {
        let original_len = self.snapshot.chat_list.len();
        self.snapshot
            .chat_list
            .retain(|chat| chat.conversation_id != conversation_id);
        self.threads.remove(conversation_id);

        if self.snapshot.chat_list.len() == original_len {
            return false;
        }

        if self.snapshot.active_conversation.conversation_id == conversation_id {
            if let Some(next_chat) = self.snapshot.chat_list.first().cloned() {
                self.load_conversation(&next_chat.conversation_id);
            } else {
                self.snapshot.active_conversation = ConversationView::empty(ConversationKind::Direct);
                self.snapshot.active_messages.clear();
            }
        }

        true
    }

    fn selected_group_candidates(&self) -> i32 {
        self.snapshot
            .group_candidates
            .iter()
            .filter(|candidate| candidate.is_selected)
            .count() as i32
    }

    fn direct_conversation_for_contact(&self, contact_id: &str) -> Option<String> {
        self.snapshot
            .contact_list
            .iter()
            .find(|contact| contact.contact_id == contact_id)
            .map(|contact| contact.direct_conversation_id.clone())
    }

    fn recompute_app_flags(&mut self) {
        self.snapshot.app_flags.direct_peer_count = self
            .snapshot
            .contact_list
            .iter()
            .filter(|contact| !contact.is_relay)
            .count() as i32;
        self.snapshot.app_flags.relay_peer_count = self
            .snapshot
            .contact_list
            .iter()
            .filter(|contact| contact.is_relay)
            .count() as i32;
    }

    fn direct_conversation_id(peer_id: &str) -> String {
        format!("dm-{}", peer_id)
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
        assert!(!snapshot.chat_list.is_empty());
        assert!(!snapshot.contact_list.is_empty());
        assert!(snapshot.identity.has_identity);
    }

    #[test]
    fn send_message_updates_thread_and_preview() {
        let mut backend = MockBackend::new();
        let events = backend.handle_command(Command::SendMessage {
            conversation_id: "dm-7f1f3b2a9d6e4c8b".to_string(),
            plaintext: "Hello from the mock backend".to_string(),
        });

        assert!(matches!(events[0], AppEvent::MessageAppended { .. }));
        assert!(backend.snapshot().chat_list.iter().any(|chat| {
            chat.last_message == "Hello from the mock backend"
                && chat.conversation_id == "dm-7f1f3b2a9d6e4c8b"
        }));
    }
}
