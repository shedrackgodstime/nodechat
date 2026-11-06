//! People, connection requests, and ticket-based contact/group flows.

use nsh_base::input;
use nsh_core::{
    element::{AnyElement, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::{ConnectionRequest, Contact, Conversation, ConversationKind, InvitationKind, Store},
    icons::icon,
    press,
    theme::{chat, colors, radius},
    widgets::{ContactIdentity, PageHeader},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Home,
    Start,
    Connect,
    GroupDetails,
    JoinGroup,
    Requests,
}

/// A secondary label used for connection and group request cards.
fn kind_label(kind: InvitationKind) -> &'static str {
    match kind {
        InvitationKind::Contact => "Contact request",
        InvitationKind::Group => "Group invitation",
    }
}

fn action_button(
    label: impl Into<SharedString>,
    action: impl Fn() + 'static,
    primary: bool,
) -> AnyElement {
    let label = label.into();
    press()
        .label(label.clone())
        .on_press(action)
        .child(
            view()
                .h(px(38.0))
                .px_3()
                .rounded(px(radius::CONTROL))
                .items_center()
                .justify_center()
                .bg(if primary {
                    colors::ACCENT_FILL
                } else {
                    colors::RAISED
                })
                .border_1()
                .border_color(if primary {
                    colors::ACCENT_FILL
                } else {
                    colors::BORDER
                })
                .child(
                    text(label)
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if primary {
                            colors::ON_ACCENT
                        } else {
                            colors::TEXT
                        }),
                ),
        )
        .into_element()
}

fn section_title(title: &'static str, detail: &'static str) -> AnyElement {
    view()
        .px_4()
        .pt_5()
        .pb_2()
        .gap(px(3.0))
        .child(
            text(title)
                .text_size(px(17.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::TEXT),
        )
        .child(text(detail).text_size(px(13.0)).text_color(colors::MUTED))
        .into_element()
}

fn text_field(label: &'static str, value: Signal<String>, placeholder: &'static str) -> AnyElement {
    view()
        .gap(px(6.0))
        .child(
            text(label)
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::MUTED),
        )
        .child(
            view()
                .min_h(px(44.0))
                .flex_row()
                .items_center()
                .px_3()
                .rounded(px(radius::CONTROL))
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    input()
                        .label(label)
                        .value(move || value.get())
                        .on_change(move |text: &str| value.set(text.to_owned()))
                        .placeholder(placeholder)
                        .placeholder_color(|| color(colors::FAINT))
                        .caret_color(|| color(colors::ACCENT))
                        .text_size(px(15.0))
                        .text_color(colors::TEXT)
                        .flex_1(),
                ),
        )
        .into_element()
}

fn information_card(title: impl Into<SharedString>, detail: impl Into<SharedString>) -> AnyElement {
    let title = title.into();
    let detail = detail.into();
    view()
        .mx_4()
        .p_3()
        .gap(px(6.0))
        .rounded(px(radius::INSET))
        .bg(colors::RAISED)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            text(title)
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::TEXT),
        )
        .child(
            text(detail)
                .text_size(px(13.0))
                .line_height(px(19.0))
                .text_color(colors::MUTED),
        )
        .into_element()
}

fn request_card(
    request: ConnectionRequest,
    store: Signal<Store>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    let name = request.name.to_string();
    let accept_label = if request.kind == InvitationKind::Group {
        format!("Join {}", name)
    } else {
        format!("Accept {}", name)
    };
    let decline_label = format!("Decline {}", name);
    let incoming = request.incoming;
    let card = view()
        .mx_4()
        .mb_2()
        .p_3()
        .gap_2()
        .rounded(px(radius::INSET))
        .bg(colors::RAISED)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            view().flex_row().items_center().gap_3().child(
                ContactIdentity::new(request.name.clone()).subtitle(
                    text(kind_label(request.kind))
                        .text_size(px(12.0))
                        .text_color(colors::ACCENT),
                ),
            ),
        )
        .child(
            text(request.note.clone())
                .text_size(px(13.0))
                .text_color(colors::MUTED),
        )
        .child(
            text(request.ticket.clone())
                .number_of_lines(1)
                .text_size(px(11.0))
                .text_color(colors::FAINT)
                .font_family("monospace"),
        );
    if incoming {
        let accept_store = store;
        let request_to_accept = request.clone();
        let open_group = on_open;
        let decline_store = store;
        let id = request.id.clone();
        card.child(
            view()
                .flex_row()
                .gap_2()
                .child(action_button(
                    accept_label,
                    move || {
                        if let Some(conversation) =
                            accept_request(&accept_store, &request_to_accept)
                        {
                            open_group(&conversation);
                        }
                    },
                    true,
                ))
                .child(action_button(
                    decline_label,
                    move || {
                        decline_store.update(|state| {
                            state.requests.retain(|item| item.id != id);
                        })
                    },
                    false,
                )),
        )
        .into_element()
    } else {
        let waiting = if request.kind == InvitationKind::Group {
            "Waiting for a group admin to approve this request."
        } else {
            "Waiting for the peer to accept this connection request."
        };
        card.child(text(waiting).text_size(px(12.0)).text_color(colors::WARN))
            .child(action_button(
                if request.kind == InvitationKind::Group {
                    "Cancel join request"
                } else {
                    "Cancel request"
                },
                move || {
                    store.update(|state| {
                        state.requests.retain(|item| item.id != request.id);
                    })
                },
                false,
            ))
            .into_element()
    }
}

fn pending_contact_row(request: ConnectionRequest, store: Signal<Store>) -> AnyElement {
    let name = request.name.to_string();
    let incoming = request.incoming;
    let mut row = view()
        .mx_4()
        .mb_1()
        .min_h(px(64.0))
        .px_3()
        .py_2()
        .flex_row()
        .items_center()
        .gap_3()
        .rounded(px(radius::INSET))
        .bg(colors::RAISED)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            view().flex_1().child(
                ContactIdentity::new(request.name.clone()).subtitle(
                    text(if incoming {
                        "Pending · received"
                    } else {
                        "Pending · sent"
                    })
                    .text_size(px(12.0))
                    .text_color(colors::WARN),
                ),
            ),
        );
    if incoming {
        let accept_store = store;
        let request_to_accept = request.clone();
        let decline_store = store;
        let id = request.id.clone();
        row = row
            .child(action_button(
                format!("Accept {name}"),
                move || {
                    accept_request(&accept_store, &request_to_accept);
                },
                true,
            ))
            .child(action_button(
                format!("Decline {name}"),
                move || {
                    decline_store.update(|state| {
                        state.requests.retain(|item| item.id != id);
                    })
                },
                false,
            ));
    } else {
        let id = request.id.clone();
        row = row.child(action_button(
            format!("Cancel {name} request"),
            move || {
                store.update(|state| {
                    state.requests.retain(|item| item.id != id);
                })
            },
            false,
        ));
    }
    row.into_element()
}

fn group_conversation(
    id: String,
    name: String,
    ticket: String,
    members: Vec<String>,
    approval_required: bool,
) -> Conversation {
    Conversation {
        id: id.into(),
        name: name.clone().into(),
        preview: "Group is ready. Invite people to join.".into(),
        time: "Now".into(),
        unread: 0,
        online: true,
        kind: ConversationKind::Group,
        ticket: ticket.into(),
        members: members.into_iter().map(Into::into).collect(),
        approval_required,
        messages: Vec::new(),
    }
}

fn accept_request(store: &Signal<Store>, request: &ConnectionRequest) -> Option<Conversation> {
    let id = request.id.to_string();
    let name = request.name.to_string();
    let ticket = request.ticket.to_string();
    let mut joined_group = None;
    store.update(|state| {
        if request.kind == InvitationKind::Contact {
            let contact_id = id.trim_start_matches("request-").to_owned();
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
                        name: name.clone().into(),
                        preview: "Connection accepted. Waiting for the peer.".into(),
                        time: "Now".into(),
                        unread: 0,
                        online: false,
                        kind: ConversationKind::Direct,
                        ticket: ticket.clone().into(),
                        members: Vec::new(),
                        approval_required: false,
                        messages: Vec::new(),
                    },
                );
            }
        } else {
            let existing = state
                .conversations
                .iter()
                .find(|chat| {
                    chat.kind == ConversationKind::Group
                        && (chat.ticket.to_string() == ticket || chat.name.to_string() == name)
                })
                .cloned();
            let conversation = existing.unwrap_or_else(|| {
                let group_id = format!("joined-{}", id.trim_start_matches("request-"));
                group_conversation(
                    group_id,
                    name.clone(),
                    ticket.clone(),
                    vec!["You".into(), name.clone()],
                    true,
                )
            });
            if !state
                .conversations
                .iter()
                .any(|chat| chat.id == conversation.id)
            {
                state.conversations.insert(0, conversation.clone());
            }
            joined_group = Some(conversation);
        }
        state.requests.retain(|item| item.id.to_string() != id);
    });
    joined_group
}

fn flow_panel(
    flow: Flow,
    store: Signal<Store>,
    group_name: Signal<String>,
    group_approval: Signal<bool>,
    result: Signal<String>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    match flow {
        Flow::Home | Flow::Start | Flow::Requests => view().into_element(),
        Flow::Connect => {
            let ticket = signal(String::new());
            let note = signal(String::new());
            let preview = signal(false);
            let current_ticket = ticket.clone();
            let current_preview = preview.clone();
            let current_store = store;
            let current_result = result;
            let preview_button = action_button(
                "Preview ticket",
                move || current_preview.set(!current_ticket.get_untracked().trim().is_empty()),
                true,
            );
            view()
                .mx_4()
                .p_3()
                .gap_3()
                .rounded(px(radius::INSET))
                .bg(colors::SURFACE)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    text("Connect with a ticket")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(text_field("Contact ticket or link", ticket.clone(), "Paste or type a ticket"))
                .child(text_field("Optional message", note.clone(), "Tell them how you know each other"))
                .child(preview_button)
                .child(when(
                    move || preview.get(),
                    move || {
                        let ticket_value = ticket.get();
                        let send_ticket = ticket_value.clone();
                        let send_note = note.get();
                        let send_store = current_store;
                        let send_result = current_result;
                        view()
                            .p_3()
                            .gap_2()
                            .rounded(px(radius::INSET))
                            .bg(colors::INSET)
                            .child(
                                text(if ticket_value.to_lowercase().contains("mira") {
                                    "Mira · contact preview"
                                } else {
                                    "New peer · ticket preview"
                                })
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                            )
                            .child(
                                text("Review the destination before sending a request.")
                                    .text_size(px(12.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(action_button(
                                "Send connection request",
                                move || {
                                    let request_id =
                                        format!("outgoing-{}", send_store.with_untracked(|s| s.requests.len() + 1));
                                    send_store.update(|state| {
                                        state.requests.push(ConnectionRequest {
                                            id: request_id.into(),
                                            name: if send_ticket.to_lowercase().contains("mira") {
                                                "Mira".into()
                                            } else {
                                                "New peer".into()
                                            },
                                            ticket: send_ticket.clone().into(),
                                            note: if send_note.trim().is_empty() {
                                                "Connection request sent from your device.".into()
                                            } else {
                                                send_note.clone().into()
                                            },
                                            kind: InvitationKind::Contact,
                                            incoming: false,
                                        });
                                    });
                                    send_result.set(
                                        "Request saved on this device. The peer must accept before a connection is established."
                                            .into(),
                                    );
                                },
                                true,
                            ))
                            .into_element()
                    },
                ))
                .into_element()
        }
        Flow::GroupDetails => {
            let group_store = store;
            let group_result = result;
            let open_group = on_open;
            view()
                .mx_4()
                .p_3()
                .gap_3()
                .rounded(px(radius::INSET))
                .bg(colors::SURFACE)
                .border_1()
                .border_color(colors::BORDER)
                .child(text_field("Group name", group_name.clone(), "e.g. Saturday study"))
                .child(
                    press()
                        .label("Require approval for ticket joins")
                        .on_press(move || group_approval.update(|required| *required = !*required))
                        .child(
                            view()
                                .h(px(42.0))
                                .px_3()
                                .rounded(px(radius::CONTROL))
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .bg(colors::INSET)
                                .child(
                                    text(move || {
                                        if group_approval.get() {
                                            "✓  Approve group join requests"
                                        } else {
                                            "○  Let ticket holders join directly"
                                        }
                                    })
                                    .text_size(px(13.0))
                                    .text_color(colors::TEXT),
                                ),
                        ),
                )
                .child(action_button(
                    "Create group",
                    move || {
                        let name = group_name.get_untracked().trim().to_owned();
                        if name.is_empty() {
                            group_result.set("Add a group name before creating it.".into());
                            return;
                        }
                        let (id, ticket) = group_store.with_untracked(|state| {
                            let id = format!("group-{}", state.conversations.len() + 1);
                            let ticket = format!("nodechat://group/{id}-invite");
                            (id, ticket)
                        });
                        let approval_required = group_approval.get_untracked();
                        let conversation = group_conversation(
                            id.clone(),
                            name.clone(),
                            ticket.clone(),
                            vec!["You".into()],
                            approval_required,
                        );
                        group_store.update(|state| {
                            state.conversations.insert(0, conversation.clone());
                        });
                        group_result.set(format!(
                            "Group created on this device. Share this ticket to invite more people: {ticket}"
                        ));
                        open_group(&conversation);
                    },
                    true,
                ))
                .into_element()
        }
        Flow::JoinGroup => {
            let ticket = signal(String::new());
            let preview = signal(false);
            let current_ticket = ticket.clone();
            let current_preview = preview.clone();
            let preview_button = action_button(
                "Preview group ticket",
                move || current_preview.set(!current_ticket.get_untracked().trim().is_empty()),
                true,
            );
            let group_ticket = ticket.clone();
            let group_store = store;
            let group_result = result;
            view()
                .mx_4()
                .p_3()
                .gap_3()
                .rounded(px(radius::INSET))
                .bg(colors::SURFACE)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    text("Join a group")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(text_field("Group ticket or link", ticket.clone(), "Paste a group ticket"))
                .child(preview_button)
                .child(when(
                    move || preview.get(),
                    move || {
                        let ticket = group_ticket.get();
                        let join_ticket = ticket.clone();
                        let join_store = group_store;
                        let join_result = group_result;
                        view()
                            .p_3()
                            .gap_2()
                            .rounded(px(radius::INSET))
                            .bg(colors::INSET)
                            .child(
                                text("Study Circle · group preview")
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT),
                            )
                            .child(
                                text("Shared by Ada · 5 members · approval required")
                                    .text_size(px(12.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(
                                text(ticket)
                                    .number_of_lines(1)
                                    .text_size(px(11.0))
                                    .font_family("monospace")
                                    .text_color(colors::FAINT),
                            )
                            .child(action_button(
                                "Request to join",
                                move || {
                                    let ticket = join_ticket.clone();
                                    join_store.update(|state| {
                                        let exists = state.requests.iter().any(|request| request.ticket.to_string() == ticket);
                                        if !exists {
                                            state.requests.push(ConnectionRequest {
                                                id: format!("outgoing-group-{}", state.requests.len() + 1).into(),
                                                name: "Study Circle".into(),
                                                ticket: ticket.clone().into(),
                                                note: "Waiting for a group admin to approve your request.".into(),
                                                kind: InvitationKind::Group,
                                                incoming: false,
                                            });
                                        }
                                    });
                                    join_result.set(
                                        "Join request saved. You are not a member until an admin approves it."
                                            .into(),
                                    );
                                },
                                true,
                            ))
                            .into_element()
                    },
                ))
                .into_element()
        }
    }
}

/// The saved people directory.
pub fn contacts_page(
    store: Signal<Store>,
    back: Option<Listener<()>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
    on_new: impl Fn() + Clone + 'static,
) -> AnyElement {
    lifecycle_page(store, back, on_open, on_new, Flow::Home)
}

/// The focused new-conversation chooser.
pub fn new_chat_page(
    store: Signal<Store>,
    back: Option<Listener<()>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    lifecycle_page(store, back, on_open, || {}, Flow::Start)
}

fn lifecycle_page(
    store: Signal<Store>,
    back: Option<Listener<()>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
    on_new: impl Fn() + Clone + 'static,
    initial_flow: Flow,
) -> AnyElement {
    let flow = signal(initial_flow);
    let group_name = signal(String::new());
    let group_approval = signal(true);
    let result = signal(String::new());
    let search = signal(String::new());

    scroll()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .max_w(px(760.0))
                .w_full()
                .self_center()
                .child(dynamic(move || {
                    let current_flow = flow.get();
                    match current_flow {
                        Flow::Home => people_home(
                            store,
                            flow,
                            search,
                            back.clone(),
                            on_open.clone(),
                            on_new.clone(),
                        ),
                        Flow::Start => start_page(flow, group_name, group_approval, back.clone()),
                        Flow::Requests => requests_page(store, flow, initial_flow, on_open.clone()),
                        _ => {
                            let title = match current_flow {
                                Flow::Connect => "Connect by ticket",
                                Flow::GroupDetails => "Create a group",
                                Flow::JoinGroup => "Join a group",
                                Flow::Home | Flow::Start | Flow::Requests => unreachable!(),
                            };
                            let return_flow = flow;
                            let local_back =
                                Listener::from(move |_: &()| return_flow.set(initial_flow));
                            view()
                                .child(PageHeader::new(title).back(Some(local_back)))
                                .child(flow_panel(
                                    current_flow,
                                    store,
                                    group_name,
                                    group_approval,
                                    result,
                                    on_open.clone(),
                                ))
                                .child(dynamic(move || {
                                    let message = result.get();
                                    if message.is_empty() {
                                        view().into_element()
                                    } else {
                                        information_card("Status", message)
                                    }
                                }))
                                .into_element()
                        }
                    }
                })),
        )
        .into_element()
}

fn start_page(
    flow: Signal<Flow>,
    group_name: Signal<String>,
    group_approval: Signal<bool>,
    back: Option<Listener<()>>,
) -> AnyElement {
    let connect = flow;
    let create = flow;
    let join = flow;
    let start_group_name = group_name;
    let start_group_approval = group_approval;
    view()
        .child(PageHeader::new("New conversation").back(back))
        .child(section_title(
            "Start with",
            "Choose how you want to connect.",
        ))
        .child(
            view()
                .mx_4()
                .gap_2()
                .child(action_button(
                    "Connect by ticket",
                    move || connect.set(Flow::Connect),
                    true,
                ))
                .child(action_button(
                    "Create a group",
                    move || {
                        start_group_name.set(String::new());
                        start_group_approval.set(true);
                        create.set(Flow::GroupDetails);
                    },
                    false,
                ))
                .child(action_button(
                    "Join with a group ticket",
                    move || join.set(Flow::JoinGroup),
                    false,
                )),
        )
        .child(section_title(
            "Your invite ticket",
            "Share this ticket so someone can request a connection.",
        ))
        .child(information_card(
            "This device",
            "nodechat://contact/you-4k7p2v · Ticket sharing is simulated in this preview.",
        ))
        .into_element()
}

fn people_home(
    store: Signal<Store>,
    flow: Signal<Flow>,
    search: Signal<String>,
    back: Option<Listener<()>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
    on_new: impl Fn() + Clone + 'static,
) -> AnyElement {
    let requests_flow = flow;
    let start = on_new;
    let group_open = on_open.clone();
    let contact_open = on_open;
    view()
        .child(
            PageHeader::new("People").back(back).right(
                view()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(action_button(
                        "Requests",
                        move || requests_flow.set(Flow::Requests),
                        false,
                    ))
                    .child(action_button("Start conversation", move || start(), true)),
            ),
        )
        .child(
            view().px_4().pt_3().child(
                view()
                    .h(px(42.0))
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .rounded(px(radius::CONTROL))
                    .bg(colors::RAISED)
                    .border_1()
                    .border_color(colors::BORDER)
                    .child(
                        icon()
                            .name(IconName::Search)
                            .size(16.0)
                            .tint(colors::FAINT)
                            .label(None),
                    )
                    .child(
                        input()
                            .label("Search people")
                            .value(move || search.get())
                            .on_change(move |text: &str| search.set(text.to_owned()))
                            .placeholder("Search people")
                            .placeholder_color(|| color(colors::FAINT))
                            .caret_color(|| color(colors::ACCENT))
                            .text_size(px(15.0))
                            .text_color(colors::TEXT)
                            .flex_1(),
                    ),
            ),
        )
        .child(section_title("Groups", "Your group conversations."))
        .child(dynamic(move || {
            let query = search.get().trim().to_lowercase();
            let groups = store.with(|state| {
                state
                    .conversations
                    .iter()
                    .filter(|chat| {
                        chat.kind == ConversationKind::Group
                            && (query.is_empty()
                                || chat.name.to_string().to_lowercase().contains(&query))
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            });
            let rows = groups
                .into_iter()
                .map(|chat| {
                    let open = group_open.clone();
                    let selected = chat.clone();
                    conversation_row(chat, false, Listener::from(move |_: &()| open(&selected)))
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                view()
                    .mx_4()
                    .child(information_card(
                        "No groups found",
                        "Groups you belong to appear here.",
                    ))
                    .into_element()
            } else {
                view().flex_col().children(rows).into_element()
            }
        }))
        .child(section_title(
            "Contacts",
            "Connected and pending connections.",
        ))
        .child(dynamic(move || {
            let query = search.get().trim().to_lowercase();
            let (contacts, pending) = store.with(|state| {
                let contacts = state
                    .contacts
                    .iter()
                    .filter(|contact| {
                        query.is_empty() || contact.name.to_string().to_lowercase().contains(&query)
                    })
                    .map(|contact| {
                        let chat = state
                            .conversations
                            .iter()
                            .find(|chat| chat.id == contact.id)
                            .cloned()
                            .unwrap_or_else(|| Conversation {
                                id: contact.id.clone(),
                                name: contact.name.clone(),
                                preview: "Start a conversation".into(),
                                time: "".into(),
                                unread: 0,
                                online: contact.online,
                                kind: ConversationKind::Direct,
                                ticket: contact.ticket.clone(),
                                members: Vec::new(),
                                approval_required: false,
                                messages: Vec::new(),
                            });
                        (chat, contact.verified)
                    })
                    .collect::<Vec<_>>();
                let pending = state
                    .requests
                    .iter()
                    .filter(|request| {
                        request.kind == InvitationKind::Contact
                            && (query.is_empty()
                                || request.name.to_string().to_lowercase().contains(&query))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                (contacts, pending)
            });
            let mut rows = contacts
                .into_iter()
                .map(|(chat, verified)| {
                    let open = contact_open.clone();
                    let selected = chat.clone();
                    let store = store;
                    conversation_row(
                        chat,
                        verified,
                        Listener::from(move |_: &()| {
                            store.update(|state| {
                                if !state
                                    .conversations
                                    .iter()
                                    .any(|conversation| conversation.id == selected.id)
                                {
                                    state.conversations.push(selected.clone());
                                }
                            });
                            open(&selected);
                        }),
                    )
                })
                .collect::<Vec<_>>();
            rows.extend(
                pending
                    .into_iter()
                    .map(|request| pending_contact_row(request, store)),
            );
            if rows.is_empty() {
                view()
                    .mx_4()
                    .child(information_card(
                        if query.is_empty() {
                            "No contacts yet"
                        } else {
                            "No matching contacts"
                        },
                        "Connected peers appear here.",
                    ))
                    .into_element()
            } else {
                view().flex_col().children(rows).into_element()
            }
        }))
        .into_element()
}

fn requests_page(
    store: Signal<Store>,
    flow: Signal<Flow>,
    return_to: Flow,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    let return_flow = flow;
    let local_back = Listener::from(move |_: &()| return_flow.set(return_to));
    view()
        .child(PageHeader::new("Requests").back(Some(local_back)))
        .child(section_title(
            "Pending",
            "Review each request before adding someone.",
        ))
        .child(dynamic(move || {
            let requests = store.with(|state| state.requests.clone());
            if requests.is_empty() {
                return view()
                    .mx_4()
                    .child(information_card(
                        "No pending requests",
                        "New contact and group requests will appear here.",
                    ))
                    .into_element();
            }
            let incoming = requests
                .iter()
                .filter(|request| request.incoming)
                .cloned()
                .map(|request| request_card(request, store, on_open.clone()))
                .collect::<Vec<_>>();
            let outgoing = requests
                .iter()
                .filter(|request| !request.incoming)
                .cloned()
                .collect::<Vec<_>>();
            let mut cards = incoming;
            if !outgoing.is_empty() {
                cards.push(section_title(
                    "Sent",
                    "These requests stay pending until the other side responds.",
                ));
                cards.extend(
                    outgoing
                        .into_iter()
                        .map(|request| request_card(request, store, on_open.clone())),
                );
            }
            view().flex_col().children(cards).into_element()
        }))
        .into_element()
}

/// A known peer row.
fn conversation_row(contact: Conversation, verified: bool, on_open: Listener<()>) -> AnyElement {
    press()
        .label(format!("Open chat with {}", contact.name))
        .on_press(on_open)
        .child(
            view()
                .mx_4()
                .mb_1()
                .min_h(px(64.0))
                .flex_row()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .rounded(px(radius::INSET))
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .hover(|style| style.bg(colors::SURFACE))
                .child(
                    view().flex_1().child(
                        ContactIdentity::new(contact.name.clone())
                            .online(contact.online && contact.kind == ConversationKind::Direct)
                            .verified(verified)
                            .subtitle(
                                text(if contact.kind == ConversationKind::Group {
                                    "Group conversation"
                                } else if contact.online {
                                    "Reachable now"
                                } else {
                                    "Not reachable · direct messages wait"
                                })
                                .text_size(px(12.0))
                                .text_color(
                                    if contact.kind == ConversationKind::Group {
                                        colors::ACCENT
                                    } else if contact.online {
                                        chat::ONLINE
                                    } else {
                                        colors::MUTED
                                    },
                                ),
                            ),
                    ),
                )
                .child(
                    view().child(
                        icon()
                            .name(IconName::Chevron)
                            .size(16.0)
                            .tint(colors::FAINT)
                            .label(None),
                    ),
                ),
        )
        .into_element()
}
