//! A conversation: its header, its messages and the composer.
//!
//! The header follows the platform conventions (Apple's navigation bar,
//! Material's top app bar): a leading back control, then the peer (avatar,
//! name and status), then the conversation's actions. The back control
//! appears when the conversation is a pushed screen (phones); in the wide
//! layout's pane there is nothing to go back to, so it is left out.
//!
//! The message list is a virtualized list whose cells bind to the message
//! they show (as NSH's own chat demo does): a cell is made once and reads its
//! row's value, so a change costs what it changes.

use nsh_base::input;
use nsh_core::{
    element::{AnyElement, Point, Points, ScrollEvent, SharedString, Size},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::{
        ConnectionRequest, Contact, Conversation, ConversationKind, Invitation, InvitationKind,
        InvitationStatus, Message, MessageAttachment, MessageState, Sender,
    },
    icons::icon,
    press,
    shell::store,
    theme::{chat, colors, radius},
    widgets::{Avatar, ContactIdentity, state_mark},
};

/// A message bubble: the user's on the right (accent), a peer's on the left.
fn bubble(
    message: Message,
    store: Signal<crate::ui::data::Store>,
    messages: Signal<Vec<Message>>,
    conversation: Signal<Option<Conversation>>,
) -> AnyElement {
    let mine = message.sender == Sender::Me;
    view()
        .w_full()
        .flex_row()
        .when(mine, |v| v.justify_end())
        .px_4()
        .py_1()
        .child(
            view()
                .max_w(relative(0.78))
                .px(px(14.0))
                .py(px(9.0))
                .gap(px(4.0))
                .rounded(px(18.0))
                .border_curve(BorderCurve::Continuous)
                .when(mine, |v| v.rounded_br(px(6.0)).bg(chat::SENT))
                .when(!mine, |v| v.rounded_bl(px(6.0)).bg(chat::RECEIVED))
                .children((!message.body.is_empty()).then(|| {
                    text(message.body.clone())
                        .select_text()
                        .text_size(px(15.0))
                        .line_height(px(21.0))
                        .text_color(if mine {
                            chat::SENT_FOREGROUND
                        } else {
                            chat::RECEIVED_FOREGROUND
                        })
                }))
                .children(
                    message
                        .attachment
                        .clone()
                        .map(|attachment| attachment_card(attachment, mine)),
                )
                .children(
                    message.invitation.clone().map(|invitation| {
                        invitation_card(invitation, store, messages, conversation)
                    }),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .justify_end()
                        .gap(px(4.0))
                        .child(
                            text(message.time.clone())
                                .text_size(px(11.0))
                                .text_color(if mine {
                                    chat::SENT_FOREGROUND
                                } else {
                                    colors::FAINT
                                })
                                .opacity(0.85),
                        )
                        .when(mine, |v| v.child(state_mark(message.state))),
                ),
        )
        .into_element()
}

fn attachment_card(attachment: MessageAttachment, mine: bool) -> AnyElement {
    let size = attachment.data.len();
    let size_label = if size < 1024 * 1024 {
        format!("{} KB", (size + 1023) / 1024)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    };
    view()
        .mt_1()
        .min_w(px(180.0))
        .flex_row()
        .items_center()
        .gap_2()
        .p_2()
        .rounded(px(radius::CONTROL))
        .bg(if mine {
            colors::ACCENT_FILL
        } else {
            colors::BACKGROUND
        })
        .child(
            icon()
                .name(IconName::Attach)
                .size(18.0)
                .tint(colors::ACCENT),
        )
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(attachment.name)
                        .number_of_lines(1)
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if mine {
                            chat::SENT_FOREGROUND
                        } else {
                            colors::TEXT
                        }),
                )
                .child(
                    text(format!("{size_label} · {}", attachment.mime))
                        .number_of_lines(1)
                        .text_size(px(11.0))
                        .text_color(if mine {
                            chat::SENT_FOREGROUND
                        } else {
                            colors::MUTED
                        }),
                ),
        )
        .into_element()
}

fn invitation_card(
    invitation: Invitation,
    store: Signal<crate::ui::data::Store>,
    messages: Signal<Vec<Message>>,
    conversation: Signal<Option<Conversation>>,
) -> AnyElement {
    let title = match invitation.kind {
        InvitationKind::Contact => "Contact invitation",
        InvitationKind::Group => "Private group invitation",
    };
    let name = invitation.name.to_string();
    let status = invitation.status;
    view()
        .mt_2()
        .p_3()
        .gap_2()
        .rounded(px(radius::INSET))
        .bg(colors::BACKGROUND)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            text(title)
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::ACCENT),
        )
        .child(
            text(invitation.name.clone())
                .text_size(px(15.0))
                .font_weight(FontWeight::BOLD)
                .text_color(colors::TEXT),
        )
        .child(
            text(invitation.ticket.clone())
                .number_of_lines(1)
                .text_size(px(10.0))
                .font_family("monospace")
                .text_color(colors::FAINT),
        )
        .child(
            text(match invitation.kind {
                InvitationKind::Contact => "Preview the peer before adding them.",
                InvitationKind::Group if invitation.approval_required => {
                    "Preview the group and request to join with its ticket."
                }
                InvitationKind::Group => "Preview the group and its ticket before joining.",
            })
            .text_size(px(12.0))
            .text_color(colors::MUTED),
        )
        .child(if status != InvitationStatus::Pending {
            text(match status {
                InvitationStatus::Pending => "Invitation is ready",
                InvitationStatus::RequestSent => "Join request sent · waiting for a group admin",
                InvitationStatus::Joined if invitation.kind == InvitationKind::Group => {
                    "Joined on this device · no network connection is active"
                }
                InvitationStatus::Joined => {
                    "Added on this device · no network connection is active"
                }
            })
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors::MUTED)
            .into_element()
        } else {
            let current_store = store;
            let current_messages = messages;
            let current_conversation = conversation;
            let current_invitation = invitation.clone();
            press()
                .label(
                    if invitation.kind == InvitationKind::Group && invitation.approval_required {
                        format!("Request to join {name}")
                    } else if invitation.kind == InvitationKind::Group {
                        format!("Join {name}")
                    } else {
                        format!("Add {name}")
                    },
                )
                .on_press(move || {
                    accept_invitation(
                        &current_store,
                        &current_messages,
                        &current_conversation,
                        &current_invitation,
                    )
                })
                .child(
                    view()
                        .h(px(36.0))
                        .px_3()
                        .rounded(px(radius::CONTROL))
                        .items_center()
                        .justify_center()
                        .bg(colors::ACCENT_FILL)
                        .child(
                            text(
                                if invitation.kind == InvitationKind::Group
                                    && invitation.approval_required
                                {
                                    "Previewed · request to join"
                                } else if invitation.kind == InvitationKind::Group {
                                    "Previewed · join group"
                                } else {
                                    "Previewed · add contact"
                                },
                            )
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::ON_ACCENT),
                        ),
                )
                .into_element()
        })
        .into_element()
}

fn accept_invitation(
    store: &Signal<crate::ui::data::Store>,
    messages: &Signal<Vec<Message>>,
    conversation: &Signal<Option<Conversation>>,
    invitation: &Invitation,
) {
    let invitation_id = invitation.id.to_string();
    let mut joined_group = None;
    let status = if invitation.kind == InvitationKind::Group && invitation.approval_required {
        InvitationStatus::RequestSent
    } else {
        InvitationStatus::Joined
    };
    let conversation_id = conversation
        .with_untracked(|current| current.as_ref().map(|current| current.id.to_string()));
    if let Some(id) = conversation_id {
        conversation.update(|current| {
            if let Some(current) = current {
                for message in &mut current.messages {
                    if message
                        .invitation
                        .as_ref()
                        .is_some_and(|item| item.id.to_string() == invitation_id)
                    {
                        if let Some(item) = &mut message.invitation {
                            item.status = status;
                        }
                    }
                }
            }
        });
        messages.update(|items| {
            for message in items {
                if message
                    .invitation
                    .as_ref()
                    .is_some_and(|item| item.id.to_string() == invitation_id)
                {
                    if let Some(item) = &mut message.invitation {
                        item.status = status;
                    }
                }
            }
        });
        store.update(|state| {
            let invitation_id = invitation_id.clone();
            if let Some(chat) = state
                .conversations
                .iter_mut()
                .find(|item| item.id.to_string() == id)
            {
                for message in &mut chat.messages {
                    if message
                        .invitation
                        .as_ref()
                        .is_some_and(|item| item.id.to_string() == invitation_id)
                    {
                        if let Some(item) = &mut message.invitation {
                            item.status = status;
                        }
                    }
                }
            }
            match invitation.kind {
                InvitationKind::Contact => {
                    let name = invitation.name.to_string();
                    let ticket = invitation.ticket.to_string();
                    let contact_id = format!("invite-{}", invitation_id);
                    if !state
                        .contacts
                        .iter()
                        .any(|contact| contact.id.to_string() == contact_id)
                    {
                        state.contacts.push(Contact {
                            id: contact_id.clone().into(),
                            name: name.clone().into(),
                            ticket: ticket.clone().into(),
                            online: false,
                            verified: false,
                        });
                        state.conversations.insert(
                            0,
                            Conversation {
                                id: contact_id.into(),
                                name: name.into(),
                                preview:
                                    "Added from an invitation. Waiting for a direct connection."
                                        .into(),
                                time: "Now".into(),
                                unread: 0,
                                online: false,
                                kind: ConversationKind::Direct,
                                ticket: ticket.into(),
                                members: Vec::new(),
                                approval_required: false,
                                messages: Vec::new(),
                            },
                        );
                    }
                }
                InvitationKind::Group => {
                    let name = invitation.name.to_string();
                    if invitation.approval_required {
                        let request_id = format!("invite-request-{invitation_id}");
                        if !state
                            .requests
                            .iter()
                            .any(|request| request.id.to_string() == request_id)
                        {
                            state.requests.push(ConnectionRequest {
                                id: request_id.into(),
                                name: invitation.name.clone(),
                                ticket: invitation.ticket.clone(),
                                note: "Waiting for a group admin to approve your request.".into(),
                                kind: InvitationKind::Group,
                                incoming: false,
                            });
                        }
                    } else {
                        let existing = state
                            .conversations
                            .iter()
                            .find(|chat| {
                                chat.kind == ConversationKind::Group
                                    && (chat.ticket == invitation.ticket
                                        || chat.name.to_string() == name)
                            })
                            .cloned();
                        let group = existing.unwrap_or_else(|| {
                            let id = format!("invite-group-{}", invitation_id);
                            Conversation {
                                id: id.into(),
                                name: invitation.name.clone(),
                                preview: "Joined on this device. No network connection is active."
                                    .into(),
                                time: "Now".into(),
                                unread: 0,
                                online: false,
                                kind: ConversationKind::Group,
                                ticket: invitation.ticket.clone(),
                                members: vec!["You".into(), invitation.name.clone()],
                                approval_required: invitation.approval_required,
                                messages: Vec::new(),
                            }
                        });
                        if !state.conversations.iter().any(|chat| chat.id == group.id) {
                            state.conversations.insert(0, group.clone());
                        }
                        joined_group = Some(group);
                    }
                }
            }
        });
    }
    if let Some(group) = joined_group {
        conversation.set(Some(group));
    }
}

fn group_details_card(
    conversation: Signal<Option<Conversation>>,
    show_invites: Signal<bool>,
) -> AnyElement {
    let group_name = computed(move || {
        conversation
            .with(|current| current.as_ref().map(|current| current.name.clone()))
            .unwrap_or_default()
    });
    let approval_required = computed(move || {
        conversation.with(|current| {
            current
                .as_ref()
                .is_some_and(|current| current.approval_required)
        })
    });
    let ticket = computed(move || {
        conversation
            .with(|current| current.as_ref().map(|current| current.ticket.clone()))
            .unwrap_or_default()
    });
    let members = computed(move || {
        conversation
            .with(|current| current.as_ref().map(|current| current.members.clone()))
            .unwrap_or_default()
    });
    let rotated = signal(false);
    let invite_status = signal(String::new());
    let current_conversation = conversation;
    let status = rotated;
    view()
        .flex_col()
        .gap_3()
        .child(
            view()
                .items_center()
                .gap_2()
                .py_4()
                .child(Avatar::new(group_name.clone()).size(72.0))
                .child(
                    text(group_name)
                        .text_size(px(22.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(move || {
                        let count = members.get().len();
                        if count == 1 {
                            "1 member".to_owned()
                        } else {
                            format!("{count} members")
                        }
                    })
                    .text_size(px(14.0))
                    .text_color(colors::MUTED),
                ),
        )
        .child(
            view()
                .p_3()
                .gap_2()
                .rounded(px(radius::INSET))
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    text("Invite ticket")
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(ticket)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .font_family("monospace")
                        .text_color(colors::MUTED),
                )
                .child(
                    text(move || {
                        if approval_required.get() {
                            "New members need an admin's approval."
                        } else {
                            "Anyone with this ticket can join."
                        }
                    })
                    .text_size(px(13.0))
                    .text_color(colors::MUTED),
                )
                .child(
                    press()
                        .label("Rotate invite ticket")
                        .on_press(move || {
                            if let Some(id) = current_conversation.with_untracked(|current| {
                                current.as_ref().map(|current| current.id.clone())
                            }) {
                                let new_ticket = format!("nodechat://group/{id}-rotated");
                                current_conversation.update(|current| {
                                    if let Some(current) = current {
                                        current.ticket = new_ticket.clone().into();
                                    }
                                });
                                store().update(|state| {
                                    if let Some(group) =
                                        state.conversations.iter_mut().find(|group| group.id == id)
                                    {
                                        group.ticket = new_ticket.into();
                                    }
                                });
                                status.set(true);
                            }
                        })
                        .child(
                            text("Rotate invite ticket")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::ACCENT),
                        ),
                )
                .child(when(
                    move || rotated.get(),
                    || {
                        text("The previous ticket can no longer be used for new joins.")
                            .text_size(px(12.0))
                            .text_color(colors::ACCENT)
                    },
                )),
        )
        .child(
            press()
                .label("Invite people")
                .on_press(move || show_invites.update(|visible| *visible = !*visible))
                .child(
                    view()
                        .h(px(44.0))
                        .rounded(px(radius::CONTROL))
                        .items_center()
                        .justify_center()
                        .bg(colors::ACCENT_FILL)
                        .child(
                            text("Invite people")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::ON_ACCENT),
                        ),
                ),
        )
        .child(
            view().pt_2().child(
                text(move || format!("Members · {}", members.get().len()))
                    .text_size(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT),
            ),
        )
        .child(dynamic(move || {
            let rows =
                members
                    .get()
                    .into_iter()
                    .map(|member| {
                        view()
                            .min_h(px(64.0))
                            .flex_row()
                            .items_center()
                            .px_3()
                            .border_b_1()
                            .border_b_color(colors::LINE)
                            .child(ContactIdentity::new(member).subtitle(
                                text("Member").text_size(px(12.0)).text_color(colors::MUTED),
                            ))
                            .into_element()
                    })
                    .collect::<Vec<_>>();
            view().flex_col().children(rows).into_element()
        }))
        .child(when(
            move || show_invites.get(),
            move || {
                let contacts = store().with(|state| state.contacts.clone());
                if contacts.is_empty() {
                    return text("Add a contact before inviting them to this group.")
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                        .into_element();
                }
                let rows = contacts
                    .into_iter()
                    .map(|contact| {
                        let Some(group) =
                            current_conversation.with_untracked(|current| current.clone())
                        else {
                            return view().into_element();
                        };
                        let invite_status = invite_status;
                        let target = contact.id.clone();
                        let name = contact.name.clone();
                        press()
                            .label(format!("Invite {}", contact.name))
                            .on_press(move || {
                                let invitation_id = format!("{}-invite-{}", group.id, target);
                                let mut already_sent = false;
                                store().update(|state| {
                                    if let Some(chat) = state
                                        .conversations
                                        .iter_mut()
                                        .find(|chat| chat.id == target)
                                    {
                                        if chat.messages.iter().any(|message| {
                                            message.invitation.as_ref().is_some_and(|item| {
                                                item.id.to_string() == invitation_id
                                            })
                                        }) {
                                            already_sent = true;
                                            return;
                                        }
                                        chat.messages.push(Message {
                                            id: invitation_id.clone().into(),
                                            sender: Sender::Me,
                                            body: format!(
                                                "You invited them to join {}",
                                                group.name
                                            )
                                            .into(),
                                            time: "Now".into(),
                                            state: MessageState::Queued,
                                            invitation: Some(Invitation {
                                                id: invitation_id.clone().into(),
                                                kind: InvitationKind::Group,
                                                name: group.name.clone(),
                                                ticket: group.ticket.clone(),
                                                status: InvitationStatus::Pending,
                                                approval_required: group.approval_required,
                                            }),
                                            attachment: None,
                                        });
                                        chat.preview =
                                            format!("Group invitation: {}", group.name).into();
                                        chat.time = "Now".into();
                                    }
                                });
                                invite_status.set(if already_sent {
                                    format!("Invitation already sent to {name}.").into()
                                } else {
                                    format!("Invitation sent to {name}.").into()
                                });
                            })
                            .child(
                                view()
                                    .min_h(px(64.0))
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_3()
                                    .border_b_1()
                                    .border_b_color(colors::LINE)
                                    .hover(|style| style.bg(colors::RAISED))
                                    .child(
                                        view().flex_1().child(
                                            ContactIdentity::new(contact.name.clone())
                                                .online(contact.online)
                                                .verified(contact.verified)
                                                .subtitle(
                                                    text(if contact.online {
                                                        "Reachable now"
                                                    } else {
                                                        "Not reachable · invitation will queue"
                                                    })
                                                    .text_size(px(12.0))
                                                    .text_color(if contact.online {
                                                        chat::ONLINE
                                                    } else {
                                                        colors::MUTED
                                                    }),
                                                ),
                                        ),
                                    )
                                    .child(
                                        text("Invite")
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::ACCENT),
                                    ),
                            )
                            .into_element()
                    })
                    .collect::<Vec<_>>();
                view()
                    .gap_1()
                    .children(rows)
                    .child(dynamic(move || {
                        let status = invite_status.get();
                        if status.is_empty() {
                            view().into_element()
                        } else {
                            text(status)
                                .text_size(px(12.0))
                                .text_color(colors::ACCENT)
                                .into_element()
                        }
                    }))
                    .into_element()
            },
        ))
        .into_element()
}

fn group_details(
    conversation: Signal<Option<Conversation>>,
    show_invites: Signal<bool>,
    on_back: Listener<()>,
) -> AnyElement {
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .px_3()
                .pt_safe_area()
                .pb_2()
                .border_b_1()
                .border_b_color(colors::LINE)
                .child(
                    press().label("Back to group").on_press(on_back).child(
                        view()
                            .size(px(40.0))
                            .rounded(px(12.0))
                            .items_center()
                            .justify_center()
                            .rotate(deg(180.0))
                            .active(|style| style.bg(colors::RAISED))
                            .child(
                                icon()
                                    .name(IconName::Chevron)
                                    .size(20.0)
                                    .tint(colors::TEXT)
                                    .label(None),
                            ),
                    ),
                )
                .child(
                    text("Group info")
                        .text_size(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                ),
        )
        .child(
            scroll().flex_1().child(
                view()
                    .max_w(px(640.0))
                    .w_full()
                    .self_center()
                    .px_4()
                    .py_4()
                    .child(group_details_card(conversation, show_invites)),
            ),
        )
        .into_element()
}

fn contact_details(
    conversation: Signal<Option<Conversation>>,
    on_back: Listener<()>,
    on_call: Listener<()>,
    on_remove: Listener<()>,
    on_delete: Listener<()>,
) -> AnyElement {
    let confirmation = signal(None::<bool>);
    let current = conversation.with_untracked(|current| current.clone());
    let Some(current) = current else {
        return view().into_element();
    };
    let contact = store().with(|state| {
        state
            .contacts
            .iter()
            .find(|contact| contact.id == current.id)
            .cloned()
    });
    let ticket = contact
        .as_ref()
        .map(|contact| contact.ticket.clone())
        .unwrap_or_else(|| current.ticket.clone());
    let online = contact
        .as_ref()
        .map(|contact| contact.online)
        .unwrap_or(current.online);
    let verified = contact.is_some_and(|contact| contact.verified);
    let copy_ticket = ticket.clone();

    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .px_3()
                .pt_safe_area()
                .pb_2()
                .border_b_1()
                .border_b_color(colors::LINE)
                .child(
                    press()
                        .label("Back to conversation")
                        .on_press(on_back)
                        .child(
                            view()
                                .size(px(40.0))
                                .rounded(px(12.0))
                                .items_center()
                                .justify_center()
                                .rotate(deg(180.0))
                                .child(
                                    icon()
                                        .name(IconName::Chevron)
                                        .size(20.0)
                                        .tint(colors::TEXT)
                                        .label(None),
                                ),
                        ),
                )
                .child(
                    text("Contact info")
                        .flex_1()
                        .text_size(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                ),
        )
        .child(
            scroll().flex_1().child(
                view()
                    .max_w(px(560.0))
                    .w_full()
                    .self_center()
                    .px_4()
                    .py_4()
                    .gap_3()
                    .child(
                        view()
                            .items_center()
                            .gap_2()
                            .py_3()
                            .child(Avatar::new(current.name.clone()).size(72.0).online(online))
                            .child(
                                text(current.name.clone())
                                    .text_size(px(20.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::TEXT),
                            )
                            .child(
                                text(if online {
                                    "Reachable now"
                                } else {
                                    "Not reachable · messages wait"
                                })
                                .text_size(px(13.0))
                                .text_color(if online {
                                    chat::ONLINE
                                } else {
                                    colors::MUTED
                                }),
                            )
                            .child(
                                text(if verified {
                                    "Verified contact"
                                } else {
                                    "Contact"
                                })
                                .text_size(px(12.0))
                                .text_color(colors::MUTED),
                            ),
                    )
                    .child(
                        view()
                            .p_3()
                            .gap_2()
                            .rounded(px(radius::INSET))
                            .bg(colors::RAISED)
                            .border_1()
                            .border_color(colors::BORDER)
                            .child(
                                text("Invite ticket")
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT),
                            )
                            .child(
                                text(ticket)
                                    .number_of_lines(2)
                                    .text_size(px(12.0))
                                    .font_family("monospace")
                                    .text_color(colors::MUTED),
                            )
                            .child(
                                press()
                                    .label("Copy contact ticket")
                                    .on_press(move || set_clipboard(copy_ticket.to_string()))
                                    .child(
                                        text("Copy invite ticket")
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::ACCENT),
                                    ),
                            ),
                    )
                    .child(
                        press().label("Start voice call").on_press(on_call).child(
                            view()
                                .h(px(48.0))
                                .rounded(px(radius::CONTROL))
                                .items_center()
                                .justify_center()
                                .bg(colors::ACCENT_FILL)
                                .child(
                                    text("Voice call")
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::ON_ACCENT),
                                ),
                        ),
                    )
                    .child(when(
                        move || confirmation.get().is_none(),
                        move || {
                            view()
                                .gap_2()
                                .child(
                                    press()
                                        .label("Remove contact")
                                        .on_press(move || confirmation.set(Some(false)))
                                        .child(
                                            text("Remove contact · keep chat history")
                                                .text_size(px(13.0))
                                                .text_color(colors::WARN),
                                        ),
                                )
                                .child(
                                    press()
                                        .label("Delete chat")
                                        .on_press(move || confirmation.set(Some(true)))
                                        .child(
                                            text("Delete chat · keep contact")
                                                .text_size(px(13.0))
                                                .text_color(colors::WARN),
                                        ),
                                )
                        },
                    ))
                    .child(when(
                        move || confirmation.get().is_some(),
                        move || {
                            let deleting = confirmation.get().unwrap_or(false);
                            let confirm_action = if deleting {
                                on_delete.clone()
                            } else {
                                on_remove.clone()
                            };
                            view()
                                .p_3()
                                .gap_2()
                                .rounded(px(radius::INSET))
                                .bg(colors::RAISED)
                                .border_1()
                                .border_color(colors::BORDER)
                                .child(
                                    text(if deleting {
                                        "Delete this conversation and its local history?"
                                    } else {
                                        "Remove this contact? The conversation history will stay."
                                    })
                                    .text_size(px(14.0))
                                    .text_color(colors::TEXT),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .gap_2()
                                        .child(
                                            press()
                                                .label("Cancel")
                                                .on_press(move || confirmation.set(None))
                                                .child(
                                                    text("Cancel")
                                                        .text_size(px(13.0))
                                                        .text_color(colors::MUTED),
                                                ),
                                        )
                                        .child(
                                            press()
                                                .label(if deleting {
                                                    "Confirm delete chat"
                                                } else {
                                                    "Confirm remove contact"
                                                })
                                                .on_press(confirm_action)
                                                .child(
                                                    text(if deleting {
                                                        "Delete chat"
                                                    } else {
                                                        "Remove contact"
                                                    })
                                                    .text_size(px(13.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(colors::WARN),
                                                ),
                                        ),
                                )
                        },
                    )),
            ),
        )
        .into_element()
}

fn voice_call(
    conversation: Signal<Option<Conversation>>,
    muted: Signal<bool>,
    speaker: Signal<bool>,
    on_end: Listener<()>,
) -> AnyElement {
    let name = conversation.with(|current| {
        current
            .as_ref()
            .map(|current| current.name.clone())
            .unwrap_or_else(|| "Contact".into())
    });
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_4()
        .bg(colors::BACKGROUND)
        .child(Avatar::new(name.clone()).size(88.0))
        .child(
            text(name)
                .text_size(px(24.0))
                .font_weight(FontWeight::BOLD)
                .text_color(colors::TEXT),
        )
        .child(
            text("Voice call unavailable · not connected")
                .text_size(px(14.0))
                .text_color(colors::MUTED),
        )
        .child(
            view()
                .flex_row()
                .gap_4()
                .child(
                    press()
                        .label("Toggle mute")
                        .on_press(move || muted.update(|value| *value = !*value))
                        .child(
                            text(move || if muted.get() { "Unmute" } else { "Mute" })
                                .text_size(px(14.0))
                                .text_color(colors::TEXT),
                        ),
                )
                .child(
                    press()
                        .label("Toggle speaker")
                        .on_press(move || speaker.update(|value| *value = !*value))
                        .child(
                            text(move || {
                                if speaker.get() {
                                    "Speaker on"
                                } else {
                                    "Speaker"
                                }
                            })
                            .text_size(px(14.0))
                            .text_color(colors::TEXT),
                        ),
                ),
        )
        .child(
            press().label("End call").on_press(on_end).child(
                view()
                    .h(px(48.0))
                    .px_5()
                    .rounded(px(radius::CONTROL))
                    .items_center()
                    .justify_center()
                    .bg(colors::DANGER)
                    .child(
                        text("End call")
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    ),
            ),
        )
        .into_element()
}

fn first_group_prompt(
    conversation: Signal<Option<Conversation>>,
    show_group_details: Signal<bool>,
    show_group_invites: Signal<bool>,
) -> AnyElement {
    let group_name = computed(move || {
        conversation
            .with(|current| current.as_ref().map(|current| current.name.clone()))
            .unwrap_or_default()
    });
    let ticket = computed(move || {
        conversation
            .with(|current| current.as_ref().map(|current| current.ticket.clone()))
            .unwrap_or_default()
    });
    let copied = signal(false);
    let open_invites = show_group_invites;
    let open_details = show_group_details;
    let invite_ticket = ticket;
    let copy_status = copied;

    view()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .px_4()
        .child(
            view()
                .w_full()
                .max_w(px(460.0))
                .p_4()
                .gap_3()
                .rounded(px(radius::INSET))
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .child(Avatar::new(group_name.clone()).size(56.0))
                .child(
                    text("Your group is ready")
                        .text_size(px(19.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(move || {
                        format!(
                            "{} has no other members yet. Invite people to get the conversation started.",
                            group_name.get()
                        )
                    })
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .text_color(colors::MUTED),
                )
                .child(
                    text(invite_ticket)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .font_family("monospace")
                        .text_color(colors::FAINT),
                )
                .child(
                    press()
                        .label("Invite people")
                        .on_press(move || {
                            open_invites.set(true);
                            open_details.set(true);
                        })
                        .child(
                            view()
                                .h(px(44.0))
                                .rounded(px(radius::CONTROL))
                                .items_center()
                                .justify_center()
                                .bg(colors::ACCENT_FILL)
                                .child(
                                    text("Invite people")
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::ON_ACCENT),
                                ),
                        ),
                )
                .child(
                    press()
                        .label("Copy group ticket")
                        .on_press(move || {
                            set_clipboard(invite_ticket.get_untracked().to_string());
                            copy_status.set(true);
                        })
                        .child(
                            view()
                                .h(px(42.0))
                                .rounded(px(radius::CONTROL))
                                .items_center()
                                .justify_center()
                                .bg(colors::INSET)
                                .border_1()
                                .border_color(colors::BORDER)
                                .child(
                                    text("Copy invite ticket")
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::TEXT),
                                ),
                        ),
                )
                .child(when(
                    move || copied.get(),
                    || {
                        text("Ticket copied. Share it with the people you want to invite.")
                            .text_size(px(12.0))
                            .text_color(colors::ACCENT)
                    },
                )),
        )
        .into_element()
}

/// A round header button (the back control, the actions).
fn header_button(name: IconName, label: &'static str, on_press: Listener<()>) -> AnyElement {
    press()
        .label(label)
        .on_press(on_press)
        .child(
            view()
                .size(px(40.0))
                .rounded(px(12.0))
                .items_center()
                .justify_center()
                .active(|s| s.bg(colors::RAISED))
                .child(icon().name(name).size(20.0).tint(colors::TEXT).label(None)),
        )
        .into_element()
}

/// The conversation's header: a back control (when pushed), the peer, and
/// the actions. `back` is `Some` on a pushed screen, `None` in the pane.
fn header(
    name: SharedString,
    online: bool,
    kind: ConversationKind,
    member_count: usize,
    back: Option<Listener<()>>,
    on_details: Listener<()>,
) -> AnyElement {
    let name2 = name.clone();
    let status = match kind {
        ConversationKind::Direct if online => "Reachable now".to_owned(),
        ConversationKind::Direct => "Not reachable · direct delivery waits".to_owned(),
        ConversationKind::Group if member_count == 1 => "1 member · group conversation".to_owned(),
        ConversationKind::Group => format!("{member_count} members · group conversation"),
    };
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .pt_safe_area()
        .pb_2()
        .border_b_1()
        .border_b_color(colors::LINE)
        .children(back.map(|pop| {
            // The back control's chevron points left.
            press()
                .label("Back")
                .on_press(pop)
                .child(
                    view()
                        .size(px(40.0))
                        .rounded(px(12.0))
                        .items_center()
                        .justify_center()
                        .rotate(deg(180.0))
                        .active(|s| s.bg(colors::RAISED))
                        .child(
                            icon()
                                .name(IconName::Chevron)
                                .size(20.0)
                                .tint(colors::TEXT)
                                .label(None),
                        ),
                )
                .into_element()
        }))
        .child(
            view()
                .flex_1()
                .flex_row()
                .items_center()
                .gap_3()
                .px(px(2.0))
                .child(Avatar::new(name).size(36.0).online(online))
                .child(
                    view()
                        .flex_1()
                        .gap(px(1.0))
                        .child(
                            text(name2)
                                .number_of_lines(1)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                        )
                        .child(text(status).text_size(px(12.0)).text_color(if online {
                            chat::ONLINE
                        } else {
                            colors::WARN
                        })),
                ),
        )
        .child(header_button(
            IconName::More,
            if kind == ConversationKind::Group {
                "Group details"
            } else {
                "Conversation details"
            },
            on_details,
        ))
        .into_element()
}

/// The conversation content: the messages, or the empty state when none.
///
/// `back` is `Some` where the conversation is a pushed screen (its header
/// shows a back control); `None` in the wide layout's pane.
pub fn conversation(
    conversation: Signal<Option<crate::ui::data::Conversation>>,
    back: Option<Listener<()>>,
    on_new: Option<Listener<()>>,
) -> AnyElement {
    let messages = signal(conversation.with_untracked(|current| {
        current
            .as_ref()
            .map(|current| current.messages.clone())
            .unwrap_or_default()
    }));
    let list = element_ref();
    let input_ref = element_ref();
    let following = signal(true);
    let dragging = signal(false);
    effect(move || {
        if let Some(c) = conversation.get() {
            messages.set(c.messages.clone());
        }
    });
    let name = computed(move || {
        conversation
            .with(|c| c.as_ref().map(|c| c.name.clone()))
            .unwrap_or_else(|| SharedString::from("Conversation"))
    });
    let online = computed(move || conversation.with(|c| c.as_ref().is_some_and(|c| c.online)));
    let has = computed(move || conversation.with(|c| c.is_some()));
    let kind = computed(move || {
        conversation
            .with(|c| c.as_ref().map(|c| c.kind))
            .unwrap_or_default()
    });
    let member_count = computed(move || {
        conversation
            .with(|c| c.as_ref().map(|c| c.members.len()))
            .unwrap_or_default()
    });
    let first_group_message = computed(move || {
        kind.get() == ConversationKind::Group
            && member_count.get() == 1
            && messages.with(Vec::is_empty)
    });
    let show_group_details = signal(false);
    let show_group_invites = signal(false);
    let show_contact_details = signal(false);
    let call_active = signal(false);
    let call_muted = signal(false);
    let call_speaker = signal(false);
    let has_back = back.is_some();
    let chat_back = back.clone();

    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            when(has, move || {
                let chat_back = chat_back.clone();
                when(
                    move || kind.get() == ConversationKind::Group && show_group_details.get(),
                    move || {
                        let close_details =
                            Listener::from(move |_: &()| show_group_details.set(false));
                        group_details(conversation, show_group_invites, close_details)
                    },
                )
                .otherwise({
                    let details_back = chat_back.clone();
                    move || {
                        when(show_contact_details, move || {
                            let contact_id = conversation.with_untracked(|current| {
                                current.as_ref().map(|current| current.id.clone())
                            });
                            let remove_id = contact_id.clone();
                            let remove_contact = Listener::from(move |_: &()| {
                                if let Some(id) = &remove_id {
                                    store().update(|state| {
                                        state.contacts.retain(|contact| contact.id != *id);
                                    });
                                }
                                show_contact_details.set(false);
                            });
                            let delete_id = contact_id;
                            let delete_chat = Listener::from(move |_: &()| {
                                if let Some(id) = &delete_id {
                                    store().update(|state| {
                                        state.conversations.retain(|chat| chat.id != *id);
                                    });
                                }
                                conversation.set(None);
                                show_contact_details.set(false);
                                if has_back {
                                    crate::ui::nav::pop();
                                }
                            });
                            contact_details(
                                conversation,
                                Listener::from(move |_: &()| show_contact_details.set(false)),
                                Listener::from(move |_: &()| {
                                    show_contact_details.set(false);
                                    call_active.set(true);
                                }),
                                remove_contact,
                                delete_chat,
                            )
                        })
                        .otherwise({
                            let call_back = details_back.clone();
                            move || {
                                when(call_active, move || {
                                    voice_call(
                                        conversation,
                                        call_muted,
                                        call_speaker,
                                        Listener::from(move |_: &()| {
                                            call_active.set(false);
                                            show_contact_details.set(true);
                                        }),
                                    )
                                })
                                .otherwise({
                                    let chat_back = call_back.clone();
                                    move || {
                                        view()
                                            .flex_1()
                                            .child(header(
                                                name.get_untracked(),
                                                online.get_untracked(),
                                                kind.get_untracked(),
                                                member_count.get_untracked(),
                                                chat_back.clone(),
                                                Listener::from(move |_: &()| {
                                                    if kind.get() == ConversationKind::Group {
                                                        show_group_details.set(true);
                                                    } else {
                                                        show_contact_details.set(true);
                                                    }
                                                }),
                                            ))
                                            .child(
                                                view()
                                                    .flex_1()
                                                    .pb_keyboard()
                                                    .child(
                                                        when(first_group_message, move || {
                                                            first_group_prompt(
                                                                conversation,
                                                                show_group_details,
                                                                show_group_invites,
                                                            )
                                                        })
                                                        .otherwise(move || {
                                                            view()
                                                    .flex_1()
                                                    .child(
                                                        virtual_list(
                                                            move || messages.with(Vec::len),
                                                            move |at: Position| {
                                                                let message = computed(move || {
                                                                    messages.with(|m| {
                                                                        m.get(at.index()).cloned()
                                                                    })
                                                                });
                                                                dynamic(move || {
                                                                    message.get().map(|message| {
                                                                        bubble(
                                                                            message,
                                                                            store(),
                                                                            messages,
                                                                            conversation,
                                                                        )
                                                                    })
                                                                })
                                                            },
                                                        )
                                                        .keys(move || {
                                                            messages.with(|m| {
                                                                m.iter()
                                                                    .map(|m| m.id.clone())
                                                                    .collect::<Vec<_>>()
                                                            })
                                                        })
                                                        .estimated_item_size(px(64.0))
                                                        .initial_scroll_index(
                                                            messages.with_untracked(|m| {
                                                                m.len().saturating_sub(1)
                                                            }),
                                                        )
                                                        .flex_1()
                                                        .content_padding_top(px(12.0))
                                                        .content_padding_bottom(px(12.0))
                                                        .follow_end(px(40.0))
                                                        .keyboard_dismiss_mode(
                                                            KeyboardDismissMode::Interactive,
                                                        )
                                                        .element_ref(&list)
                                                        .on_scroll_drag_start(
                                                            move |_: &Point<Points>| {
                                                                dragging.set(true);
                                                            },
                                                        )
                                                        .on_scroll(move |event: &ScrollEvent| {
                                                            if dragging.get() {
                                                                following.set(
                                                                    event.offset.y
                                                                        + event.size.height
                                                                        >= event
                                                                            .content_size
                                                                            .height
                                                                            - px(40.0),
                                                                );
                                                            }
                                                        })
                                                        .on_momentum_end(
                                                            move |_: &Point<Points>| {
                                                                dragging.set(false)
                                                            },
                                                        )
                                                        .on_scroll_end(move |_: &Point<Points>| {
                                                            dragging.set(false)
                                                        }),
                                                    )
                                                    .child(when(
                                                        move || !following.get(),
                                                        move || {
                                                            press()
                                                        .label("Jump to latest message")
                                                        .on_press(move || {
                                                            following.set(true);
                                                            list.scroll_to_end(true);
                                                        })
                                                        .child(
                                                            view()
                                                                .absolute()
                                                                .right_4()
                                                                .bottom_3()
                                                                .size(px(44.0))
                                                                .rounded_full()
                                                                .items_center()
                                                                .justify_center()
                                                                .bg(colors::RAISED)
                                                                .border_1()
                                                                .border_color(colors::BORDER)
                                                                .child(
                                                                    icon()
                                                                        .name(IconName::ArrowDown)
                                                                        .size(18.0)
                                                                        .tint(colors::TEXT)
                                                                        .label(None),
                                                                ),
                                                        )
                                                        },
                                                    ))
                                                    .into_element()
                                                        }),
                                                    )
                                                    .child(delivery_status(
                                                        online.get_untracked(),
                                                        kind.get_untracked(),
                                                    ))
                                                    .child(composer(
                                                        messages,
                                                        conversation,
                                                        list,
                                                        input_ref,
                                                        following,
                                                    )),
                                            )
                                            .into_element()
                                    }
                                })
                                .into_element()
                            }
                        })
                        .into_element()
                    }
                })
                .into_element()
            })
            .otherwise(move || empty(on_new.clone())),
        )
        .into_element()
}

fn delivery_status(online: bool, kind: ConversationKind) -> AnyElement {
    match (kind, online) {
        (ConversationKind::Direct, false) => view()
            .px_4()
            .pb_1()
            .child(
                text("Not reachable · the message waits here until a direct path exists.")
                    .text_size(px(11.0))
                    .text_color(colors::MUTED),
            )
            .into_element(),
        _ => view().into_element(),
    }
}

/// The composer: a rounded field and a send button.
fn composer(
    messages: Signal<Vec<Message>>,
    conversation: Signal<Option<Conversation>>,
    list: ElementRef,
    input_ref: ElementRef,
    following: Signal<bool>,
) -> AnyElement {
    let draft = signal(String::new());
    let input_height = signal(20.0_f32);
    let attachment = signal(None::<MessageAttachment>);
    let show_attachments = signal(false);
    let can_send =
        computed(move || !draft.with(|d| d.trim().is_empty()) || attachment.with(Option::is_some));
    let send = move || {
        let text = draft.with_untracked(|d| d.trim().to_owned());
        let picked = attachment.get_untracked();
        if text.is_empty() && picked.is_none() {
            return;
        }
        let Some(conversation_id) =
            conversation.with_untracked(|current| current.as_ref().map(|c| c.id.clone()))
        else {
            return;
        };
        let next_message = messages.with_untracked(Vec::len) + 1;
        let preview = picked
            .as_ref()
            .map_or_else(|| text.clone(), |file| format!("Attachment: {}", file.name));
        let message = Message {
            id: format!("{conversation_id}-{next_message}").into(),
            sender: Sender::Me,
            body: text.clone().into(),
            time: "Now".into(),
            state: MessageState::Queued,
            invitation: None,
            attachment: picked,
        };
        conversation.update(|current| {
            if let Some(current) = current {
                current.messages.push(message.clone());
                current.preview = preview.clone().into();
                current.time = "Now".into();
                current.unread = 0;
            }
        });
        store().update(|store| {
            if let Some(current) = store
                .conversations
                .iter_mut()
                .find(|current| current.id == conversation_id)
            {
                current.messages.push(message);
                current.preview = preview.into();
                current.time = "Now".into();
                current.unread = 0;
            }
        });
        following.set(true);
        list.scroll_to_end(true);
        draft.set(String::new());
        attachment.set(None);
        show_attachments.set(false);
        input_ref.focus();
    };
    let submit = send.clone();
    view()
        .flex_col()
        .gap_1()
        .px_3()
        .pt_2()
        .pb_3()
        .border_t_1()
        .border_t_color(colors::LINE)
        .child(when(
            move || attachment.with(Option::is_some),
            move || {
                let file = attachment.with(|item| item.as_ref().map(|item| item.name.clone()));
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded(px(radius::CONTROL))
                    .bg(colors::RAISED)
                    .child(
                        icon()
                            .name(IconName::Attach)
                            .size(16.0)
                            .tint(colors::ACCENT),
                    )
                    .child(
                        text(file.unwrap_or_default())
                            .flex_1()
                            .text_size(px(13.0))
                            .text_color(colors::TEXT),
                    )
                    .child(
                        press()
                            .label("Remove attachment")
                            .on_press(move || attachment.set(None))
                            .child(text("Remove").text_size(px(12.0)).text_color(colors::MUTED)),
                    )
            },
        ))
        .child(when(
            move || show_attachments.get(),
            move || {
                let file_attachment = attachment;
                let file_menu = show_attachments;
                let file_picker = press()
                    .label("Choose file")
                    .on_press(move || {
                        file_menu.set(false);
                        let picked = pick_files(nsh_core::picker::FilePickRequest::new(false, 1));
                        spawn(async move {
                            if let Some(file) = picked.await.pop() {
                                file_attachment.set(Some(MessageAttachment {
                                    name: file.name.into(),
                                    mime: file.mime.into(),
                                    data: file.data,
                                }));
                            }
                        });
                    })
                    .child(text("File").text_size(px(14.0)).text_color(colors::TEXT));
                let image_attachment = attachment;
                let image_menu = show_attachments;
                let image_picker = press()
                    .label("Choose photo")
                    .on_press(move || {
                        image_menu.set(false);
                        let picked = pick_images(nsh_core::picker::PickRequest::default());
                        spawn(async move {
                            if let Some(file) = picked.await.pop() {
                                image_attachment.set(Some(MessageAttachment {
                                    name: file.name.into(),
                                    mime: file.mime.into(),
                                    data: file.data,
                                }));
                            }
                        });
                    })
                    .child(text("Photo").text_size(px(14.0)).text_color(colors::TEXT));
                view()
                    .flex_row()
                    .gap_4()
                    .px_3()
                    .py_2()
                    .rounded(px(radius::CONTROL))
                    .bg(colors::RAISED)
                    .children([file_picker.into_element(), image_picker.into_element()])
            },
        ))
        .child(
            view()
                .flex_row()
                .items_end()
                .gap_2()
                .child(
                    press()
                        .label("Add attachment")
                        .on_press(move || show_attachments.update(|shown| *shown = !*shown))
                        .child(
                            view()
                                .size(px(44.0))
                                .rounded_full()
                                .items_center()
                                .justify_center()
                                .bg(colors::RAISED)
                                .child(
                                    icon()
                                        .name(IconName::Attach)
                                        .size(20.0)
                                        .tint(colors::MUTED)
                                        .label(None),
                                ),
                        ),
                )
                .child(
                    view()
                        .flex_1()
                        .rounded(px(22.0))
                        .border_curve(BorderCurve::Continuous)
                        .bg(colors::RAISED)
                        .border_1()
                        .border_color(colors::BORDER)
                        .px(px(12.0))
                        .py(px(8.0))
                        .child(
                            input()
                                .element_ref(&input_ref)
                                .value(move || draft.get())
                                .multiline(true)
                                .on_content_size_change(move |size: &Size<Points>| {
                                    input_height.set(size.height.clamp(px(20.0), px(100.0)).0);
                                })
                                .submit_behavior(if is_desktop() {
                                    SubmitBehavior::Submit
                                } else {
                                    SubmitBehavior::Newline
                                })
                                .on_submit(move |_: &str| submit())
                                .on_change(move |text: &str| draft.set(text.to_owned()))
                                .placeholder("Message")
                                .placeholder_color(|| color(colors::FAINT))
                                .caret_color(|| color(chat::SENT))
                                .text_size(px(15.0))
                                .text_color(colors::TEXT)
                                .h(move || px(input_height.get()))
                                .line_height(px(20.0)),
                        ),
                )
                .child(
                    view()
                        .opacity(move || if can_send.get() { 1.0_f32 } else { 0.4 })
                        .pointer_events(move || {
                            if can_send.get() {
                                PointerEvents::Auto
                            } else {
                                PointerEvents::None
                            }
                        })
                        .child(
                            press().label("Send").on_press(send).child(
                                view()
                                    .size(px(44.0))
                                    .rounded(px(22.0))
                                    .items_center()
                                    .justify_center()
                                    .bg(chat::SENT)
                                    .child(
                                        icon()
                                            .name(IconName::Send)
                                            .size(20.0)
                                            .tint(chat::SENT_FOREGROUND)
                                            .label(None),
                                    ),
                            ),
                        ),
                ),
        )
        .into_element()
}

/// An empty conversation: a centered invitation.
fn empty(on_new: Option<Listener<()>>) -> AnyElement {
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            view()
                .size(px(64.0))
                .rounded(px(20.0))
                .items_center()
                .justify_center()
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    icon()
                        .name(IconName::Chat)
                        .size(30.0)
                        .tint(chat::SENT)
                        .label(None),
                ),
        )
        .child(
            text("Select a conversation")
                .text_size(px(17.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::TEXT),
        )
        .child(
            text("Choose a chat to read it, or start a new one.")
                .text_size(px(14.0))
                .text_color(colors::MUTED)
                .max_w(px(320.0))
                .text_center(),
        )
        .children(on_new.map(|on_new| {
            press().label("Browse contacts").on_press(on_new).child(
                view()
                    .h(px(40.0))
                    .px_3()
                    .rounded(px(radius::CONTROL))
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .bg(chat::SENT)
                    .child(
                        icon()
                            .name(IconName::Contacts)
                            .size(16.0)
                            .tint(chat::SENT_FOREGROUND)
                            .label(None),
                    )
                    .child(
                        text("Browse contacts")
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(chat::SENT_FOREGROUND),
                    ),
            )
        }))
        .into_element()
}
