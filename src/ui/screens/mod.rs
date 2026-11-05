//! NodeChat's screens: each is a function returning the screen's content, on
//! the shared parts and the store in [`crate::ui::data`]. The shell places
//! them (a pushed screen on phones, the open pane on wide windows).

mod chats;
mod contacts;
mod conversation;
mod settings;

pub(crate) use self::{
    chats::chats_list, contacts::contacts_list, conversation::conversation,
    settings::settings_list,
};
