//! NodeChat's shell: the responsive frame the app lives in.
//!
//! Three adaptive layouts, chosen by the window's width:
//!
//! - **Narrow** (phone / small tablet): the tab bar of the app's sections at
//!   the bottom; each section is a full page. The **Chats** page is the
//!   conversation list, and opening a chat **pushes** the conversation (its
//!   header has a back control, and the swipe/back key pop it).
//! - **Medium** (desktop / large tablet): the **master-detail** layout — the
//!   sidebar holds the conversation list, and the content area shows the
//!   open conversation (a glance, not navigation). **Contacts** and
//!   **Settings** are full pages that take over the content area (the
//!   sidebar keeps its single job, the list), as desktop apps present their
//!   settings (Discord, Telegram, Slack).
//! - **Wide** (desktop): a compact section rail, a dedicated chat list, and
//!   the conversation pane, keeping primary navigation visible without
//!   crowding the list.
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
    data::{Conversation, Store, sample_store},
    desktop,
    icons::icon,
    layout::{desktop_rail, sidebar_width},
    nav, press,
    screens::{chats_list, chats_page, contacts_page, conversation, new_chat_page, settings_page},
    theme::{chat, radius, tints},
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

fn new_chat_mode() -> Signal<bool> {
    global::<bool>()
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
    new_chat_mode().set(false);
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

/// The adaptive desktop layout.
fn wide_layout() -> AnyElement {
    let chosen = section();
    when(desktop_rail, move || rail_layout(chosen))
        .otherwise(move || sidebar_layout(chosen))
        .into_element()
}

/// Desktop layout with a compact section rail, chat list, and detail pane.
fn rail_layout(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(navigation_rail(chosen))
        .child(chat_list_panel())
        .child(detail_pane(chosen))
        .into_element()
}

/// Desktop layout with navigation and chats combined in the sidebar.
fn sidebar_layout(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(sidebar(chosen))
        .child(detail_pane(chosen))
        .into_element()
}

/// The current section's main content.
fn detail_pane(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .flex_1()
        .child(dynamic(move || match chosen.get().0 {
            Section::Chats => {
                conversation(open(), None, Some(Listener::from(|_: &()| open_new_chat())))
            }
            Section::Contacts => {
                if new_chat_mode().get() {
                    new_chat_page(
                        store(),
                        Some(back_to_chats(chosen)),
                        open_people_conversation,
                    )
                } else {
                    contacts_page(
                        store(),
                        Some(back_to_chats(chosen)),
                        open_people_conversation,
                        open_new_chat,
                    )
                }
            }
            Section::Settings => settings_page(store(), Some(back_to_chats(chosen))),
        }))
        .into_element()
}

/// A listener that returns the wide layout to the conversations.
fn back_to_chats(chosen: Signal<Chosen>) -> Listener<()> {
    Listener::from(move |_: &()| {
        new_chat_mode().set(false);
        chosen.set(Chosen(Section::Chats));
    })
}

fn select_section(chosen: Signal<Chosen>, selected: Section) {
    new_chat_mode().set(false);
    chosen.set(Chosen(selected));
}

/// Opens the new-conversation chooser in the desktop pane or phone stack.
fn open_new_chat() {
    if desktop() {
        new_chat_mode().set(true);
        section().set(Chosen(Section::Contacts));
    } else {
        nav::push("new-chat", || new_chat_screen());
    }
}

fn new_chat_screen() -> Screen {
    let back = Listener::from(|_: &()| nav::pop());
    nsh_base::nav_page()
        .header_visible(false)
        .child(new_chat_page(
            store(),
            Some(back),
            open_new_chat_conversation,
        ))
}

fn open_new_chat_conversation(conversation: &Conversation) {
    nav::pop();
    open_people_conversation(conversation);
}

/// The phone layout: the section's page over a tab bar.
fn phone_layout() -> AnyElement {
    let chosen = section();
    let routes = nav::routes();
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            nav_stack()
                .flex_1()
                .child(
                    nav_page().key("root").header_visible(false).child(dynamic(
                        move || match chosen.get().0 {
                            Section::Chats => {
                                chats_page(store(), open(), |c| open_conversation(c), open_new_chat)
                            }
                            Section::Contacts => contacts_page(
                                store(),
                                None,
                                open_people_conversation,
                                open_new_chat,
                            ),
                            Section::Settings => settings_page(store(), None),
                        },
                    )),
                )
                .child(pushed()),
        )
        .child(when(
            move || routes.with(|routes| routes.0.is_empty()),
            move || tab_bar(chosen),
        ))
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

/// Opens a People-directory conversation, returning the wide layout to Chats first.
fn open_people_conversation(conversation: &Conversation) {
    open_conversation(conversation);
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
                    text("Chats")
                        .flex_1()
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(new_chat_button(open_new_chat)),
        )
        .child(chats_list(store, open(), |c| open_conversation(c)))
        .into_element()
}

/// The dedicated desktop list column.
fn chat_list_panel() -> AnyElement {
    let store = store();
    view()
        .w(px(sidebar_width()))
        .flex_col()
        .bg(colors::BACKGROUND)
        .border_r_1()
        .border_r_color(colors::LINE)
        .child(
            view()
                .h(px(60.0))
                .flex_row()
                .items_center()
                .px_4()
                .child(
                    text("Chats")
                        .flex_1()
                        .text_size(px(22.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(new_chat_button(open_new_chat)),
        )
        .child(chats_list(store, open(), |c| open_conversation(c)))
        .into_element()
}

/// The main-action icon used to start a conversation.
fn new_chat_button(on_new: impl Fn() + 'static) -> AnyElement {
    press()
        .label("New chat")
        .on_press(on_new)
        .child(
            view()
                .size(px(36.0))
                .rounded(px(radius::CONTROL))
                .items_center()
                .justify_center()
                .hover(|s| s.bg(colors::RAISED))
                .active(|s| s.bg(colors::RAISED))
                .child(
                    icon()
                        .name(IconName::Plus)
                        .size(19.0)
                        .tint(colors::ACCENT)
                        .label(None),
                ),
        )
        .into_element()
}

/// The compact desktop rail for the app's primary sections.
fn navigation_rail(chosen: Signal<Chosen>) -> AnyElement {
    view()
        .w(px(72.0))
        .flex_col()
        .items_center()
        .gap_2()
        .pt_3()
        .bg(colors::INSET)
        .border_r_1()
        .border_r_color(colors::LINE)
        .child(
            view()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .mb_2()
                .child(
                    view()
                        .size(px(38.0))
                        .rounded(px(11.0))
                        .items_center()
                        .justify_center()
                        .bg(colors::ACCENT_FILL)
                        .child(
                            icon()
                                .name(IconName::Node)
                                .size(22.0)
                                .tint(colors::ON_ACCENT)
                                .label(None),
                        ),
                )
                .child(
                    text("NodeChat")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::MUTED),
                ),
        )
        .children(Section::ALL.into_iter().map(move |s| {
            let selected = move || chosen.get().0 == s;
            view()
                .w(px(58.0))
                .h(px(54.0))
                .rounded(px(radius::CONTROL))
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.0))
                .cursor_pointer()
                .role(AccessibilityRole::Button)
                .label(s.label())
                .accessibility_state(move || AccessibilityState {
                    selected: selected(),
                    ..AccessibilityState::default()
                })
                .style_when(selected, |style| style.bg(tints::ACCENT))
                .hover(|style| style.bg(colors::SURFACE))
                .active(|style| style.bg(colors::SURFACE))
                .on_press(move || select_section(chosen, s))
                .child(icon().name(s.icon()).size(19.0).tint(move || {
                    if selected() {
                        colors::ACCENT
                    } else {
                        colors::MUTED
                    }
                }))
                .child(
                    text(s.label())
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
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
                .rounded(px(12.0))
                .items_center()
                .justify_center()
                .bg(chat::SENT)
                .child(
                    icon()
                        .name(IconName::Node)
                        .size(20.0)
                        .tint(chat::SENT_FOREGROUND)
                        .label(None),
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
                .when(selected(), |st| {
                    st.border_l_1().border_l_color(colors::ACCENT)
                })
                .hover(|st| st.bg(colors::SURFACE))
                .active(|st| st.bg(colors::SURFACE))
                .on_press(move || select_section(chosen, s))
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
                    select_section(chosen, s);
                })
                .child(
                    view()
                        .w(px(52.0))
                        .h(px(30.0))
                        .rounded(px(15.0))
                        .items_center()
                        .justify_center()
                        .style_when(selected, |style| style.bg(tints::ACCENT))
                        .child(icon().name(s.icon()).size(21.0).tint(move || {
                            if selected() {
                                chat::SENT
                            } else {
                                colors::FAINT
                            }
                        })),
                )
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
