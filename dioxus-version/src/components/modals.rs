use crate::types::{AppState, Chat, Contact, Message};
use dioxus::prelude::*;

#[component]
pub fn Modals() -> Element {
    let state = use_context::<AppState>();
    let mut show_create_group = state.show_create_group;
    let mut show_add_contact = state.show_add_contact;
    let mut group_name_input = state.group_name_input;
    let mut group_desc_input = state.group_desc_input;
    let mut selected_group_contacts = state.selected_group_contacts;
    let mut contact_ticket_input = state.contact_ticket_input;
    let mut chats = state.chats;
    let mut contacts = state.contacts;
    let mut active_chat_id = state.active_chat_id;
    let mut mobile_show_chat = state.mobile_show_chat;

    rsx! {
        // Modal: Create Group Chat Dialog
        if show_create_group() {
            div {
                class: "absolute inset-0 z-40 flex items-center justify-center bg-black/70 backdrop-blur-sm",

                div {
                    class: "w-full max-w-md p-6 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl text-left",

                    h3 { class: "text-lg font-bold text-white mb-2", "Create Group Swarm" }
                    p { class: "text-slate-400 text-xs mb-5", "Initialize a symmetric-key encrypted chat room over iroh-gossip." }

                    div {
                        class: "space-y-4 mb-6",

                        // Name input
                        div {
                            class: "flex flex-col space-y-1.5",
                            label { class: "text-xs text-slate-400 font-medium", "Group Name" }
                            input {
                                class: "bg-depth-black border border-depth-border rounded-xl px-3.5 py-2.5 text-sm text-white focus:border-brand-blue outline-none transition-colors duration-150",
                                placeholder: "e.g. Gossip Swarm",
                                value: "{group_name_input}",
                                oninput: move |evt| group_name_input.set(evt.value())
                            }
                        }

                        // Description input
                        div {
                            class: "flex flex-col space-y-1.5",
                            label { class: "text-xs text-slate-400 font-medium", "Group Description" }
                            input {
                                class: "bg-depth-black border border-depth-border rounded-xl px-3.5 py-2.5 text-sm text-white focus:border-brand-blue outline-none transition-colors duration-150",
                                placeholder: "What is this swarm topic?",
                                value: "{group_desc_input}",
                                oninput: move |evt| group_desc_input.set(evt.value())
                            }
                        }

                        // Member Picker List
                        div {
                            class: "flex flex-col space-y-1.5",
                            label { class: "text-xs text-slate-400 font-medium", "Select Members" }
                            div {
                                class: "max-h-40 overflow-y-auto overscroll-y-contain border border-depth-border/60 rounded-xl p-2 bg-depth-black/40 space-y-1",

                                for contact in contacts().iter() {
                                    {
                                        let is_checked = selected_group_contacts().contains(&contact.id);
                                        let contact_id_clone = contact.id.clone();
                                        rsx! {
                                            div {
                                                key: "{contact.id}",
                                                class: "flex items-center justify-between p-2 rounded-lg hover:bg-depth-light/40 transition-colors text-xs cursor-pointer",
                                                onclick: move |_| {
                                                    let mut list = selected_group_contacts();
                                                    if let Some(pos) = list.iter().position(|id| *id == contact_id_clone) {
                                                        list.remove(pos);
                                                    } else {
                                                        list.push(contact_id_clone.clone());
                                                    }
                                                    selected_group_contacts.set(list);
                                                },

                                                div {
                                                    class: "flex items-center space-x-2.5",
                                                    div {
                                                        class: "w-7 h-7 rounded-lg bg-depth-light border border-depth-border/50 text-[10px] text-white flex items-center justify-center font-bold",
                                                        "{contact.initials}"
                                                    }
                                                    span { class: "text-slate-200", "{contact.name}" }
                                                }

                                                // Styled checkbox
                                                div {
                                                    class: format!(
                                                        "w-4.5 h-4.5 rounded border flex items-center justify-center transition-colors {}",
                                                        if is_checked { "bg-brand-blue border-brand-blue" } else { "border-depth-border bg-depth-light" }
                                                    ),
                                                    if is_checked {
                                                        svg {
                                                            class: "w-3 h-3 text-white",
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            stroke_width: "3.0",
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
                    }

                    // Actions buttons
                    div {
                        class: "flex space-x-3",
                        button {
                            class: "flex-grow py-2.5 rounded-xl border border-depth-border hover:bg-depth-light text-slate-300 hover:text-white text-xs font-semibold transition-colors active:scale-98",
                            onclick: move |_| {
                                show_create_group.set(false);
                                group_name_input.set(String::new());
                                group_desc_input.set(String::new());
                                selected_group_contacts.set(vec![]);
                            },
                            "Cancel"
                        }
                        button {
                            class: "flex-grow py-2.5 rounded-xl bg-brand-blue hover:bg-brand-blue-hover text-white text-xs font-semibold transition-all active:scale-98 shadow-md shadow-brand-blue/20",
                            onclick: move |_| {
                                if !group_name_input().trim().is_empty() {
                                    let name = group_name_input().trim().to_string();
                                    let desc = group_desc_input().trim().to_string();
                                    let initials: String = name.chars().take(2).collect::<String>().to_uppercase();
                                    let id = format!("group_{}", name.to_lowercase().replace(' ', "_"));

                                    // Add group chat
                                    let mut list = chats();
                                    list.push(Chat {
                                        id: id.clone(),
                                        name: name.clone(),
                                        initials,
                                        last_message: format!("Group \"{}\" created.", name),
                                        timestamp: "Now".to_string(),
                                        unread: 0,
                                        is_group: true,
                                        is_online: true,
                                        is_verified: false,
                                        has_queued: false,
                                        messages: vec![
                                            Message {
                                                id: "g_init".to_string(),
                                                sender: "".to_string(),
                                                text: format!("🔒 Swarm group topic derived securely. Description: {}", desc),
                                                timestamp: "Just now".to_string(),
                                                is_outgoing: false,
                                                is_system: true,
                                                status: "read".to_string(),
                                                ..Default::default()
                                            }
                                        ]
                                    });
                                    chats.set(list);
                                    active_chat_id.set(Some(id));

                                    // Reset fields
                                    show_create_group.set(false);
                                    group_name_input.set(String::new());
                                    group_desc_input.set(String::new());
                                    selected_group_contacts.set(vec![]);
                                    mobile_show_chat.set(true);
                                }
                            },
                            "Create Group"
                        }
                    }
                }
            }
        }

        // Modal: Add Contact / Peer via Ticket
        if show_add_contact() {
            div {
                class: "absolute inset-0 z-40 flex items-center justify-center bg-black/70 backdrop-blur-sm",

                div {
                    class: "w-full max-w-md p-6 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl text-left",

                    h3 { class: "text-lg font-bold text-white mb-2", "Add Peer via Ticket" }
                    p { class: "text-slate-400 text-xs mb-5", "Paste a connection ticket or scan a QR code to resolve safety credentials." }

                    div {
                        class: "space-y-4 mb-6",

                        // Ticket input
                        div {
                            class: "flex flex-col space-y-1.5",
                            label { class: "text-xs text-slate-400 font-medium", "P2P Connection Ticket" }
                            input {
                                class: "bg-depth-black border border-depth-border rounded-xl px-3.5 py-2.5 text-xs text-slate-300 font-mono focus:border-brand-blue outline-none transition-colors duration-150",
                                placeholder: "iroh://ticket/peer_...",
                                value: "{contact_ticket_input}",
                                oninput: move |evt| contact_ticket_input.set(evt.value())
                            }
                        }

                        // Mock QR Code Scanner container
                        div {
                            class: "flex flex-col items-center justify-center border border-depth-border/60 rounded-xl p-4 bg-depth-black/40 relative overflow-hidden group",

                            div {
                                class: "w-32 h-32 border-2 border-brand-blue/30 rounded-xl flex items-center justify-center relative p-2 bg-depth-dark/80",

                                // Corners of scanner
                                div { class: "absolute top-0 left-0 w-4 h-4 border-t-2 border-l-2 border-brand-blue -mt-0.5 -ml-0.5 rounded-tl" }
                                div { class: "absolute top-0 right-0 w-4 h-4 border-t-2 border-r-2 border-brand-blue -mt-0.5 -mr-0.5 rounded-tr" }
                                div { class: "absolute bottom-0 left-0 w-4 h-4 border-b-2 border-l-2 border-brand-blue -mb-0.5 -ml-0.5 rounded-bl" }
                                div { class: "absolute bottom-0 right-0 w-4 h-4 border-b-2 border-r-2 border-brand-blue -mb-0.5 -mr-0.5 rounded-br" }

                                // Simulated laser sweep
                                div { class: "absolute inset-x-0 h-0.5 bg-brand-blue/60 top-1/2 -translate-y-1/2 animate-bounce" }

                                // Simulated QR pixels
                                div {
                                    class: "grid grid-cols-5 gap-1.5 opacity-30 w-full h-full",
                                    for _ in 0..25 {
                                        div { class: "bg-white rounded-[2px]" }
                                    }
                                }
                            }
                            span { class: "text-[10px] text-slate-500 mt-2 font-medium uppercase tracking-wider", "Simulator Scanner Active" }
                        }
                    }

                    // Actions buttons
                    div {
                        class: "flex space-x-3",
                        button {
                            class: "flex-grow py-2.5 rounded-xl border border-depth-border hover:bg-depth-light text-slate-300 hover:text-white text-xs font-semibold transition-colors active:scale-98",
                            onclick: move |_| {
                                show_add_contact.set(false);
                                contact_ticket_input.set(String::new());
                            },
                            "Cancel"
                        }
                        button {
                            class: "flex-grow py-2.5 rounded-xl bg-brand-blue hover:bg-brand-blue-hover text-white text-xs font-semibold transition-all active:scale-98 shadow-md shadow-brand-blue/20",
                            onclick: move |_| {
                                let ticket = contact_ticket_input();
                                if !ticket.trim().is_empty() {
                                    // Mock name generation
                                    let name = "Node Peer ".to_string() + &ticket.chars().take(4).collect::<String>();
                                    let initials = "NP".to_string();
                                    let id = format!("peer_{}", name.to_lowercase().replace(' ', "_"));

                                    // Insert mock contact
                                    let mut list = contacts();
                                    list.push(Contact {
                                        id: id.clone(),
                                        name: name.clone(),
                                        initials,
                                        node_id: ticket,
                                        is_online: true,
                                        is_verified: false,
                                    });
                                    contacts.set(list);

                                    // Insert mock conversation
                                    let mut list = chats();
                                    list.push(Chat {
                                        id: id.clone(),
                                        name,
                                        initials: "NP".to_string(),
                                        last_message: "Establish secured gossip route...".to_string(),
                                        timestamp: "Now".to_string(),
                                        unread: 0,
                                        is_group: false,
                                        is_online: true,
                                        is_verified: false,
                                        has_queued: false,
                                        messages: vec![
                                            Message {
                                                id: "c_init".to_string(),
                                                sender: "".to_string(),
                                                text: "🔒 Direct gossip swarming channel compiled. safety numbers derived.".to_string(),
                                                timestamp: "Just now".to_string(),
                                                is_outgoing: false,
                                                is_system: true,
                                                status: "read".to_string(),
                                                ..Default::default()
                                            }
                                        ]
                                    });
                                    chats.set(list);
                                    active_chat_id.set(Some(id));

                                    // Reset fields
                                    show_add_contact.set(false);
                                    contact_ticket_input.set(String::new());
                                    mobile_show_chat.set(true);
                                }
                            },
                            "Add Contact"
                        }
                    }
                }
            }
        }
    }
}
