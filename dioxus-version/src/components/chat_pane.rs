use crate::helpers::{send_message, sleep_ms};
use crate::types::{AppState, Chat, Message};
use dioxus::prelude::*;

// Standalone send message helper function to avoid closure capture lifetime errors
fn perform_send(
    mut chats: Signal<Vec<Chat>>,
    chat_id: String,
    chat_online: bool,
    mut message_input: Signal<String>,
    mut is_typing: Signal<bool>,
) {
    if message_input().trim().is_empty() {
        return;
    }
    let text = message_input().trim().to_string();
    message_input.set(String::new());

    let mut list = chats();
    if let Some((msg_id, _)) = send_message(&mut list, &chat_id, &text, chat_online) {
        chats.set(list);

        if chat_online {
            let cid = chat_id.clone();
            let mid = msg_id.clone();

            spawn(async move {
                // Transition to "sent" (single check mark)
                sleep_ms(400).await;
                let mut l = chats();
                if let Some(p) = l.iter().position(|c| c.id == cid) {
                    if let Some(m) = l[p].messages.iter().position(|x| x.id == mid) {
                        l[p].messages[m].status = "sent".to_string();
                        chats.set(l);
                    }
                }

                // Transition to "delivered" (double check marks grey)
                sleep_ms(600).await;
                let mut l = chats();
                if let Some(p) = l.iter().position(|c| c.id == cid) {
                    if let Some(m) = l[p].messages.iter().position(|x| x.id == mid) {
                        l[p].messages[m].status = "delivered".to_string();
                        chats.set(l);
                    }
                }

                // Transition to "read" (double check marks blue)
                sleep_ms(500).await;
                let mut l = chats();
                if let Some(p) = l.iter().position(|c| c.id == cid) {
                    if let Some(m) = l[p].messages.iter().position(|x| x.id == mid) {
                        l[p].messages[m].status = "read".to_string();
                        chats.set(l);
                    }
                }

                // Simulate typing indicator and mock response
                sleep_ms(800).await;
                is_typing.set(true);

                sleep_ms(1800).await;
                is_typing.set(false);

                let mock_responses = [
                    "ED25519 signature verified. We are safe.",
                    "Direct UDP tunnel is super stable. Iroh transport works!",
                    "I'm testing the Dioxus UI frontend now, it feels extremely smooth.",
                    "Gossip Swarm looks optimal. Synchronization completed successfully.",
                    "Checked the WAL logs, 0 packet loss on local database checkpoint.",
                    "Sure! Let's check my safety numbers to verify identity keys.",
                ];

                let idx = (tokio::time::Instant::now().elapsed().as_nanos()
                    % mock_responses.len() as u128) as usize;
                let reply_text = mock_responses[idx].to_string();

                let mut l = chats();
                if let Some(p) = l.iter().position(|c| c.id == cid) {
                    let sender = l[p].name.clone();
                    let rep_id = format!("m_reply_{}", l[p].messages.len());
                    l[p].messages.push(Message {
                        id: rep_id,
                        sender,
                        text: reply_text,
                        timestamp: "Just now".to_string(),
                        is_outgoing: false,
                        status: "read".to_string(),
                        ..Default::default()
                    });
                    l[p].last_message = mock_responses[idx].to_string();
                    chats.set(l);
                }
            });
        }
    }
}

#[component]
pub fn ChatPane() -> Element {
    let state = use_context::<AppState>();
    let mut chats = state.chats;
    let active_chat_id = state.active_chat_id;
    let mut message_input = state.message_input;
    let mut show_info_panel = state.show_info_panel;
    let mut mobile_show_chat = state.mobile_show_chat;
    let display_name_input = state.display_name_input;

    // Local typing indicator state
    let is_typing = use_signal(|| false);

    let active_chat = active_chat_id().and_then(|id| chats().iter().find(|c| c.id == id).cloned());

    let Some(chat) = active_chat else {
        return rsx! {};
    };

    // Auto-scroll chat messages to bottom
    let msg_count = chat.messages.len();
    let chat_id_str_for_effect = chat.id.clone();
    use_effect(move || {
        let _dep = (msg_count, chat_id_str_for_effect.clone());
        spawn(async move {
            sleep_ms(50).await;
            document::eval(
                r#"
                var el = document.getElementById('chat-messages');
                if (el) el.scrollTop = el.scrollHeight;
            "#,
            );
        });
    });

    // Prepare cloned keys to avoid move conflicts
    let chat_id_str_for_invite = chat.id.clone();
    let chat_id_for_send_key = chat.id.clone();
    let chat_id_for_send_btn = chat.id.clone();
    let chat_id_for_share = chat.id.clone();
    let chat_online = chat.is_online;

    rsx! {
        // Horizontal split layout container (Workspace + Slide Info drawer)
        div {
            class: "flex flex-row w-full h-full min-h-0 min-w-0 relative",

            // Left Side: Chat Workspace Pane
            div {
                class: "flex-grow h-full min-h-0 min-w-0 flex flex-col bg-depth-black",

                // Chat Header
                div {
                    class: "h-16 flex items-center justify-between px-6 border-b border-depth-border bg-depth-dark/80 backdrop-blur-md sticky top-0 z-10",

                    div {
                        class: "flex items-center space-x-3.5 min-w-0 flex-grow cursor-pointer select-none",
                        onclick: move |_| show_info_panel.toggle(),

                        // Back button (Mobile only)
                        button {
                            class: "md:hidden p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent active:scale-95 transition-all duration-100",
                            onclick: move |evt| {
                                evt.stop_propagation(); // Prevent drawer toggle
                                mobile_show_chat.set(false);
                            },
                            svg {
                                class: "w-5 h-5",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M15.75 19.5L8.25 12l7.5-7.5"
                                }
                            }
                        }

                        // Avatar
                        div {
                            class: format!(
                                "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-xs font-semibold shadow-inner border {}",
                                if chat.is_group { "bg-teal-600/20 border-teal-500/30 text-teal-400" }
                                else if chat.is_verified { "bg-brand-blue/20 border-brand-blue/30 text-brand-blue" }
                                else { "bg-depth-light border-depth-border/50 text-slate-300" }
                            ),
                            "{chat.initials}"
                        }

                        div {
                            class: "min-w-0 flex-grow",
                            div {
                                class: "flex items-center space-x-1.5",
                                h3 { class: "font-semibold text-sm text-white truncate", "{chat.name}" }
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
                                    "text-[10px] font-semibold uppercase tracking-wider {}",
                                    if chat.is_online { "text-emerald-400" } else { "text-slate-500" }
                                ),
                                if chat.is_group { "Swarms gossiping" } else if chat.is_online { "Direct connection active" } else { "Offline" }
                            }
                        }
                    }

                    button {
                        class: "p-2.5 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent hover:border-depth-border active:scale-95 transition-all duration-100",
                        title: "Conversation Details",
                        onclick: move |_| show_info_panel.toggle(),
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.8",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M11.25 11.25l.041-.02a.75.75 0 111.063.852l-.708 2.836a.75.75 0 001.063.852l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z"
                            }
                        }
                    }
                }

                // Message List
                div {
                    id: "chat-messages",
                    class: "flex-grow overflow-y-auto overscroll-y-contain p-6 space-y-4 bg-depth-black/20 scrollbar-thin scrollbar-thumb-depth-border scroll-smooth",

                    if chat.messages.is_empty() {
                        div {
                            class: "h-full flex flex-col items-center justify-center text-center p-8 space-y-3",
                            div {
                                class: "w-12 h-12 rounded-xl bg-depth-light border border-depth-border flex items-center justify-center text-slate-500",
                                svg {
                                    class: "w-6 h-6",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "1.8",
                                    view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" }
                                }
                            }
                            h4 { class: "font-semibold text-xs text-slate-400", "Swarm Session Established" }
                            p { class: "text-[10px] text-slate-500 max-w-[200px] leading-relaxed", "Chat messages are local-only and ratcheted end-to-end securely." }
                        }
                    } else {
                        for msg in chat.messages.iter() {
                            if msg.is_system {
                                // System indicator banner
                                div {
                                    key: "{msg.id}",
                                    class: "flex justify-center my-2",
                                    div {
                                        class: "px-3.5 py-1.5 rounded-full bg-depth-dark border border-depth-border/70 text-slate-400 font-mono text-[9px] uppercase tracking-wider flex items-center space-x-1.5 shadow-sm",
                                        span { "{msg.text}" }
                                    }
                                }
                            } else if msg.is_invite {
                                // Group Swarm Invite card
                                div {
                                    key: "{msg.id}",
                                    class: "flex flex-col max-w-[75%] items-start text-left bg-depth-dark border border-brand-blue/30 rounded-2xl p-4 space-y-3.5 shadow-lg shadow-black/10 animate-fade-in",

                                    div {
                                        class: "flex items-start space-x-3",
                                        div {
                                            class: "w-9 h-9 rounded-xl bg-brand-blue/10 border border-brand-blue/20 text-brand-blue flex items-center justify-center flex-shrink-0 mt-0.5",
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
                                            h4 { class: "font-semibold text-xs text-white", "Swarm Group Invitation" }
                                            p { class: "text-[10px] text-slate-400 leading-relaxed mt-0.5", "Received from {msg.sender}" }
                                        }
                                    }

                                    div {
                                        class: "p-3 rounded-xl bg-depth-black/80 border border-depth-border space-y-1",
                                        h5 { class: "font-bold text-xs text-white", "{msg.invite_group_name}" }
                                        p { class: "text-[10px] text-slate-400 leading-relaxed", "{msg.invite_desc}" }
                                        div { class: "text-[9px] font-mono text-slate-500 truncate mt-1 select-all", "Topic: {msg.invite_topic_id}" }
                                    }

                                    div {
                                        class: "w-full pt-1.5",
                                        if msg.invite_joined {
                                            div {
                                                class: "w-full py-2 bg-depth-light border border-depth-border text-slate-500 rounded-xl text-center text-xs font-semibold flex items-center justify-center space-x-1.5",
                                                svg {
                                                    class: "w-4.5 h-4.5 text-emerald-500",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2.5",
                                                    view_box: "0 0 24 24",
                                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 12.75l6 6 9-13.5" }
                                                }
                                                span { "Joined Swarm" }
                                            }
                                        } else {
                                            button {
                                                class: "w-full py-2 bg-brand-blue hover:bg-brand-blue-hover text-white rounded-xl text-xs font-semibold transition-all active:scale-95 shadow-md shadow-brand-blue/15",
                                                onclick: {
                                                    let msg_id_clone = msg.id.clone();
                                                    let invite_group_name_clone = msg.invite_group_name.clone();
                                                    let chat_id_for_click = chat_id_str_for_invite.clone();
                                                    move |_| {
                                                        let mut list = chats();
                                                        if let Some(pos) = list.iter().position(|c| c.id == chat_id_for_click) {
                                                            if let Some(m_pos) = list[pos].messages.iter().position(|m| m.id == msg_id_clone) {
                                                                // Set joined status
                                                                list[pos].messages[m_pos].invite_joined = true;

                                                                // Create a new chat for this group
                                                                let group_id = "group_".to_string() + &invite_group_name_clone.to_lowercase().replace(' ', "_");
                                                                let has_group_chat = list.iter().any(|c| c.id == group_id);
                                                                if !has_group_chat {
                                                                    let initials: String = invite_group_name_clone.chars().take(2).collect::<String>().to_uppercase();
                                                                    list.push(Chat {
                                                                        id: group_id,
                                                                        name: invite_group_name_clone.clone(),
                                                                        initials,
                                                                        last_message: "You joined the group via invitation.".to_string(),
                                                                        timestamp: "Just now".to_string(),
                                                                        unread: 0,
                                                                        is_group: true,
                                                                        is_online: true,
                                                                        is_verified: false,
                                                                        has_queued: false,
                                                                        messages: vec![
                                                                             Message {
                                                                                id: "g1".to_string(),
                                                                                sender: "Charlie Chat".to_string(),
                                                                                text: "Welcome to the group swarm!".to_string(),
                                                                                timestamp: "12:00 PM".to_string(),
                                                                                is_outgoing: false,
                                                                                status: "read".to_string(),
                                                                                ..Default::default()
                                                                            }
                                                                        ]
                                                                    });
                                                                }
                                                                chats.set(list);
                                                            }
                                                        }
                                                    }
                                                },
                                                span { "Accept & Join Group Swarm" }
                                            }
                                        }
                                    }
                                }
                            } else if msg.is_contact_share {
                                div {
                                    key: "{msg.id}",
                                    class: format!(
                                        "flex flex-col max-w-[75%] {} items-start text-left bg-depth-dark border border-brand-blue/30 rounded-2xl p-4 space-y-3.5 shadow-lg shadow-black/10 animate-fade-in",
                                        if msg.is_outgoing { "ml-auto" } else { "" }
                                    ),

                                    div {
                                        class: "flex items-start space-x-3 w-full",
                                        div {
                                            class: "w-9 h-9 rounded-xl bg-brand-blue/10 border border-brand-blue/20 text-brand-blue flex items-center justify-center flex-shrink-0 mt-0.5",
                                            svg {
                                                class: "w-5 h-5",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "1.8",
                                                view_box: "0 0 24 24",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                                            }
                                        }
                                        div {
                                            class: "min-w-0 flex-grow",
                                            h4 { class: "font-semibold text-xs text-slate-400 uppercase tracking-wider", "👤 Shared Contact" }
                                            h3 { class: "font-bold text-sm text-white truncate mt-0.5", "{msg.share_name}" }
                                        }
                                    }

                                    div {
                                        class: "w-full p-3 rounded-xl bg-depth-black/80 border border-depth-border space-y-1.5",
                                        div { class: "text-[9px] text-slate-500 font-bold uppercase tracking-wider", "Node Connection Ticket" }
                                        div { class: "text-[10px] font-mono text-slate-300 break-all select-all leading-normal bg-depth-dark/40 p-2 rounded-lg border border-depth-border/30", "{msg.share_ticket}" }
                                    }

                                    div {
                                        class: "w-full pt-1.5",
                                        button {
                                            class: "w-full py-2 bg-brand-blue hover:bg-brand-blue-hover text-white rounded-xl text-xs font-semibold transition-all active:scale-95 shadow-md shadow-brand-blue/15 flex items-center justify-center space-x-1.5",
                                            onclick: {
                                                let share_name = msg.share_name.clone();
                                                let share_ticket = msg.share_ticket.clone();
                                                let share_node_id = msg.share_node_id.clone();
                                                let mut contacts = state.contacts;
                                                let mut logs = state.logs;
                                                move |_| {
                                                    let mut list = contacts.write();
                                                    let already_exists = list.iter().any(|c| c.node_id == share_ticket || c.id == share_node_id);
                                                    if !already_exists {
                                                        list.push(crate::types::Contact {
                                                            id: share_name.to_lowercase().replace(' ', "_"),
                                                            name: share_name.clone(),
                                                            initials: share_name.chars().take(2).collect::<String>().to_uppercase(),
                                                            node_id: share_ticket.clone(),
                                                            is_online: true,
                                                            is_verified: false,
                                                        });
                                                        logs.write().push(format!("[P2P] Added peer from shared contact card: {}", share_name));
                                                    }
                                                }
                                            },
                                            svg {
                                                class: "w-4 h-4",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2.0",
                                                view_box: "0 0 24 24",
                                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235A8.902 8.902 0 0112 15c2.9 0 5.486 1.383 7.126 3.518L19 19.5H5l-1-2.652z" }
                                            }
                                            span { "Add Connection" }
                                        }
                                    }
                                }
                            } else {
                                // Standard message bubble
                                div {
                                    key: "{msg.id}",
                                    class: format!(
                                        "flex flex-col max-w-[70%] {}",
                                        if msg.is_outgoing { "items-end ml-auto" } else { "items-start" }
                                    ),

                                    // Sender name (group only)
                                    if chat.is_group && !msg.is_outgoing {
                                        span { class: "text-[10px] text-slate-400 mb-1 px-1.5 font-medium", "{msg.sender}" }
                                    }

                                    // Message Text Bubble
                                    div {
                                        class: format!(
                                            "rounded-2xl px-4 py-2.5 shadow-md border {}",
                                            if msg.is_outgoing {
                                                "bg-brand-blue border-brand-blue-hover text-white rounded-tr-none"
                                            } else {
                                                "bg-depth-light border-depth-border/30 text-slate-100 rounded-tl-none"
                                            }
                                        ),
                                        p { class: "text-sm leading-relaxed break-words", "{msg.text}" }
                                    }

                                    // Timestamp and status info
                                    div {
                                        class: "flex items-center space-x-1.5 mt-1 px-1",
                                        span { class: "text-[10px] text-slate-500", "{msg.timestamp}" }

                                        if msg.is_outgoing {
                                            if msg.status == "queued" {
                                                div { class: "w-3 h-3 border-[1.5px] border-slate-500 border-t-transparent rounded-full animate-spin flex-shrink-0" }
                                            } else if msg.status == "sent" {
                                                svg {
                                                    class: "w-3.5 h-3.5 text-slate-500",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2.0",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        d: "M4.5 12.75l6 6 9-13.5"
                                                    }
                                                }
                                            } else if msg.status == "delivered" {
                                                // Double checks in slate color (Delivered check marks)
                                                div {
                                                    class: "flex -space-x-1.5 text-slate-500",
                                                    svg {
                                                        class: "w-3.5 h-3.5",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "2.5",
                                                        view_box: "0 0 24 24",
                                                        path {
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                            d: "M4.5 12.75l6 6 9-13.5"
                                                        }
                                                    }
                                                    svg {
                                                        class: "w-3.5 h-3.5",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "2.5",
                                                        view_box: "0 0 24 24",
                                                        path {
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                            d: "M4.5 12.75l6 6 9-13.5"
                                                        }
                                                    }
                                                }
                                            } else if msg.status == "read" {
                                                // Double checks in brand color
                                                div {
                                                    class: "flex -space-x-1.5 text-brand-blue",
                                                    svg {
                                                        class: "w-3.5 h-3.5",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "2.5",
                                                        view_box: "0 0 24 24",
                                                        path {
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                            d: "M4.5 12.75l6 6 9-13.5"
                                                        }
                                                    }
                                                    svg {
                                                        class: "w-3.5 h-3.5",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "2.5",
                                                        view_box: "0 0 24 24",
                                                        path {
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                            d: "M4.5 12.75l6 6 9-13.5"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Render typing indicator bubble
                    if is_typing() {
                        div {
                            class: "flex flex-col max-w-[70%] items-start animate-pulse",
                            div {
                                class: "rounded-2xl px-4 py-2.5 shadow-md border bg-depth-light border-depth-border/30 text-slate-400 rounded-tl-none flex items-center space-x-1.5",
                                div { class: "w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" }
                                div { class: "w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce delay-75" }
                                div { class: "w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce delay-150" }
                            }
                        }
                    }
                }

                // Bottom Input Bar
                div {
                    class: "p-4 border-t border-depth-border bg-depth-dark/80 backdrop-blur-md flex items-center space-x-3.5",

                    // Quick action buttons
                    button {
                        class: "p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent hover:border-depth-border transition-all active:scale-95 duration-100",
                        title: "Share Contact card",
                        onclick: move |__evt| {
                            let mut list = chats();
                            let user_name = display_name_input();
                            let payload = serde_json::json!({
                                "name": user_name,
                                "node_id": "iroh://peer/self_node_id_hash",
                                "ticket": "iroh://ticket/self_ticket_value"
                            }).to_string();
                            send_message(&mut list, &chat_id_for_share, &payload, true);
                            chats.set(list);
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.8",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M19.114 5.636a9 9 0 010 12.728M16.463 8.288a5.25 5.25 0 010 7.424M6.75 8.25l4.72-4.72a.75.75 0 011.28-5.33v15.88a.75.75 0 01-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.01 9.01 0 012.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75z"
                            }
                        }
                    }

                    // Text Input container
                    div {
                        class: "flex-grow relative flex items-center bg-depth-black border border-depth-border focus-within:border-brand-blue/70 transition-colors duration-150 rounded-xl px-4 py-2.5 text-sm",

                        input {
                            class: "bg-transparent text-white border-none w-full outline-none placeholder-slate-500",
                            placeholder: "Type a secure message...",
                            value: "{message_input}",
                            oninput: move |evt| message_input.set(evt.value()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter && !message_input().trim().is_empty() {
                                    perform_send(chats, chat_id_for_send_key.clone(), chat_online, message_input, is_typing);
                                }
                            }
                        }
                    }

                    // Send Button
                    button {
                        class: "p-2.5 rounded-xl bg-brand-blue hover:bg-brand-blue-hover text-white transition-all active:scale-95 duration-100 flex items-center justify-center flex-shrink-0 shadow-md shadow-brand-blue/20",
                        onclick: move |_| {
                            perform_send(chats, chat_id_for_send_btn.clone(), chat_online, message_input, is_typing);
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.8",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5"
                            }
                        }
                    }
                }
            }
        }
    }
}
