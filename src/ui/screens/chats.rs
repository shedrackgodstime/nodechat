//! The chats list: the app's root screen, the conversations newest first.

use nsh_base::input;
use nsh_core::{
    element::{AnyElement, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::{Conversation, MessageState, Store},
    icons::icon,
    press,
    theme::{chat, colors, radius},
    widgets::{Avatar, state_mark},
};

/// The list header: the title and the new-chat action.
fn header(title: impl IntoReactive<SharedString>, on_new: impl Into<Listener<()>>) -> AnyElement {
    let on_new: Listener<()> = on_new.into();
    press()
        .label("New chat")
        .on_press(on_new)
        .child(
            view()
                .h(px(56.0))
                .flex_row()
                .items_center()
                .gap_3()
                .px_4()
                .child(
                    text(title)
                        .flex_1()
                        .text_size(px(24.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    view()
                        .size(px(36.0))
                        .rounded(px(10.0))
                        .items_center()
                        .justify_center()
                        .bg(colors::RAISED)
                        .border_1()
                        .border_color(colors::BORDER)
                        .child(
                            icon()
                                .name(IconName::Plus)
                                .size(18.0)
                                .tint(colors::TEXT)
                                .label(None),
                        ),
                ),
        )
        .into_element()
}

/// A search field.
fn search(query: Signal<String>) -> AnyElement {
    view()
        .px_4()
        .pb_2()
        .child(
            view()
                .h(px(40.0))
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .rounded(px(radius::CONTROL))
                .border_curve(BorderCurve::Continuous)
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
                        .value(move || query.get())
                        .on_change(move |text: &str| query.set(text.to_owned()))
                        .placeholder("Search")
                        .placeholder_color(|| color(colors::FAINT))
                        .caret_color(|| color(chat::ONLINE))
                        .text_size(px(15.0))
                        .text_color(colors::TEXT)
                        .flex_1(),
                ),
        )
        .into_element()
}

/// A conversation's row: avatar, name, preview, time and unread count.
fn chat_row(conversation: Conversation, on_open: Listener<()>, selected: bool) -> AnyElement {
    let unread = conversation.unread;
    let online = conversation.online;
    view()
        .h(px(72.0))
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .cursor_pointer()
        .role(AccessibilityRole::Button)
        .label(conversation.name.clone())
        .accessibility_state(move || AccessibilityState {
            selected,
            ..AccessibilityState::default()
        })
        .style_when(selected, |s| s.bg(colors::SURFACE))
        .hover(|s| s.bg(colors::SURFACE))
        .active(|s| s.bg(colors::SURFACE))
        .on_press(on_open)
        .child(
            Avatar::new(conversation.name.clone())
                .size(48.0)
                .online(online),
        )
        .child(
            view()
                .flex_1()
                .gap(px(3.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            text(conversation.name.clone())
                                .flex_1()
                                .number_of_lines(1)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                        )
                        .child(
                            text(conversation.time.clone())
                                .text_size(px(12.0))
                                .text_color(colors::FAINT)
                                .tabular_nums(),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(state_mark(
                            conversation
                                .messages
                                .last()
                                .map(|m| m.state)
                                .unwrap_or(MessageState::Read),
                        ))
                        .child(
                            text(conversation.preview.clone())
                                .flex_1()
                                .number_of_lines(1)
                                .text_size(px(14.0))
                                .text_color(colors::MUTED),
                        )
                        .children((unread > 0).then(|| {
                            view()
                                .min_w(px(20.0))
                                .h(px(20.0))
                                .rounded_full()
                                .items_center()
                                .justify_center()
                                .bg(chat::ONLINE)
                                .child(
                                    text(unread.to_string())
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(AdaptiveColor::from(nsh_core::element::rgb(
                                            0xffffff,
                                        ))),
                                )
                        })),
                ),
        )
        .into_element()
}

/// The list: one row per conversation.
fn matching_conversations(store: Signal<Store>, query: Signal<String>) -> Vec<Conversation> {
    let query = query.get().trim().to_lowercase();
    store.with(|s| {
        s.conversations
            .iter()
            .filter(|conversation| {
                query.is_empty()
                    || conversation
                        .name
                        .to_string()
                        .to_lowercase()
                        .contains(&query)
                    || conversation
                        .preview
                        .to_string()
                        .to_lowercase()
                        .contains(&query)
            })
            .cloned()
            .collect()
    })
}

fn list(
    store: Signal<Store>,
    query: Signal<String>,
    selected: Signal<Option<Conversation>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    virtual_list(
        move || matching_conversations(store, query).len(),
        move |at: Position| {
            let conversation = computed(move || {
                matching_conversations(store, query)
                    .get(at.index())
                    .cloned()
            });
            let open = on_open.clone();
            dynamic(move || {
                let open = open.clone();
                let conversation = conversation.get()?;
                let is_selected = selected.with(|current| {
                    current
                        .as_ref()
                        .is_some_and(|current| current.id == conversation.id)
                });
                Some(untracked(move || {
                    let chosen = conversation.clone();
                    chat_row(
                        conversation,
                        Listener::from(move |_: &()| open(&chosen)),
                        is_selected,
                    )
                }))
            })
        },
    )
    .keys(move || {
        matching_conversations(store, query)
            .into_iter()
            .map(|conversation| conversation.id)
            .collect::<Vec<_>>()
    })
    .estimated_item_size(px(72.0))
    .flex_1()
    .into_element()
}

/// The chats list content: the search field and the list (no title).
pub fn chats_list(
    store: Signal<Store>,
    selected: Signal<Option<Conversation>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
) -> AnyElement {
    let query = signal(String::new());
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(search(query))
        .child(dynamic(move || {
            if matching_conversations(store, query).is_empty() {
                view()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .px_4()
                    .child(
                        text("No matching chats")
                            .text_size(px(14.0))
                            .text_color(colors::MUTED),
                    )
                    .into_element()
            } else {
                list(store, query, selected, on_open.clone())
            }
        }))
        .into_element()
}

/// The chats page (the phone's root): a title, then the list.
pub fn chats_page(
    store: Signal<Store>,
    selected: Signal<Option<Conversation>>,
    on_open: impl Fn(&Conversation) + Clone + 'static,
    on_new: impl Fn() + 'static,
) -> AnyElement {
    let new_chat = move |_: &()| on_new();
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(header("Chats", new_chat))
        .child(chats_list(store, selected, on_open))
        .into_element()
}
