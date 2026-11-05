//! NodeChat's shell: the responsive frame the app lives in.
//!
//! Two idioms, chosen by the window's width (`desktop()`), as WhatsApp and
//! Telegram do across platforms:
//!
//! - **Wide** (desktop / large tablet): a sidebar of the app's sections over
//!   the chat list, beside the open conversation. Selecting a chat swaps the
//!   pane (a glance, not navigation); nothing selected shows an empty state.
//! - **Narrow** (phone / small tablet): the chat list fills the window, the
//!   three sections sit in a tab bar at the bottom, and opening a chat
//!   **pushes** it on the navigation stack (system header, swipe/back pops).
//!
//! The shell is made again when the width crosses the threshold (`desktop`),
//! so a screen reads the layout once.

use nsh_base::{nav_page, nav_stack};
use nsh_core::{
    element::{AccessibilityState, AnyElement, Screen, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    colors,
    data::{Conversation, Store, sample_store},
    desktop,
    icons::icon,
    layout::sidebar_width,
    nav,
    screens::{chats_list, contacts_list, conversation as conversation_view, settings_list},
    theme::{chat, radius},
};

/// The app's top-level sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    /// The conversation list.
    #[default]
    Chats,
    /// The known peers and this device's ticket.
    Contacts,
    /// Identity, notifications and network.
    Settings,
}

impl Section {
    /// Its label.
    fn label(self) -> &'static str {
        match self {
            Section::Chats => "Chats",
            Section::Contacts => "Contacts",
            Section::Settings => "Settings",
        }
    }

    /// Its icon.
    fn icon(self) -> IconName {
        match self {
            Section::Chats => IconName::Chat,
            Section::Contacts => IconName::Contacts,
            Section::Settings => IconName::Settings,
        }
    }

    /// The sections in order.
    const ALL: [Section; 3] = [Section::Chats, Section::Contacts, Section::Settings];
}

/// The chosen section, a surface-wide signal the chrome and the panes share.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Chosen(pub Section);

/// The chosen section's signal.
pub fn section() -> Signal<Chosen> {
    global::<Chosen>()
}

/// The conversation open in the wide layout (`None`: none): a surface-wide
/// signal of the selection, `Option<Conversation>` itself (its `Default` is
/// `None`).
pub fn open() -> Signal<Option<Conversation>> {
    global::<Option<Conversation>>()
}

/// The app's data, a surface-wide signal (seeded at mount with the sample
/// store until the backend fills it).
pub fn store() -> Signal<Store> {
    global::<Store>()
}

/// The application's shell.
#[component]
pub fn AppShell() -> impl IntoElement {
    // Seed the store once with the sample data (the backend fills it later).
    if untracked(|| store().with(|s| s.conversations.is_empty())) {
        store().set(sample_store());
    }
    view()
        .flex_1()
        .child(when(desktop, wide_layout).otherwise(phone_layout))
}

/// Opens the conversation `conversation`: fills the wide pane on desktops,
/// pushes it on phones.
fn open_conversation(conversation: &Conversation) {
    let conversation = conversation.clone();
    if desktop() {
        open().set(Some(conversation));
    } else {
        nav::push(conversation.id.clone(), move || conversation_route(&conversation));
    }
}

/// A conversation as a screen, for the phone stack.
fn conversation_route(conversation: &Conversation) -> Screen {
    let id = conversation.id.clone();
    nsh_base::nav_page()
        .title(conversation.name.clone())
        .child(conversation_view(signal(Some(conversation.clone()))))
        .key(id)
}

/// The wide layout: sidebar beside the open pane.
fn wide_layout() -> AnyElement {
    let chosen = section();
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(sidebar(chosen))
        .child(
            view()
                .flex_1()
                .child(match chosen.get().0 {
                    Section::Chats => conversation_view(open()),
                    Section::Contacts => contacts_list(store()),
                    Section::Settings => settings_list(store()),
                }),
        )
        .into_element()
}

/// The phone layout: the section content, with a tab bar and a nav stack.
fn phone_layout() -> AnyElement {
    let chosen = section();
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            nav_stack()
                .flex_1()
                .child(
                    nav_page()
                        .key("root")
                        .header_visible(false)
                        .child(dynamic(move || match chosen.get().0 {
                            Section::Chats => chats_list(store(), |c| open_conversation(c)),
                            Section::Contacts => contacts_list(store()),
                            Section::Settings => settings_list(store()),
                        })),
                )
                .child(pushed()),
        )
        .child(tab_bar(chosen))
        .into_element()
}

/// The screens pushed on the phone stack.
fn pushed() -> AnyElement {
    let routes = nav::routes();
    each(
        move || routes.with(|r| r.0.clone()),
        |route: &nav::Route| route.key.clone(),
        |route, depth| {
            let route = route.get_untracked();
            (route.screen)().on_dismiss(move || nav::truncate(depth.get_untracked()))
        },
    )
    .into_element()
}

/// The wide sidebar: the app's identity, the sections, and the chat list.
fn sidebar(chosen: Signal<Chosen>) -> AnyElement {
    let store = store();
    view()
        .w(px(sidebar_width()))
        .flex_col()
        .bg(colors::INSET)
        .border_r_1()
        .border_r_color(colors::LINE)
        .child(brand(store))
        .child(nav_items(chosen))
        .child(
            view()
                .flex_1()
                .border_t_1()
                .border_t_color(colors::LINE)
                .child(dynamic(move || match chosen.get().0 {
                    Section::Chats => chats_list(store, |c| open_conversation(c)),
                    Section::Contacts => contacts_list(store),
                    Section::Settings => settings_list(store),
                })),
        )
        .into_element()
}

/// The sidebar's brand: the app's name and this device's state.
fn brand(store: Signal<Store>) -> AnyElement {
    let online = store.clone().into_reactive().map(|s| s.online);
    let online_text = online.clone();
    let online_color = online;
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .pt_4()
        .pb_3()
        .child(
            view()
                .size(px(34.0))
                .rounded(px(10.0))
                .items_center()
                .justify_center()
                .bg(chat::SENT)
                .child(
                    text("N")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(chat::SENT_FOREGROUND),
                ),
        )
        .child(
            view()
                .flex_1()
                .gap(px(1.0))
                .child(
                    text("NodeChat")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(move || if online_text.get() { "Online" } else { "Offline" })
                        .text_size(px(12.0))
                        .text_color(move || {
                            if online_color.get() {
                                chat::ONLINE
                            } else {
                                colors::FAINT
                            }
                        }),
                ),
        )
        .into_element()
}

/// The section buttons in the sidebar.
fn nav_items(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .px_2()
        .pb_2()
        .gap(px(2.0))
        .children(Section::ALL.into_iter().map(move |s| {
            let selected = move || chosen.get().0 == s;
            view()
                .h(px(38.0))
                .px_3()
                .rounded(px(radius::CONTROL))
                .flex_row()
                .items_center()
                .gap_3()
                .cursor_pointer()
                .role(AccessibilityRole::Button)
                .label(s.label())
                .accessibility_state(move || AccessibilityState {
                    selected: selected(),
                    ..AccessibilityState::default()
                })
                .style_when(selected, |st| st.bg(colors::SURFACE))
                .hover(|st| st.bg(colors::SURFACE))
                .active(|st| st.bg(colors::SURFACE))
                .on_press(move || chosen.set(Chosen(s)))
                .child(
                    icon()
                        .name(s.icon())
                        .size(18.0)
                        .tint(move || {
                            if selected() {
                                colors::TEXT
                            } else {
                                colors::MUTED
                            }
                        }),
                )
                .child(
                    text(s.label())
                        .text_size(px(14.0))
                        .style_when(selected, |st| st.font_weight(FontWeight::SEMIBOLD))
                        .text_color(move || {
                            if selected() {
                                colors::TEXT
                            } else {
                                colors::MUTED
                            }
                        }),
                )
                .into_element()
        }))
        .into_element()
}

/// The bottom tab bar on phones.
fn tab_bar(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .bg(colors::BACKGROUND)
        .border_t_1()
        .border_t_color(colors::LINE)
        .pb_safe_area()
        .flex_row()
        .children(Section::ALL.into_iter().map(move |s| {
            let selected = move || chosen.get().0 == s;
            view()
                .flex_1()
                .h(px(56.0))
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.0))
                .role(AccessibilityRole::Tab)
                .label(s.label())
                .accessibility_state(move || AccessibilityState {
                    selected: selected(),
                    ..AccessibilityState::default()
                })
                .on_press(move || {
                    chosen.set(Chosen(s));
                    nav::pop_to_top();
                })
                .child(
                    icon()
                        .name(s.icon())
                        .size(22.0)
                        .tint(move || {
                            if selected() {
                                chat::SENT
                            } else {
                                colors::FAINT
                            }
                        }),
                )
                .child(
                    text(s.label())
                        .text_size(px(11.0))
                        .text_color(move || {
                            if selected() {
                                chat::SENT
                            } else {
                                colors::FAINT
                            }
                        }),
                )
                .into_element()
        }))
        .into_element()
}

