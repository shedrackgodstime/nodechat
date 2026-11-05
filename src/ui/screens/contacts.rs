//! The contacts screen: the peers this device knows, with their tickets.

use nsh_core::{
    element::{AnyElement, SharedString},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::{Contact, Store},
    icons::icon,
    press,
    theme::{chat, colors, radius},
    widgets::Avatar,
};

/// A contact's row: avatar, name, ticket, and its verified mark.
fn contact_row(contact: Contact) -> AnyElement {
    view()
        .h(px(72.0))
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .child(
            Avatar::new(contact.name.clone())
                .size(48.0)
                .online(contact.online)
                .verified(contact.verified),
        )
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(contact.name.clone())
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(contact.ticket.clone())
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(colors::FAINT)
                        .font_family("monospace"),
                ),
        )
        .children(contact.verified.then(|| {
            icon()
                .name(IconName::Shield)
                .size(18.0)
                .tint(chat::VERIFIED)
                .label(None)
        }))
        .into_element()
}

/// The contacts list content.
pub fn contacts_list(store: Signal<Store>) -> AnyElement {
    let add = move |_: &()| {};
    let copy = move |_: &()| {};
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            press()
                .label("Add contact")
                .on_press(add)
                .child(
                    view()
                        .h(px(56.0))
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .child(
                            text("Contacts")
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
                ),
        )
        .child(
            text("YOUR TICKET")
                .px_4()
                .pt_2()
                .pb_1()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .letter_spacing(px(0.6))
                .text_color(colors::FAINT),
        )
        .child(
            view()
                .mx_4()
                .p_3()
                .rounded(px(radius::INSET))
                .border_curve(BorderCurve::Continuous)
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    icon()
                        .name(IconName::Lock)
                        .size(18.0)
                        .tint(chat::SENT)
                        .label(None),
                )
                .child(
                    text("nodechat4k7p…2v")
                        .flex_1()
                        .text_size(px(13.0))
                        .text_color(colors::MUTED)
                        .font_family("monospace"),
                )
                .child(
                    press()
                        .label("Copy ticket")
                        .on_press(copy)
                        .child(
                            text("Copy")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chat::SENT),
                        ),
                ),
        )
        .child(
            text("KNOWN PEERS")
                .px_4()
                .pt_4()
                .pb_1()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .letter_spacing(px(0.6))
                .text_color(colors::FAINT),
        )
        .child(
            virtual_list(
                move || store.with(|s| s.contacts.len()),
                move |at: Position| {
                    let contact = computed(move || store.with(|s| s.contacts.get(at.index()).cloned()));
                    dynamic(move || contact.get().map(contact_row))
                },
            )
            .keys(move || {
                store.with(|s| s.contacts.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
            })
            .estimated_item_size(px(72.0))
            .flex_1(),
        )
        .into_element()
}
