//! NodeChat's shell: the responsive frame the app lives in.
//!
//! Two idioms, chosen by the window's width (`desktop()`):
//!
//! - **Narrow** (phone / small tablet): the tab bar of the app's sections at
//!   the bottom; each section is a full page. The **Chats** page is the
//!   conversation list, and opening a chat **pushes** the conversation (its
//!   header has a back control, and the swipe/back key pop it).
//! - **Wide** (desktop / large tablet): the **master-detail** layout — the
//!   sidebar holds the conversation list, and the content area shows the
//!   open conversation (a glance, not navigation). **Contacts** and
//!   **Settings** are full pages that take over the content area (the
//!   sidebar keeps its single job, the list), as desktop apps present their
//!   settings (Discord, Telegram, Slack).
//!
//! The shell is made again when the width crosses the threshold, so a screen
//! reads the layout once.

use nsh_base::{nav_page, nav_stack};
use nsh_core::{
    element::{AccessibilityState, AnyElement, Screen},
    prelude::*,
};

use crate::ui::{
    IconName, colors,
    data::{Contact, Conversation, Store, sample_store},
    desktop,
    icons::icon,
    layout::sidebar_width,
    nav, press,
    screens::{chats_list, chats_page, contacts_page, conversation, settings_page},
    theme::{chat, radius},
};

/// The app's sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    /// The conversations.
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

/// The chosen section, a surface-wide signal the chrome and the pages share.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct Chosen(pub Section);

/// The chosen section's signal.
pub fn section() -> Signal<Chosen> {
    global::<Chosen>()
}

/// The conversation open in the wide layout (`None`: none): `Option` itself,
/// whose `Default` is `None`.
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
    if untracked(|| store().with(|s| s.conversations.is_empty())) {
        store().set(sample_store());
    }
    view()
        .flex_1()
        .child(when(desktop, wide_layout).otherwise(phone_layout))
}

/// Opens the conversation `conversation`: fills the wide pane, or pushes it
/// on phones.
fn open_conversation(conversation: &Conversation) {
    section().set(Chosen(Section::Chats));
    if desktop() {
        open().set(Some(conversation.clone()));
    } else {
        let conversation = conversation.clone();
        nav::push(conversation.id.clone(), move || {
            conversation_screen(&conversation)
        });
    }
}

/// A conversation as a pushed screen, with a back control in its header.
fn conversation_screen(current: &Conversation) -> Screen {
    let back = Listener::from(|_: &()| nav::pop());
    nsh_base::nav_page()
        .title(current.name.clone())
        .header_visible(false)
        .child(conversation(
            signal(Some(current.clone())),
            Some(back),
            None,
        ))
}

/// The wide layout: the sidebar (the list) beside the content area.
fn wide_layout() -> AnyElement {
    let chosen = section();
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(sidebar(chosen))
        .child(view().flex_1().child(dynamic(move || match chosen.get().0 {
            Section::Chats => conversation(
                open(),
                None,
                Some(Listener::from(|_: &()| {
                    section().set(Chosen(Section::Contacts));
                })),
            ),
            Section::Contacts => contacts_page(store(), Some(back_to_chats(chosen)), open_contact),
            Section::Settings => settings_page(store(), Some(back_to_chats(chosen))),
        })))
        .into_element()
}

/// A listener that returns the wide layout to the conversations.
fn back_to_chats(chosen: Signal<Chosen>) -> Listener<()> {
    Listener::from(move |_: &()| chosen.set(Chosen(Section::Chats)))
}

/// The phone layout: the section's page over a tab bar.
fn phone_layout() -> AnyElement {
    let chosen = section();
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            nav_stack()
                .flex_1()
                .child(
                    nav_page().key("root").header_visible(false).child(dynamic(
                        move || match chosen.get().0 {
                            Section::Chats => chats_page(
                                store(),
                                open(),
                                |c| open_conversation(c),
                                || {
                                    section().set(Chosen(Section::Contacts));
                                },
                            ),
                            Section::Contacts => contacts_page(store(), None, open_contact),
                            Section::Settings => settings_page(store(), None),
                        },
                    )),
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

/// Opens a contact's conversation, returning the wide layout to Chats first.
fn open_contact(contact: &Contact) {
    let matching = store().with_untracked(|s| {
        s.conversations
            .iter()
            .find(|conversation| conversation.id == contact.id)
            .cloned()
    });
    if let Some(conversation) = matching {
        open_conversation(&conversation);
    }
}

/// The wide sidebar: app identity, section navigation and the chat list.
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
                .h(px(40.0))
                .flex_row()
                .items_center()
                .px_4()
                .child(
                    text("CHATS")
                        .flex_1()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .letter_spacing(px(0.6))
                        .text_color(colors::FAINT),
                )
                .child(
                    press()
                        .label("New chat")
                        .on_press(move || chosen.set(Chosen(Section::Contacts)))
                        .child(
                            view()
                                .size(px(30.0))
                                .rounded(px(radius::CONTROL))
                                .items_center()
                                .justify_center()
                                .hover(|s| s.bg(colors::SURFACE))
                                .active(|s| s.bg(colors::SURFACE))
                                .child(
                                    icon()
                                        .name(IconName::Plus)
                                        .size(18.0)
                                        .tint(colors::MUTED)
                                        .label(None),
                                ),
                        ),
                ),
        )
        .child(chats_list(store, open(), |c| open_conversation(c)))
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
                    text(move || {
                        if online_text.get() {
                            "Online"
                        } else {
                            "Offline"
                        }
                    })
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

/// Section navigation for the wide sidebar.
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
                .child(icon().name(s.icon()).size(18.0).tint(move || {
                    if selected() {
                        colors::TEXT
                    } else {
                        colors::MUTED
                    }
                }))
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

/// The bottom tab bar on phones: the app's top-level sections.
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
                    if nav::depth() > 0 {
                        nav::pop_to_top();
                    }
                    chosen.set(Chosen(s));
                })
                .child(icon().name(s.icon()).size(22.0).tint(move || {
                    if selected() {
                        chat::SENT
                    } else {
                        colors::FAINT
                    }
                }))
                .child(text(s.label()).text_size(px(11.0)).text_color(move || {
                    if selected() {
                        chat::SENT
                    } else {
                        colors::FAINT
                    }
                }))
                .into_element()
        }))
        .into_element()
}
