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
    element::{AnyElement, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::{Conversation, Message, MessageState, Sender},
    icons::icon,
    press,
    shell::store,
    theme::{chat, colors, radius},
    widgets::{Avatar, state_mark},
};

/// A message bubble: the user's on the right (accent), a peer's on the left.
fn bubble(message: Message) -> AnyElement {
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
                .child(
                    text(message.body.clone())
                        .select_text()
                        .text_size(px(15.0))
                        .line_height(px(21.0))
                        .text_color(if mine {
                            chat::SENT_FOREGROUND
                        } else {
                            chat::RECEIVED_FOREGROUND
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
fn header(name: SharedString, online: bool, back: Option<Listener<()>>) -> AnyElement {
    let name2 = name.clone();
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
                        .child(
                            text(if online { "Online" } else { "Offline" })
                                .text_size(px(12.0))
                                .text_color(if online { chat::ONLINE } else { colors::FAINT }),
                        ),
                ),
        )
        .child(header_button(
            IconName::More,
            "More",
            Listener::from(|_: &()| {}),
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
    let messages = signal(Vec::<Message>::new());
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

    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            when(has, move || {
                view()
                    .flex_1()
                    .child(header(
                        name.get_untracked(),
                        online.get_untracked(),
                        back.clone(),
                    ))
                    .child(
                        virtual_list(
                            move || messages.with(Vec::len),
                            move |at: Position| {
                                let message =
                                    computed(move || messages.with(|m| m.get(at.index()).cloned()));
                                dynamic(move || message.get().map(bubble))
                            },
                        )
                        .keys(move || {
                            messages.with(|m| m.iter().map(|m| m.id.clone()).collect::<Vec<_>>())
                        })
                        .estimated_item_size(px(64.0))
                        .flex_1()
                        .content_padding_top(px(12.0))
                        .content_padding_bottom(px(12.0))
                        .follow_end(px(40.0)),
                    )
                    .child(composer(messages, conversation))
                    .into_element()
            })
            .otherwise(move || empty(on_new.clone())),
        )
        .into_element()
}

/// The composer: a rounded field and a send button.
fn composer(
    messages: Signal<Vec<Message>>,
    conversation: Signal<Option<Conversation>>,
) -> AnyElement {
    let draft = signal(String::new());
    let can_send = computed(move || !draft.with(|d| d.trim().is_empty()));
    let send = move || {
        let text = draft.with_untracked(|d| d.trim().to_owned());
        if text.is_empty() {
            return;
        }
        let Some(conversation_id) =
            conversation.with_untracked(|current| current.as_ref().map(|c| c.id.clone()))
        else {
            return;
        };
        let next_message = messages.with_untracked(Vec::len) + 1;
        let message = Message {
            id: format!("{conversation_id}-{next_message}").into(),
            sender: Sender::Me,
            body: text.clone().into(),
            time: "Now".into(),
            state: MessageState::Queued,
        };
        conversation.update(|current| {
            if let Some(current) = current {
                current.messages.push(message.clone());
                current.preview = text.clone().into();
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
                current.preview = text.into();
                current.time = "Now".into();
                current.unread = 0;
            }
        });
        draft.set(String::new());
    };
    let submit = send.clone();
    view()
        .flex_row()
        .items_end()
        .gap_2()
        .p_3()
        .border_t_1()
        .border_t_color(colors::LINE)
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
                        .value(move || draft.get())
                        .multiline(true)
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
                        .text_color(colors::TEXT),
                ),
        )
        .child(
            view()
                .opacity(move || if can_send.get() { 1.0_f32 } else { 0.4 })
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
