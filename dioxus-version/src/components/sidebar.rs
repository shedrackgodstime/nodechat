use crate::helpers::create_chat;
use crate::types::{AppState, Chat, Contact};
use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    let state = use_context::<AppState>();
    let mut current_tab = state.current_tab;
    let mut previous_tab = state.previous_tab;
    let mut active_chat_id = state.active_chat_id;
    let mut search_query = state.search_query;
    let mut show_create_group = state.show_create_group;
    let mut show_add_contact = state.show_add_contact;
    let mut is_locked = state.is_locked;
    let mut show_info_panel = state.show_info_panel;
    let mut mobile_show_chat = state.mobile_show_chat;
    let mut chats = state.chats;
    let contacts = state.contacts;

    // Filter compute
    let search_lower = search_query().to_lowercase();

    let filtered_chats: Vec<Chat> = chats()
        .iter()
        .filter(|c| search_lower.is_empty() || c.name.to_lowercase().contains(&search_lower))
        .cloned()
        .collect();

    let filtered_contacts: Vec<Contact> = contacts()
        .iter()
        .filter(|c| search_lower.is_empty() || c.name.to_lowercase().contains(&search_lower))
        .cloned()
        .collect();

    let group_chats: Vec<Chat> = chats().iter().filter(|c| c.is_group).cloned().collect();

    let sidebar_is_contacts = if current_tab() == "settings" || current_tab() == "diagnostics" {
        previous_tab() == "contacts"
    } else {
        current_tab() == "contacts"
    };

    rsx! {
        div {
            class: format!(
                "w-full md:w-80 lg:w-96 flex-shrink-0 flex flex-col bg-depth-dark border-r border-depth-border transition-all duration-300 {}",
                if mobile_show_chat() { "hidden md:flex" } else { "flex" }
            ),

            // Sidebar Header
            div {
                class: "p-4 flex items-center justify-between border-b border-depth-border bg-depth-dark/50",

                div {
                    class: "flex items-center space-x-3",
                    // App styled logo
                    div {
                        class: "w-9 h-9 rounded-xl bg-depth-black flex items-center justify-center border border-depth-border shadow-inner text-brand-blue font-bold text-sm",
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path { d: "M12 2C6.477 2 2 6.477 2 12c0 1.821.487 3.53 1.338 5.008L2.054 22l5.122-1.312C8.61 21.416 10.246 22 12 22c5.523 0 10-4.477 10-10S17.523 2 12 2zm1 h-2v-2h2v2zm0-4h-2V7h2v5z" }
                        }
                    }
                    div {
                        h1 { class: "font-semibold text-base text-white tracking-tight", "NodeChat" }
                        p { class: "text-[10px] text-brand-blue font-semibold uppercase tracking-wider", "P2P Secure Swarm" }
                    }
                }

                // Quick controls
                div {
                    class: "flex items-center space-x-1.5",

                    // Add Contact Action
                    button {
                        class: "p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent hover:border-depth-border transition-all duration-150",
                        title: "Add Contact",
                        onclick: move |_| { show_add_contact.set(true); },
                        svg {
                            class: "w-4.5 h-4.5 text-brand-blue",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.0",
                            view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                        }
                    }

                    button {
                        class: "p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent hover:border-depth-border transition-all duration-150",
                        title: "Lock Identity",
                        onclick: move |_| { is_locked.set(true); },
                        svg {
                            class: "w-4.5 h-4.5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.8",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z"
                            }
                        }
                    }
                }
            }

            // Search Bar
            div {
                class: "p-3 border-b border-depth-border/30 bg-depth-dark/30",
                div {
                    class: "relative flex items-center bg-depth-black border border-depth-border/60 focus-within:border-brand-blue/70 transition-colors duration-150 rounded-xl px-3 py-2 text-sm",
                    svg {
                        class: "w-4 h-4 text-slate-500 mr-2",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
                        }
                    }
                    input {
                        class: "bg-transparent text-white border-none w-full outline-none placeholder-slate-500",
                        placeholder: "Search chats, peers...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value())
                    }
                }
            }

            // Chat / Contact Scroll List
            div {
                class: "flex-grow overflow-y-auto overscroll-y-contain space-y-1 p-2 bg-depth-dark/20",

                if sidebar_is_contacts {
                    div {
                        class: "flex flex-col space-y-5 p-2",

                        // Quick Actions Card Group
                        div {
                            class: "flex flex-col space-y-1.5",
                            span { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider px-2", "Quick Actions" }
                            div {
                                class: "rounded-2xl bg-depth-black border border-depth-border overflow-hidden divide-y divide-depth-border/40",

                                div {
                                    class: "flex items-center space-x-3.5 p-3 hover:bg-depth-light/30 transition-colors cursor-pointer select-none",
                                    onclick: move |_| show_add_contact.set(true),
                                    div {
                                        class: "w-9 h-9 rounded-xl bg-brand-blue/10 border border-brand-blue/20 text-brand-blue flex items-center justify-center flex-shrink-0",
                                        svg {
                                            class: "w-5 h-5",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.8",
                                            view_box: "0 0 24 24",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235A8.902 8.902 0 0112 15c2.9 0 5.486 1.383 7.126 3.518L19 19.5H5l-1-2.652z" }
                                        }
                                    }
                                    div {
                                        class: "min-w-0 flex-grow",
                                        h4 { class: "font-semibold text-xs text-white", "Add Peer by Ticket" }
                                        p { class: "text-[10px] text-slate-400", "Paste a join code to connect" }
                                    }
                                    span { class: "text-slate-500 text-sm font-medium pr-1", "›" }
                                }

                                div {
                                    class: "flex items-center space-x-3.5 p-3 hover:bg-depth-light/30 transition-colors cursor-pointer select-none",
                                    onclick: move |_| show_create_group.set(true),
                                    div {
                                        class: "w-9 h-9 rounded-xl bg-brand-blue/10 border border-brand-blue/20 text-brand-blue flex items-center justify-center flex-shrink-0",
                                        svg {
                                            class: "w-5 h-5",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.8",
                                            view_box: "0 0 24 24",
                                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M18 18.72a9.094 9.094 0 003.741-.479 3 3 0 00-4.682-2.72m.94 3.198l.001.031c0 .225-.012.447-.037.666A11.944 11.944 0 0112 21c-2.17 0-4.207-.576-5.963-1.584A6.062 6.062 0 016 18.719m12 0a5.97 5.97 0 00-.75-2.906m-.173-1.036a5.454 5.454 0 01-1.077-3.07 3.375 3.375 0 00-6 0 5.455 5.455 0 01-1.077 3.07M6 18.72a9.094 9.094 0 01-3.741-.479 3 3 0 014.682-2.72m-.94 3.198l-.001.031c0 .225.012.447.037.666A11.944 11.944 0 0012 21c2.17 0 4.207-.576 5.963-1.584A6.06 6.06 0 0118 18.722zm-12 0a5.97 5.97 0 01.75-2.906m-.75 2.906A3.003 3.003 0 016 18.72" }
                                        }
                                    }
                                    div {
                                        class: "min-w-0 flex-grow",
                                        h4 { class: "font-semibold text-xs text-white", "Create New Group Chat" }
                                        p { class: "text-[10px] text-slate-400", "Broadcast to multiple peers" }
                                    }
                                    span { class: "text-slate-500 text-sm font-medium pr-1", "›" }
                                }
                            }
                        }

                        if !group_chats.is_empty() {
                            div {
                                class: "flex flex-col space-y-1.5",
                                span { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider px-2", "Groups" }
                                div {
                                    class: "space-y-1",
                                    for group in group_chats.iter() {
                                        div {
                                            key: "{group.id}",
                                            class: "flex items-center justify-between p-3 rounded-2xl hover:bg-depth-light/30 border border-transparent hover:border-depth-border/30 transition-all duration-200 cursor-pointer",
                                            onclick: {
                                                let group_id = group.id.clone();
                                                move |_| {
                                                    active_chat_id.set(Some(group_id.clone()));
                                                    show_info_panel.set(false);
                                                    current_tab.set("chats".to_string());
                                                    mobile_show_chat.set(true);
                                                }
                                            },
                                            div {
                                                class: "flex items-center space-x-3 min-w-0 flex-grow",
                                                div {
                                                    class: "w-9 h-9 rounded-xl bg-teal-600/20 border border-teal-500/30 text-teal-400 flex items-center justify-center text-xs font-bold flex-shrink-0",
                                                    "{group.initials}"
                                                }
                                                div {
                                                    class: "min-w-0 flex-grow",
                                                    h4 { class: "font-semibold text-xs text-white truncate", "{group.name}" }
                                                    p { class: "text-[10px] text-slate-400 truncate", "3 members active" }
                                                }
                                            }
                                            span { class: "text-slate-500 text-sm font-medium pr-1", "›" }
                                        }
                                    }
                                }
                            }
                        }

                        div {
                            class: "flex flex-col space-y-1.5",
                            span { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider px-2", "Direct Contacts" }

                            if filtered_contacts.is_empty() {
                                div {
                                    class: "p-4 rounded-2xl bg-depth-black border border-depth-border/60 text-center select-none",
                                    if search_query().trim().is_empty() {
                                        span { class: "text-slate-500 text-xs", "No peers linked yet." }
                                    } else {
                                        span { class: "text-slate-500 text-xs", "No contacts match \"{search_query}\"" }
                                    }
                                }
                            } else {
                                div {
                                    class: "space-y-1",
                                    for contact in filtered_contacts.iter() {
                                        div {
                                            key: "{contact.id}",
                                            class: "flex items-center justify-between p-3 rounded-2xl hover:bg-depth-light/30 border border-transparent hover:border-depth-border/30 transition-all duration-200 cursor-pointer",
                                            onclick: {
                                                let contact_id = contact.id.clone();
                                                let contact_name = contact.name.clone();
                                                let contact_initials = contact.initials.clone();
                                                let contact_online = contact.is_online;
                                                let contact_verified = contact.is_verified;
                                                move |_| {
                                                    let mut list = chats();
                                                    let has_chat = list.iter().any(|c| c.id == contact_id);
                                                    if !has_chat {
                                                        list.push(create_chat(&contact_id, &contact_name, &contact_initials, false, contact_online, contact_verified));
                                                        chats.set(list);
                                                    }
                                                    active_chat_id.set(Some(contact_id.clone()));
                                                    show_info_panel.set(true);
                                                    current_tab.set("chats".to_string());
                                                    mobile_show_chat.set(true);
                                                }
                                            },
                                            div {
                                                class: "flex items-center space-x-3 min-w-0 flex-grow",
                                                div {
                                                    class: "relative flex-shrink-0",
                                                    div {
                                                        class: format!(
                                                            "w-9 h-9 rounded-xl bg-depth-light border border-depth-border/50 flex items-center justify-center text-slate-300 text-xs font-semibold {}",
                                                            if contact.is_verified { "border-brand-blue/30 text-brand-blue bg-brand-blue/5" } else { "" }
                                                        ),
                                                        "{contact.initials}"
                                                    }
                                                    if contact.is_online {
                                                        div { class: "absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-emerald-500 rounded-full border-2 border-depth-dark shadow-sm shadow-emerald-500/50" }
                                                    }
                                                }
                                                div {
                                                    class: "min-w-0 flex-grow",
                                                    div {
                                                        class: "flex items-center space-x-1.5",
                                                        h4 { class: "font-semibold text-xs text-white truncate", "{contact.name}" }
                                                        if contact.is_verified {
                                                            svg {
                                                                class: "w-3 h-3 text-brand-blue flex-shrink-0",
                                                                fill: "currentColor",
                                                                view_box: "0 0 20 20",
                                                                path {
                                                                    fill_rule: "evenodd",
                                                                    d: "M10 1.944A11.954 11.954 0 012.166 5C2.056 5.649 2 6.319 2 7c0 5.222 3.293 9.68 7.88 11.23a.75.75 0 00.24 0C14.707 16.68 18 12.222 18 7c0-.681-.056-1.35-.166-2C14.15 3.32 10.85 1.943 10 1.944zm3.97 6.03a.75.75 0 00-1.08-1.04l-3.25 3.5-1.5-1.5a.75.75 0 10-1.08 1.04l2 2a.75.75 0 001.08 0l3.75-4z",
                                                                    clip_rule: "evenodd"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    p { class: "text-[9px] text-slate-500 truncate font-mono", "{contact.node_id}" }
                                                }
                                            }
                                            span { class: "text-slate-500 text-sm font-medium pr-1", "›" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div {
                        class: "flex flex-col space-y-1",
                        if filtered_chats.is_empty() {
                            if search_query().trim().is_empty() {
                                div { class: "text-center text-slate-500 py-8 text-sm", "No conversations yet" }
                            } else {
                                div { class: "text-center text-slate-500 py-8 text-sm", "No chats match \"{search_query}\"" }
                            }
                        } else {
                            for chat in filtered_chats.iter() {
                                div {
                                    key: "{chat.id}",
                                    class: format!(
                                        "flex items-center justify-between p-3 rounded-2xl cursor-pointer transition-all duration-200 border {}",
                                        if Some(chat.id.clone()) == active_chat_id() {
                                            "bg-depth-light/80 border-depth-border shadow-lg shadow-black/10"
                                        } else {
                                            "bg-transparent border-transparent hover:bg-depth-light/30 hover:border-depth-border/30"
                                        }
                                    ),
                                    onclick: {
                                        let chat_id = chat.id.clone();
                                        move |_| {
                                            active_chat_id.set(Some(chat_id.clone()));
                                            show_info_panel.set(false);
                                            mobile_show_chat.set(true);
                                            let mut list = chats();
                                            if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
                                                list[pos].unread = 0;
                                                chats.set(list);
                                            }
                                        }
                                    },
                                    div {
                                        class: "flex items-center space-x-3 min-w-0 flex-grow",
                                        div {
                                            class: "relative flex-shrink-0",
                                            div {
                                                class: format!(
                                                    "w-11 h-11 rounded-2xl flex items-center justify-center text-white text-sm font-semibold shadow-inner border {}",
                                                    if chat.is_group {
                                                        "bg-teal-600/20 border-teal-500/30 text-teal-400"
                                                    } else if chat.is_verified {
                                                        "bg-brand-blue/20 border-brand-blue/30 text-brand-blue"
                                                    } else {
                                                        "bg-depth-light border-depth-border/50 text-slate-300"
                                                    }
                                                ),
                                                "{chat.initials}"
                                            }
                                            if chat.is_online {
                                                div { class: "absolute -bottom-0.5 -right-0.5 w-3.5 h-3.5 bg-emerald-500 rounded-full border-2 border-depth-dark shadow-sm shadow-emerald-500/50" }
                                            }
                                        }
                                        div {
                                            class: "min-w-0 flex-grow",
                                            div {
                                                class: "flex items-center space-x-1.5",
                                                h3 { class: "font-medium text-sm text-white truncate", "{chat.name}" }
                                                if chat.is_verified {
                                                    svg {
                                                        class: "w-3.5 h-3.5 text-brand-blue flex-shrink-0",
                                                        fill: "currentColor",
                                                        view_box: "0 0 20 20",
                                                        path {
                                                            fill_rule: "evenodd",
                                                            d: "M10 1.944A11.954 11.954 0 012.166 5C2.056 5.649 2 6.319 2 7c0 5.222 3.293 9.68 7.88 11.23a.75.75 0 00.24 0C14.707 16.68 18 12.222 18 7c0-.681-.056-1.35-.166-2C14.15 3.32 10.85 1.943 10 1.944zm3.97 6.03a.75.75 0 00-1.08-1.04l-3.25 3.5-1.5-1.5a.75.75 0 10-1.08 1.04l2 2a.75.75 0 001.08 0l3.75-4z",
                                                            clip_rule: "evenodd"
                                                        }
                                                    }
                                                }
                                            }
                                            p {
                                                class: format!(
                                                    "text-xs truncate {}",
                                                    if chat.unread > 0 { "text-brand-blue font-medium" } else { "text-slate-400" }
                                                ),
                                                "{chat.last_message}"
                                            }
                                        }
                                    }
                                    div {
                                        class: "flex flex-col items-end space-y-1.5 ml-2 flex-shrink-0",
                                        span { class: "text-[10px] text-slate-500", "{chat.timestamp}" }
                                        if chat.unread > 0 {
                                            span {
                                                class: "flex h-5 min-w-5 px-1 items-center justify-center rounded-full bg-brand-blue text-[10px] font-bold text-white shadow-sm shadow-brand-blue/30 animate-pulse",
                                                "{chat.unread}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Sidebar Footer Tabs
            div {
                class: "p-2 border-t border-depth-border bg-depth-dark flex justify-around",

                button {
                    class: format!(
                        "flex flex-col items-center flex-grow py-2 rounded-xl transition-all duration-200 {}",
                        if current_tab() == "chats" { "text-brand-blue bg-brand-blue/5 font-semibold" } else { "text-slate-400 hover:text-slate-200" }
                    ),
                    onclick: move |_| current_tab.set("chats".to_string()),
                    svg {
                        class: "w-5 h-5 mb-1",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 01.865-.501 48.172 48.172 0 003.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z"
                        }
                    }
                    span { class: "text-[10px]", "Chats" }
                }

                button {
                    class: format!(
                        "flex flex-col items-center flex-grow py-2 rounded-xl transition-all duration-200 {}",
                        if current_tab() == "contacts" { "text-brand-blue bg-brand-blue/5 font-semibold" } else { "text-slate-400 hover:text-slate-200" }
                    ),
                    onclick: move |_| current_tab.set("contacts".to_string()),
                    svg {
                        class: "w-5 h-5 mb-1",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.109A9.342 9.342 0 0012.24 20.5a9.34 9.34 0 00-2.24-.128v-.109m0-1.142c0-1.113.285-2.16.786-3.07M9 19.128a9.38 9.38 0 01-2.625.372 9.337 9.337 0 01-4.121-.952 4.125 4.125 0 017.533-2.493M9 19.128v-.003c0-1.113.285-2.16.786-3.07m0 0A3.375 3.375 0 109 8.25a3.375 3.375 0 000 6.75zm3-1.875a3.375 3.375 0 100-6.75 3.375 3.375 0 000 6.75zM2.625 19.128a9.337 9.337 0 014.121-.952m14.629.952a9.337 9.337 0 00-4.121-.952"
                        }
                    }
                    span { class: "text-[10px]", "Contacts" }
                }

                button {
                    class: format!(
                        "flex flex-col items-center flex-grow py-2 rounded-xl transition-all duration-200 {}",
                        if current_tab() == "settings" { "text-brand-blue bg-brand-blue/5 font-semibold" } else { "text-slate-400 hover:text-slate-200" }
                    ),
                    onclick: move |_| {
                        previous_tab.set(current_tab());
                        active_chat_id.set(None);
                        current_tab.set("settings".to_string());
                        mobile_show_chat.set(true);
                    },
                    svg {
                        class: "w-5 h-5 mb-1",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.43l-1.003.828c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.214-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.43l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.991l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.28z"
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        }
                    }
                    span { class: "text-[10px]", "Settings" }
                }
            }
        }
    }
}
