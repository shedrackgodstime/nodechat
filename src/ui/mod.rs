//! NodeChat's UI: the app's shell and screens, written in Rust on NSH.
//!
//! The reusable layer is adapted from NSH's own Rust gallery
//! (`ref/nsh/examples/gallery-rs`): the look ([`theme`]), the icons, the
//! building blocks ([`parts`]), the stateful controls ([`controls`],
//! [`animated`], [`sheet`]) and the desktop-layout helpers ([`layout`]).
//! NodeChat adds [`shell`], its [`screens`] and a few [`widgets`].

mod animated;
mod controls;
mod device;
mod icons;
mod layout;
mod parts;
mod shapes;
mod sheet;
mod theme;
mod widgets;

mod data;
mod nav;
mod screens;
mod shell;

use nsh_core::prelude::*;

pub(crate) use self::{
    animated::{PressRole, appear, press},
    controls::{Segment, SegmentedElement, SegmentedSize, segmented, toggle},
    device::{bind_refresh, hint},
    icons::{IconName, Tint, icon},
    layout::{
        TERMINAL_COLUMN, TERMINAL_ROW, columns_for, desktop, pane_width, sidebar_width, terminal,
    },
    parts::{
        Badge, Button, ButtonVariant, Card, Figure, IconTile, Inset, Meter, PageHeader,
        RoundButton, Row, Section, Stat, caption, code, header, legend, titled, with_header,
    },
    shapes::{lengths, pt, rect},
    sheet::sheet,
    theme::{Finish, Finished, MONO, Tone, colors, inset_shadow, motion, radius, rgb_a, tints},
};

pub use self::shell::AppShell;

/// The application's root component.
#[component]
pub fn App() -> impl IntoElement {
    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(shell::app_shell())
}
