//! NodeChat's small widgets: the pieces the screens repeat, built on the
//! shared parts in [`crate::ui::parts`] and the palette in
//! [`crate::ui::theme`]. Each is a plain function or builder, styled with
//! `Styled` methods, as NSH's own components are.

use nsh_core::{
    element::{AdaptiveColor, AnyElement, StyleRefinement},
    prelude::*,
};

use crate::ui::{
    IconName,
    data::MessageState,
    icons::{Tint, icon},
    parts::Label,
    theme::{chat, colors},
};

/// An avatar: initials on a tinted circle, with an optional presence or
/// verified dot.
pub struct Avatar {
    name: Label,
    size: f32,
    online: Option<Prop<bool>>,
    verified: Option<Prop<bool>>,
    root: View,
}

impl Avatar {
    /// An avatar of `name` (its first letter is shown), 48 pt.
    pub fn new(name: impl IntoReactive<SharedString>) -> Self {
        Self {
            name: name.into_reactive(),
            size: 48.0,
            online: None,
            verified: None,
            root: view(),
        }
    }

    /// Its diameter.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// A presence dot at its corner while `online`.
    pub fn online(mut self, online: impl Into<Prop<bool>>) -> Self {
        self.online = Some(online.into());
        self
    }

    /// A verified mark at its corner.
    pub fn verified(mut self, verified: impl Into<Prop<bool>>) -> Self {
        self.verified = Some(verified.into());
        self
    }
}

impl IntoElement for Avatar {
    fn into_element(self) -> AnyElement {
        let size = self.size;
        let initial = self.name.clone().map(|name| {
            SharedString::from(
                name.chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_default(),
            )
        });
        let dot = size * 0.28;
        self.root
            .size(px(size))
            .child(
                view()
                    .size(px(size))
                    .rounded(px(size / 2.0))
                    .items_center()
                    .justify_center()
                    .bg(colors::RAISED)
                    .border_1()
                    .border_color(colors::BORDER)
                    .child(
                        text(initial)
                            .text_size(px(size * 0.4))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::MUTED),
                    ),
            )
            .children(self.online.map(|online| {
                view()
                    .absolute()
                    .right(px(0.0))
                    .bottom(px(0.0))
                    .size(px(dot))
                    .rounded(px(dot / 2.0))
                    .border_2()
                    .border_color(colors::SURFACE)
                    .bg(move || {
                        if online.get() {
                            chat::ONLINE
                        } else {
                            colors::FAINT
                        }
                    })
            }))
            .children(self.verified.map(|verified| {
                view()
                    .absolute()
                    .right(px(-2.0))
                    .bottom(px(size * 0.42))
                    .size(px(dot * 0.9))
                    .rounded(px(dot * 0.45))
                    .items_center()
                    .justify_center()
                    .bg(chat::VERIFIED)
                    .opacity(move || if verified.get() { 1.0_f32 } else { 0.0 })
                    .child(
                        icon()
                            .name(IconName::Check)
                            .size(dot * 0.6)
                            .tint(crate::ui::icons::Tint::from(AdaptiveColor::from(
                                nsh_core::element::rgb(0xffffff),
                            )))
                            .label(None),
                    )
            }))
            .into_element()
    }
}

impl Styled for Avatar {
    fn style(&mut self) -> &mut StyleRefinement {
        Styled::style(&mut self.root)
    }
}

impl FluentBuilder for Avatar {}

/// A message's state as a small mark (a tick, a clock, an alert).
pub fn state_mark(state: impl IntoReactive<MessageState>) -> View {
    let state = state.into_reactive();
    let name = state.clone().map(|s| match s {
        MessageState::Queued => IconName::Clock,
        MessageState::Failed => IconName::Alert,
        _ => IconName::Check,
    });
    let tint = state.map(|s| Tint::from(match s {
        MessageState::Queued => chat::QUEUED,
        MessageState::Failed => chat::FAILED,
        MessageState::Read => chat::ONLINE,
        _ => colors::FAINT,
    }));
    view()
        .size(px(14.0))
        .child(icon().name(name).size(13.0).tint(tint).label(None))
}
