use crate::helpers::create_chat;
use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();
    let mut join_ticket_input = state.join_ticket_input;
    let mut chats = state.chats;
    let mut active_chat_id = state.active_chat_id;
    let mut mobile_show_chat = state.mobile_show_chat;

    rsx! {
        div {
            class: "flex-grow flex flex-col items-center justify-center p-8 text-center bg-gradient-to-b from-depth-black to-depth-dark/20 h-full overflow-y-auto overscroll-y-contain",

            div {
                class: "max-w-lg space-y-8 flex flex-col items-center w-full",

                // Large Brand Logo Indicator
                div {
                    class: "w-28 h-28 rounded-3xl bg-depth-dark border-2 border-depth-border flex items-center justify-center text-brand-blue shadow-2xl shadow-brand-blue/5 animate-pulse",
                    svg {
                        class: "w-14 h-14",
                        fill: "currentColor",
                        view_box: "0 0 24 24",
                        path { d: "M12 2C6.477 2 2 6.477 2 12c0 1.821.487 3.53 1.338 5.008L2.054 22l5.122-1.312C8.61 21.416 10.246 22 12 22c5.523 0 10-4.477 10-10S17.523 2 12 2zm1 14h-2v-2h2v2zm0-4h-2V7h2v5z" }
                    }
                }

                div {
                    h2 { class: "text-3xl font-extrabold tracking-tight text-white mb-2", "NodeChat Decentralized Swarm" }
                    p { class: "text-slate-400 text-sm max-w-md mx-auto leading-relaxed", "Welcome to your local P2P node. Message history is fully encrypted in SQLite. Connect directly to peers over UDP without servers." }
                }

                // Peer metrics
                div {
                    class: "grid grid-cols-3 gap-4 w-full p-4 rounded-2xl bg-depth-dark border border-depth-border/70",

                    div {
                        h4 { class: "text-[10px] text-slate-500 uppercase tracking-wider mb-1", "DHT Peers" }
                        p { class: "text-base font-bold text-brand-blue", "3 Online" }
                    }
                    div {
                        h4 { class: "text-[10px] text-slate-500 uppercase tracking-wider mb-1", "Relay Mode" }
                        p { class: "text-base font-bold text-brand-blue", "Direct UDP" }
                    }
                    div {
                        h4 { class: "text-[10px] text-slate-500 uppercase tracking-wider mb-1", "Security Tag" }
                        p { class: "text-base font-bold text-brand-blue font-mono text-xs mt-1", "23f81b" }
                    }
                }

                // Quick join ticket form
                div {
                    class: "w-full space-y-2.5",
                    div {
                        class: "flex items-center bg-depth-black border border-depth-border rounded-xl px-4 py-2.5 text-sm",
                        input {
                            class: "bg-transparent text-white border-none w-full outline-none placeholder-slate-500 text-xs font-mono",
                            placeholder: "Paste peer ticket iroh://peer/... to start chat",
                            value: "{join_ticket_input}",
                            oninput: move |evt| join_ticket_input.set(evt.value())
                        }
                    }
                    button {
                        class: "w-full py-3 bg-brand-blue hover:bg-brand-blue-hover text-white rounded-xl text-xs font-semibold tracking-wider uppercase transition-all shadow-md shadow-brand-blue/20 active:scale-98",
                        onclick: move |_| {
                            let ticket = join_ticket_input();
                            if !ticket.trim().is_empty() {
                                let name = "Node Peer ".to_string() + &ticket.chars().take(4).collect::<String>();
                                let initials = "NP".to_string();
                                let id = format!("peer_{}", name.to_lowercase().replace(' ', "_"));

                                // Create mock chat
                                let mut list = chats();
                                list.push(create_chat(&id, &name, &initials, false, true, false));
                                chats.set(list);

                                active_chat_id.set(Some(id));
                                join_ticket_input.set(String::new());
                                mobile_show_chat.set(true);
                            }
                        },
                        "Connect to Peer Node"
                    }
                }
            }
        }
    }
}
