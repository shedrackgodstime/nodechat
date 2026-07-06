use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn InfoDrawer() -> Element {
    let state = use_context::<AppState>();
    let mut chats = state.chats;
    let mut contacts = state.contacts;
    let mut active_chat_id = state.active_chat_id;
    let mut show_info_panel = state.show_info_panel;
    let mut mobile_show_chat = state.mobile_show_chat;
    let mut info_clear_confirm = state.info_clear_confirm;
    let mut info_remove_confirm = state.info_remove_confirm;

    let active_chat = active_chat_id().and_then(|id| chats().iter().find(|c| c.id == id).cloned());

    let Some(chat) = active_chat else {
        return rsx! {};
    };

    if !show_info_panel() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "w-full md:w-80 lg:w-96 flex-shrink-0 flex flex-col bg-depth-dark border-l border-depth-border absolute md:relative inset-0 md:inset-auto z-30 transition-all duration-300 animate-slide-in h-full overflow-y-auto overscroll-y-contain",

            // Header
            div {
                class: "h-16 flex items-center justify-between px-4 border-b border-depth-border bg-depth-dark/50",
                h3 { class: "font-semibold text-sm text-white", if chat.is_group { "Group Info" } else { "Contact Info" } }
                button {
                    class: "p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent transition-all",
                    onclick: move |_| show_info_panel.set(false),
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M6 18L18 6M6 6l12 12" }
                    }
                }
            }

            // Scroll details
            div {
                class: "p-4 space-y-6",

                // Summary Card
                div {
                    class: "p-4 rounded-2xl bg-depth-black border border-depth-border/60 flex flex-col items-center text-center",

                    // Large Avatar
                    div {
                        class: format!(
                            "w-16 h-16 rounded-3xl flex items-center justify-center text-white text-lg font-bold border-2 mb-3 {}",
                            if chat.is_group { "bg-teal-600/20 border-teal-500/30 text-teal-400" }
                            else if chat.is_verified { "bg-brand-blue/20 border-brand-blue/30 text-brand-blue" }
                            else { "bg-depth-light border-depth-border/50 text-slate-300" }
                        ),
                        "{chat.initials}"
                    }

                    h4 { class: "font-bold text-white text-base", "{chat.name}" }
                    p {
                        class: format!("text-xs font-semibold mt-1 {}", if chat.is_online { "text-emerald-400" } else { "text-slate-500" }),
                        if chat.is_group { "Gossip Swarm Active" } else if chat.is_online { "Online • Direct Tunnel" } else { "Offline" }
                    }
                }

                // Quick Actions Card (Verify toggle)
                if !chat.is_group {
                    div {
                        class: "p-4 rounded-2xl bg-depth-black border border-depth-border/60 space-y-3.5",
                        h5 { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider", "Safety Settings" }

                        div {
                            class: "flex items-center justify-between",
                            div {
                                h6 { class: "text-xs font-semibold text-white", "Trust Verification" }
                                p { class: "text-[10px] text-slate-400", "Approve keys & safety numbers" }
                            }

                            // Toggle slider
                            button {
                                class: format!(
                                    "w-11 h-6 rounded-full relative transition-colors duration-150 {}",
                                    if chat.is_verified { "bg-brand-blue" } else { "bg-depth-light border border-depth-border" }
                                ),
                                onclick: {
                                    let chat_id = chat.id.clone();
                                    move |_| {
                                        let mut list = chats();
                                        if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
                                            list[pos].is_verified = !list[pos].is_verified;
                                            chats.set(list);

                                            // Sync with contacts list
                                            let mut contact_list = contacts();
                                            if let Some(contact_pos) = contact_list.iter().position(|c| c.id == chat_id) {
                                                contact_list[contact_pos].is_verified = chats()[pos].is_verified;
                                                contacts.set(contact_list);
                                            }
                                        }
                                    }
                                },
                                div {
                                    class: format!(
                                        "w-5 h-5 rounded-full bg-white absolute top-0.5 transition-all duration-150 {}",
                                        if chat.is_verified { "left-[22px]" } else { "left-0.5" }
                                    )
                                }
                            }
                        }
                    }
                }

                // Protocol details card
                div {
                    class: "p-4 rounded-2xl bg-depth-black border border-depth-border/60 space-y-2.5",
                    h5 { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider", if chat.is_group { "GOSSIP SWARM ID" } else { "P2P ENDPOINT TICKET" } }
                    p { class: "text-[11px] font-mono text-slate-400 break-all select-all selection:bg-brand-blue/30 leading-relaxed", "iroh://ticket/identity_secp256k1_9f2a7b8823f81b312c12a884e" }
                }

                // Danger Zone Card
                div {
                    class: "p-4 rounded-2xl bg-depth-black border border-red-950/40 space-y-3.5",
                    h5 { class: "text-[10px] text-red-500 font-bold uppercase tracking-wider", "Danger Zone" }

                    // Action 1: Clear History (Requires confirmation click)
                    if !info_clear_confirm() {
                        button {
                            class: "w-full py-2.5 rounded-xl border border-depth-border hover:border-red-900/30 bg-depth-light/20 hover:bg-red-500/5 text-slate-300 hover:text-red-400 text-xs font-semibold transition-colors flex items-center justify-between px-3.5",
                            onclick: move |_| info_clear_confirm.set(true),
                            span { "Clear Message History" }
                            svg {
                                class: "w-4 h-4 text-slate-500 hover:text-red-400",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" }
                            }
                        }
                    } else {
                        div {
                            class: "p-3 rounded-xl bg-red-950/20 border border-red-900/30 space-y-2.5",
                            div {
                                h6 { class: "text-[11px] font-bold text-red-400 flex items-center space-x-1.5", "⚠️ Confirm Clearing Logs" }
                                p { class: "text-[9px] text-slate-400 mt-0.5 leading-relaxed", "All local messages in this chat will be deleted permanently." }
                            }
                            div {
                                class: "flex space-x-2 justify-end",
                                button {
                                    class: "px-2.5 py-1 rounded bg-depth-black hover:bg-depth-light border border-depth-border text-[9px] text-slate-400 hover:text-white transition-colors",
                                    onclick: move |_| info_clear_confirm.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "px-2.5 py-1 rounded bg-red-600 hover:bg-red-700 text-[9px] text-white font-bold transition-all shadow-md shadow-red-900/30",
                                    onclick: {
                                        let chat_id = chat.id.clone();
                                        move |_| {
                                            let mut list = chats();
                                            if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
                                                list[pos].messages.clear();
                                                list[pos].last_message = "Chat cleared.".to_string();
                                                chats.set(list);
                                                info_clear_confirm.set(false);
                                                show_info_panel.set(false);
                                            }
                                        }
                                    },
                                    "Clear"
                                }
                            }
                        }
                    }

                    // Action 2: Remove Contact (Requires confirmation click)
                    if !info_remove_confirm() {
                        button {
                            class: "w-full py-2.5 rounded-xl border border-depth-border hover:border-red-900/50 bg-depth-light/20 hover:bg-red-500/10 text-slate-300 hover:text-red-400 text-xs font-semibold transition-colors flex items-center justify-between px-3.5",
                            onclick: move |_| info_remove_confirm.set(true),
                            span { if chat.is_group { "Leave Swarm Group" } else { "Remove Contact Peer" } }
                            svg {
                                class: "w-4.5 h-4.5 text-slate-500 hover:text-red-400",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.5 8.25l-7.5 7.5-7.5-7.5" }
                            }
                        }
                    } else {
                        div {
                            class: "p-3 rounded-xl bg-red-950/20 border border-red-900/30 space-y-2.5",
                            div {
                                h6 { class: "text-[11px] font-bold text-red-400 flex items-center space-x-1.5", if chat.is_group { "⚠️ Confirm Leaving Group" } else { "⚠️ Confirm Removing Contact" } }
                                p { class: "text-[9px] text-slate-400 mt-0.5 leading-relaxed", if chat.is_group { "You will leave this gossip swarm. You need a new invite to rejoin." } else { "Remove this peer contact from your local list." } }
                            }
                            div {
                                class: "flex space-x-2 justify-end",
                                button {
                                    class: "px-2.5 py-1 rounded bg-depth-black hover:bg-depth-light border border-depth-border text-[9px] text-slate-400 hover:text-white transition-colors",
                                    onclick: move |_| info_remove_confirm.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "px-2.5 py-1 rounded bg-red-600 hover:bg-red-700 text-[9px] text-white font-bold transition-all shadow-md shadow-red-900/30",
                                    onclick: {
                                        let chat_id = chat.id.clone();
                                        move |_| {
                                            let mut list = chats();
                                            if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
                                                list.remove(pos);
                                                chats.set(list);
                                                active_chat_id.set(None);
                                                info_remove_confirm.set(false);
                                                show_info_panel.set(false);
                                                mobile_show_chat.set(false);
                                            }
                                        }
                                    },
                                    if chat.is_group { "Leave" } else { "Remove" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
