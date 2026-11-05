//! NodeChat's shell: the responsive frame the app lives in.
//!
//! Adapted from NSH's own Rust gallery shell (`ref/nsh/examples/gallery-rs`):
//! on wide windows a sidebar of the app's sections beside the open pane; on
//! phones the pane alone. The window's width decides (`ui::desktop`), and
//! the shell is made again when it changes, so a screen reads the layout
//! once.

use nsh_base::{nav_page, nav_stack};
use nsh_core::{element::AnyElement, prelude::*};

use crate::ui::{
    IconName, chat::chat, colors, desktop, icon, layout::sidebar_width, nav,
};

/// The application's shell.
#[component]
pub fn AppShell() -> impl IntoElement {
    view()
        .flex_1()
        .child(when(desktop, wide_layout).otherwise(phone_layout))
}

/// The phone layout: the navigation stack alone.
fn phone_layout() -> AnyElement {
    nav_stack()
        .flex_1()
        .child(nav_page().key("root").header_visible(false).child(chat()))
        .child(pushed())
        .into_element()
}

/// The wide layout: a sidebar beside the open pane.
fn wide_layout() -> AnyElement {
    view()
        .flex_1()
        .flex_row()
        .bg(colors::BACKGROUND)
        .child(sidebar())
        .child(
            nav_stack()
                .flex_1()
                .child(nav_page().key("root").header_visible(false).child(chat()))
                .child(pushed()),
        )
        .into_element()
}

/// The screens pushed over the root.
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

/// The app's name and the sections.
fn sidebar() -> AnyElement {
    view()
        .w(px(sidebar_width()))
        .bg(colors::INSET)
        .border_r_1()
        .border_r_color(colors::LINE)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .px_5()
                .pt_5()
                .pb_3()
                .child(
                    view()
                        .size(px(32.0))
                        .rounded(px(8.0))
                        .items_center()
                        .justify_center()
                        .bg(colors::ACCENT)
                        .child(
                            text("N")
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(colors::ON_ACCENT),
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
                            text(format!("NSH · {}", os().name()))
                                .text_size(px(12.0))
                                .text_color(colors::MUTED),
                        ),
                ),
        )
        .child(
            view().px_5().pt_6().child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        icon()
                            .name(IconName::Chat)
                            .size(16.0)
                            .tint(colors::ACCENT)
                            .label(None),
                    )
                    .child(
                        text("Chats")
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    ),
            ),
        )
        .into_element()
}
