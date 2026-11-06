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
        let avatar_background = self.name.clone().map(|name| {
            let index = name.bytes().fold(0usize, |hash, byte| {
                hash.wrapping_mul(31).wrapping_add(byte as usize)
            }) % chat::AVATAR_BACKGROUNDS.len();
            chat::AVATAR_BACKGROUNDS[index]
        });
        let avatar_foreground = self.name.clone().map(|name| {
            let index = name.bytes().fold(0usize, |hash, byte| {
                hash.wrapping_mul(31).wrapping_add(byte as usize)
            }) % chat::AVATAR_FOREGROUNDS.len();
            chat::AVATAR_FOREGROUNDS[index]
        });
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
                    .bg(avatar_background)
                    .border_1()
                    .border_color(colors::BORDER)
                    .child(
                        text(initial)
                            .text_size(px(size * 0.4))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(avatar_foreground),
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

/// A contact's shared avatar and identity layout, with context-specific
/// subtitle and trailing content.
pub(crate) struct ContactIdentity {
    name: Label,
    online: Option<bool>,
    verified: bool,
    subtitle: Option<AnyElement>,
    title_trailing: Option<AnyElement>,
}

impl ContactIdentity {
    pub(crate) fn new(name: impl IntoReactive<SharedString>) -> Self {
        Self {
            name: name.into_reactive(),
            online: None,
            verified: false,
            subtitle: None,
            title_trailing: None,
        }
    }

    pub(crate) fn online(mut self, online: bool) -> Self {
        self.online = Some(online);
        self
    }

    pub(crate) fn verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    pub(crate) fn subtitle(mut self, subtitle: impl IntoElement) -> Self {
        self.subtitle = Some(subtitle.into_element());
        self
    }

    pub(crate) fn title_trailing(mut self, trailing: impl IntoElement) -> Self {
        self.title_trailing = Some(trailing.into_element());
        self
    }
}

impl IntoElement for ContactIdentity {
    fn into_element(self) -> AnyElement {
        let avatar = Avatar::new(self.name.clone())
            .size(44.0)
            .verified(self.verified);
        let avatar = match self.online {
            Some(online) => avatar.online(online),
            None => avatar,
        };
        view()
            .flex_row()
            .flex_1()
            .items_center()
            .gap_3()
            .child(avatar)
            .child(
                view()
                    .flex_1()
                    .gap(px(2.0))
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                text(self.name)
                                    .flex_1()
                                    .number_of_lines(1)
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT),
                            )
                            .children(self.title_trailing),
                    )
                    .children(self.subtitle),
            )
            .into_element()
    }
}

/// A message's state as a small mark (a tick, a clock, an alert).
pub fn state_mark(state: impl IntoReactive<MessageState>) -> View {
    let state = state.into_reactive();
    let name = state.clone().map(|s| match s {
        MessageState::Queued => IconName::Clock,
        MessageState::Failed => IconName::Alert,
        _ => IconName::Check,
    });
    let tint = state.map(|s| {
        Tint::from(match s {
            MessageState::Queued => chat::QUEUED,
            MessageState::Failed => chat::FAILED,
            MessageState::Read => chat::ONLINE,
            _ => colors::FAINT,
        })
    });
    view()
        .size(px(14.0))
        .child(icon().name(name).size(13.0).tint(tint).label(None))
}

/// A page's header: an optional back control, the title, and trailing
/// actions, on the page's background over a hairline. Used where a page
/// replaces the content area (wide windows); on phones the tab bar is the
/// navigation, so a page header there is title-only.
pub struct PageHeader {
    title: SharedString,
    back: Option<Listener<()>>,
    right: Option<AnyElement>,
}

impl PageHeader {
    /// A header titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            back: None,
            right: None,
        }
    }

    /// A back control: `Some` where the page replaces the content area.
    pub fn back(mut self, back: Option<Listener<()>>) -> Self {
        self.back = back;
        self
    }

    /// A trailing action.
    pub fn right(mut self, right: impl IntoElement) -> Self {
        self.right = Some(right.into_element());
        self
    }
}

impl IntoElement for PageHeader {
    fn into_element(self) -> AnyElement {
        let title = self.title;
        view()
            .h(px(56.0))
            .flex_row()
            .items_center()
            .gap_2()
            .px_2()
            .pt_safe_area()
            .border_b_1()
            .border_b_color(colors::LINE)
            .children(self.back.map(|pop| {
                view()
                    .size(px(40.0))
                    .rounded(px(12.0))
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .role(AccessibilityRole::Button)
                    .label("Back")
                    .active(|s| s.bg(colors::RAISED))
                    .on_press(pop)
                    .child(
                        view().rotate(nsh_core::geometry::deg(180.0)).child(
                            icon()
                                .name(IconName::Chevron)
                                .size(20.0)
                                .tint(colors::TEXT)
                                .label(None),
                        ),
                    )
            }))
            .child(
                text(title)
                    .flex_1()
                    .px(px(2.0))
                    .number_of_lines(1)
                    .text_size(px(20.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors::TEXT),
            )
            .children(self.right)
            .into_element()
    }
}
