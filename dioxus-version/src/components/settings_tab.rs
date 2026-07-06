use crate::helpers::sleep_ms;
use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsTab() -> Element {
    let state = use_context::<AppState>();
    let mut display_name_input = state.display_name_input;
    let mut settings_name_draft = state.settings_name_draft;
    let mut show_copied_toast = state.show_copied_toast;
    let mut show_clear_confirm = state.show_clear_confirm;
    let mut show_reset_confirm = state.show_reset_confirm;
    let mut is_locked = state.is_locked;
    let mut current_tab = state.current_tab;
    let mut mobile_show_chat = state.mobile_show_chat;
    let mut chats = state.chats;
    let mut has_identity = state.has_identity;
    let mut onboarding_step = state.onboarding_step;
    let mut pin_input = state.pin_input;

    let settings_initials = if display_name_input().trim().is_empty() {
        "UN".to_string()
    } else {
        display_name_input()
            .trim()
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase()
    };

    rsx! {
        div {
            class: "flex-grow flex flex-col bg-depth-black p-8 overflow-y-auto overscroll-y-contain space-y-6 md:max-w-3xl md:mx-auto md:w-full select-none",

            // Header
            div {
                class: "flex items-center justify-between pb-2 border-b border-depth-border/30",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight text-white", "Settings" }
                    p { class: "text-slate-400 text-xs mt-0.5", "Manage your local identity and app security" }
                }

                // Back button for Mobile
                button {
                    class: "md:hidden p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent transition-all",
                    onclick: move |_| mobile_show_chat.set(false),
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
            }

            // 1. Profile Summary Card
            div {
                class: "p-5 rounded-2xl bg-depth-dark border border-depth-border flex items-center space-x-4 shadow-lg shadow-black/10 animate-fade-in",

                // Avatar
                div {
                    class: "w-16 h-16 rounded-3xl bg-brand-blue/20 border border-brand-blue/30 text-brand-blue flex items-center justify-center font-bold text-xl shadow-inner",
                    "{settings_initials}"
                }

                div {
                    h3 { class: "font-bold text-white text-base", "{display_name_input}" }
                    p { class: "text-xs text-emerald-400 font-semibold flex items-center space-x-1.5 mt-0.5", "Local identity active & configured" }
                }
            }

            // 2. Identity & Security Card Group
            div {
                class: "flex flex-col space-y-2",
                span { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider px-2", "Identity & Security" }

                div {
                    class: "rounded-2xl bg-depth-dark border border-depth-border overflow-hidden divide-y divide-depth-border/40",

                    // Row 1: Edit Display Name
                    div {
                        class: "flex flex-col",
                        div {
                            class: "p-4 flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-2 sm:space-y-0",
                            div {
                                h4 { class: "text-xs font-semibold text-white", "Display Name" }
                                p { class: "text-[10px] text-slate-400", "How others see you in chat swarms" }
                            }
                            input {
                                class: "bg-depth-black border border-depth-border/70 rounded-xl px-3 py-1.5 text-xs text-white focus:border-brand-blue outline-none transition-colors max-w-xs sm:w-64 font-semibold",
                                value: "{settings_name_draft}",
                                oninput: move |evt| settings_name_draft.set(evt.value())
                            }
                        }

                        // Sliding action panel for unsaved changes
                        if settings_name_draft() != display_name_input() {
                            div {
                                class: "px-4 py-2.5 bg-brand-blue/5 border-t border-depth-border/40 flex items-center justify-between transition-all duration-300 animate-fade-in",
                                span { class: "text-[10px] text-brand-blue font-semibold", "Unsaved changes detected" }
                                div {
                                    class: "flex space-x-2",
                                    button {
                                        class: "px-3 py-1 rounded-lg bg-depth-black hover:bg-depth-light border border-depth-border text-[10px] text-slate-400 hover:text-white transition-colors",
                                        onclick: move |_| {
                                            settings_name_draft.set(display_name_input());
                                        },
                                        "Cancel"
                                    }
                                    button {
                                        class: "px-3 py-1 rounded-lg bg-brand-blue hover:bg-brand-blue-hover text-[10px] text-white font-semibold transition-all shadow-md shadow-brand-blue/10",
                                        onclick: move |_| {
                                            let name = settings_name_draft().trim().to_string();
                                            if !name.is_empty() {
                                                display_name_input.set(name);
                                            }
                                        },
                                        "Save Changes"
                                    }
                                }
                            }
                        }
                    }

                    // Row 2: Connection Ticket
                    div {
                        class: "p-4 flex items-center justify-between cursor-pointer hover:bg-depth-light/20 transition-all",
                        onclick: move |_| {
                            show_copied_toast.set(true);
                            spawn(async move {
                                sleep_ms(2000).await;
                                show_copied_toast.set(false);
                            });
                        },
                        div {
                            class: "min-w-0 flex-grow pr-4",
                            h4 { class: "text-xs font-semibold text-white", "My Connection Ticket" }
                            p { class: "text-[10px] text-slate-500 truncate font-mono mt-0.5", "iroh://ticket/identity_secp256k1_9f2a7b8823f81b312c12a884e" }
                        }
                        button {
                            class: "px-3 py-1.5 rounded-xl bg-depth-black hover:bg-depth-light border border-depth-border text-[10px] text-slate-400 hover:text-white transition-colors flex-shrink-0 flex items-center space-x-1.5",
                            title: "Copy Ticket",
                            if show_copied_toast() {
                                span { class: "text-[9px] text-emerald-400 font-bold tracking-wide animate-pulse", "Copied ✓" }
                            } else {
                                svg {
                                    class: "w-4.5 h-4.5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "1.8",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H5.25m10.5 9.375a1.125 1.125 0 001.125-1.125v-8.25a1.125 1.125 0 00-1.125-1.125H8.25A1.125 1.125 0 007.125 7.875v8.25a1.125 1.125 0 001.125 1.125h7.5z"
                                    }
                                }
                            }
                        }
                    }

                    // Row 3: Security PIN Lock settings
                    div {
                        class: "p-4 flex items-center justify-between",
                        div {
                            h4 { class: "text-xs font-semibold text-white", "Vault PIN Protection" }
                            p { class: "text-[10px] text-slate-400", "Enable vault passcode locks" }
                        }
                        button {
                            class: "px-3 py-1.5 rounded-xl border border-depth-border bg-depth-black hover:bg-depth-light text-slate-300 hover:text-white text-xs font-semibold transition-colors",
                            onclick: move |_| {
                                is_locked.set(true);
                            },
                            "Activate Vault PIN Lock"
                        }
                    }
                }
            }

            // 3. Info & Diagnostics Card Group
            div {
                class: "flex flex-col space-y-2",
                span { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider px-2", "Info & Diagnostics" }

                div {
                    class: "rounded-2xl bg-depth-dark border border-depth-border overflow-hidden",

                    // Row 1: Diagnostics console shortcut
                    div {
                        class: "p-4 flex items-center justify-between cursor-pointer hover:bg-depth-light/20 transition-all",
                        onclick: move |_| {
                            current_tab.set("diagnostics".to_string());
                        },
                        div {
                            h4 { class: "text-xs font-semibold text-white", "Open Diagnostics Telemetry Console" }
                            p { class: "text-[10px] text-slate-400", "Observe peer and cryptographic event logs" }
                        }
                        span { class: "text-slate-500 text-sm font-medium pr-1", "›" }
                    }
                }
            }

            // 4. Danger Zone Card Group (Safe double-confirmation flow)
            div {
                class: "flex flex-col space-y-2",
                span { class: "text-[10px] text-red-500 font-bold uppercase tracking-wider px-2", "Danger Zone" }

                div {
                    class: "rounded-2xl bg-depth-dark border border-red-950/40 overflow-hidden divide-y divide-red-950/20",

                    // Row 1: Clear History
                    if !show_clear_confirm() {
                        button {
                            class: "w-full p-4 flex items-center justify-between text-left hover:bg-red-500/5 transition-all text-slate-300 hover:text-red-400",
                            onclick: move |_| show_clear_confirm.set(true),
                            div {
                                h4 { class: "text-xs font-semibold", "Clear Message History" }
                                p { class: "text-[10px] text-slate-400 mt-0.5", "Delete all message logs from this device" }
                            }
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
                            class: "p-4 bg-red-950/20 border border-red-900/40 flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-2.5 sm:space-y-0",
                            div {
                                h4 { class: "text-xs font-bold text-red-400 flex items-center space-x-1.5", "⚠️ Confirm Clearing Logs" }
                                p { class: "text-[10px] text-slate-400 mt-0.5", "All local messages in chats will be deleted permanently." }
                            }
                            div {
                                class: "flex space-x-2 self-end sm:self-center",
                                button {
                                    class: "px-3 py-1.5 rounded-lg bg-depth-black hover:bg-depth-light border border-depth-border text-[10px] text-slate-400 hover:text-white transition-colors",
                                    onclick: move |_| show_clear_confirm.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "px-3 py-1.5 rounded-lg bg-red-600 hover:bg-red-700 text-[10px] text-white font-bold transition-all shadow-md shadow-red-900/30",
                                    onclick: move |_| {
                                        let mut list = chats();
                                        for chat in list.iter_mut() {
                                            chat.messages.clear();
                                            chat.last_message = "Chat cleared.".to_string();
                                        }
                                        chats.set(list);
                                        show_clear_confirm.set(false);
                                    },
                                    "Clear Logs"
                                }
                            }
                        }
                    }

                    // Row 2: Reset Identity
                    if !show_reset_confirm() {
                        button {
                            class: "w-full p-4 flex items-center justify-between text-left hover:bg-red-500/10 transition-all text-slate-300 hover:text-red-400",
                            onclick: move |_| show_reset_confirm.set(true),
                            div {
                                h4 { class: "text-xs font-semibold", "Reset Identity" }
                                p { class: "text-[10px] text-slate-400 mt-0.5", "Permanently delete your profile and cryptographic keys" }
                            }
                            svg {
                                class: "w-4 h-4 text-slate-500 hover:text-red-400",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" }
                            }
                        }
                    } else {
                        div {
                            class: "p-4 bg-red-950/30 border border-red-900 flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-2.5 sm:space-y-0",
                            div {
                                h4 { class: "text-xs font-bold text-red-500 flex items-center space-x-1.5", "⚠️ DESTRUCTIVE ACTION" }
                                p { class: "text-[10px] text-slate-300 mt-0.5", "Factory reset will erase your cryptographic vault completely." }
                            }
                            div {
                                class: "flex space-x-2 self-end sm:self-center",
                                button {
                                    class: "px-3 py-1.5 rounded-lg bg-depth-black hover:bg-depth-light border border-depth-border text-[10px] text-slate-400 hover:text-white transition-colors",
                                    onclick: move |_| show_reset_confirm.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "px-3 py-1.5 rounded-lg bg-red-600 hover:bg-red-700 text-[10px] text-white font-bold transition-all shadow-md shadow-red-900/30",
                                    onclick: move |_| {
                                        has_identity.set(false);
                                        onboarding_step.set(0);
                                        is_locked.set(false);
                                        pin_input.set(String::new());
                                        show_reset_confirm.set(false);
                                    },
                                    "Reset Vault"
                                }
                            }
                        }
                    }
                }
            }

            // 5. Version Info Footer
            div {
                class: "text-center pt-4 border-t border-depth-border/30",
                span { class: "text-[10px] text-slate-500 font-mono tracking-wider", "NodeChat v0.1.0-alpha • Local-first P2P" }
            }
        }
    }
}
