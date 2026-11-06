//! The settings screen: this device's identity, notifications and network.
//! It is a full page (a tab page on phones, a page in the content area on
//! wide windows).

use nsh_core::{
    element::{AnyElement, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::Store,
    icons::icon,
    parts::{Row, Section},
    theme::{chat, colors},
    toggle,
    widgets::PageHeader,
};

/// A settings row: an icon, a label, an optional detail, and a trailing
/// element (a toggle, a value).
fn setting_row(
    name: IconName,
    label: &'static str,
    detail: Option<&'static str>,
    trailing: Option<AnyElement>,
) -> AnyElement {
    Row::new(false)
        .child(icon().name(name).size(18.0).tint(colors::MUTED).label(None))
        .child(
            view()
                .flex_1()
                .gap(px(1.0))
                .child(text(label).text_size(px(15.0)).text_color(colors::TEXT))
                .children(detail.map(|d| text(d).text_size(px(12.0)).text_color(colors::FAINT))),
        )
        .children(trailing)
        .into_element()
}

/// The settings page. `back` is `Some` where the page replaces the content
/// area (wide windows); `None` when it is a tab page (phones).
pub fn settings_page(store: Signal<Store>, back: Option<Listener<()>>) -> AnyElement {
    let name = computed(move || store.with(|s| s.me.clone()));
    let online = computed(move || store.with(|s| s.online));
    let notifications = signal(true);
    let read_receipts = signal(true);

    let identity = Section::new("Identity")
        .child(
            Row::new(true)
                .child(
                    view()
                        .size(px(40.0))
                        .rounded(px(12.0))
                        .items_center()
                        .justify_center()
                        .bg(colors::RAISED)
                        .border_1()
                        .border_color(colors::BORDER)
                        .child(
                            text(name.clone().into_reactive().map(|n| {
                                SharedString::from(
                                    n.chars()
                                        .next()
                                        .map(|c| c.to_uppercase().to_string())
                                        .unwrap_or_default(),
                                )
                            }))
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::TEXT),
                        ),
                )
                .child(
                    view()
                        .flex_1()
                        .gap(px(1.0))
                        .child(
                            text(name)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                        )
                        .child(
                            text(move || {
                                if online.get() {
                                    "Reachable on the network"
                                } else {
                                    "Offline"
                                }
                            })
                            .text_size(px(12.0))
                            .text_color(move || {
                                if online.get() {
                                    chat::ONLINE
                                } else {
                                    colors::FAINT
                                }
                            }),
                        ),
                ),
        )
        .child(setting_row(
            IconName::Lock,
            "Encryption",
            Some("Messages are end-to-end encrypted"),
            None,
        ))
        .child(setting_row(
            IconName::Contacts,
            "My ticket",
            Some("Share it so peers can reach you"),
            None,
        ));

    let notifications_section = Section::new("Notifications")
        .child(
            Row::new(true)
                .child(
                    view().flex_1().child(
                        text("Message notifications")
                            .text_size(px(15.0))
                            .text_color(colors::TEXT),
                    ),
                )
                .child(
                    toggle()
                        .value(notifications)
                        .on_change(move |&on: &bool| notifications.set(on)),
                ),
        )
        .child(
            Row::new(false)
                .child(
                    view().flex_1().child(
                        text("Send read receipts")
                            .text_size(px(15.0))
                            .text_color(colors::TEXT),
                    ),
                )
                .child(
                    toggle()
                        .value(read_receipts)
                        .on_change(move |&on: &bool| read_receipts.set(on)),
                ),
        );

    let network = Section::new("Network & delivery")
        .child(setting_row(
            IconName::Contacts,
            "Direct peer-to-peer",
            Some("Connect directly to peers without a central account"),
            None,
        ))
        .child(setting_row(
            IconName::Clock,
            "Offline messages",
            Some("Queued on this device until a direct path is available"),
            None,
        ))
        .child(setting_row(
            IconName::Lock,
            "Background availability",
            Some("The operating system may pause network work while the app is closed"),
            None,
        ));

    scroll()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .max_w(px(760.0))
                .w_full()
                .self_center()
                .child(PageHeader::new("Settings").back(back))
                .child(
                    view()
                        .p_4()
                        .gap_4()
                        .child(identity)
                        .child(notifications_section)
                        .child(network),
                ),
        )
        .into_element()
}
