//! NodeChat's Rust UI: its root component, which `nsh run`, `nsh package`
//! and `cargo run` mount.
//!
//! The whole app is Rust on Native Shell (NSH); see [`ui`] for the screens,
//! the shell and the theme.

use nsh_core::prelude::*;

pub mod ui;

pub use crate::ui::App;

/// The app's UI set (`nsh.json` `"rust"`): the components a TSX entry could
/// import from `nodechat/native`, were there one. The app is Rust-only, so
/// this registers what the crate ships for the host to find by name.
#[native_module("nodechat/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}
