//! NodeChat's screens. The shell places them: the conversation list is the
//! master (a pushed page on phones, the sidebar on wide windows), the
//! conversation is the detail, and contacts and settings are full pages.

mod chats;
mod contacts;
mod conversation;
mod settings;

pub(crate) use self::{
    chats::{chats_list, chats_page},
    contacts::{contacts_page, new_chat_page},
    conversation::conversation,
    settings::settings_page,
};
