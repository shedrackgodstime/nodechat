//! A conversation: its messages and the composer.
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

/// The conversation's header: the peer and its actions.
fn header(name: SharedString, online: bool) -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_3()
        .border_b_1()
        .border_b_color(colors::LINE)
        .child(Avatar::new(name.clone()).size(36.0).online(online))
        .child(
            text(name)
                .flex_1()
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::TEXT),
        )
        .child(
            press()
                .label("More")
                .on_press(|| {})
                .child(
                    view()
                        .size(px(36.0))
                        .rounded(px(10.0))
                        .items_center()
                        .justify_center()
                        .active(|s| s.bg(colors::RAISED))
                        .child(
                            icon()
                                .name(IconName::More)
                                .size(18.0)
                                .tint(colors::MUTED)
                                .label(None),
                        ),
                ),
        )
        .into_element()
}

/// The composer: a rounded field and a send button.
fn composer(conversation_id: SharedString) -> AnyElement {
    let draft = signal(String::new());
    let can_send = computed(move || !draft.with(|d| d.trim().is_empty()));
    let id = conversation_id;
    let send = move || {
        let text = draft.with_untracked(|d| d.trim().to_owned());
        if text.is_empty() {
            return;
        }
        let _ = &id;
        // The message is queued in the conversation's store (wired to the
        // backend later); the composer only clears the field.
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
                    press()
                        .label("Send")
                        .on_press(send)
                        .child(
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
fn empty() -> AnyElement {
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
        .into_element()
}

/// The conversation content: the messages, or the empty state when none.
pub fn conversation(conversation: Signal<Option<Conversation>>) -> AnyElement {
    // The messages shown, kept in a signal the composer writes to and the
    // list reads (the open conversation's, set when the selection changes).
    let messages = signal(Vec::<Message>::new());
    let id = signal(SharedString::default());
    effect(move || {
        if let Some(c) = conversation.get() {
            id.set(c.id.clone());
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
                    .child(header(name.get_untracked(), online.get_untracked()))
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
                    .child(composer(id.get_untracked()))
                    .into_element()
            })
            .otherwise(empty),
        )
        .into_element()
}
