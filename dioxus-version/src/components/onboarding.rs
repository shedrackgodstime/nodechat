use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn Onboarding() -> Element {
    let state = use_context::<AppState>();
    let mut onboarding_step = state.onboarding_step;
    let mut temp_name_input = state.temp_name_input;

    rsx! {
        div {
            class: "absolute inset-0 z-50 flex flex-col items-center justify-center bg-depth-black select-none",

            if onboarding_step() == 0 {
                // Onboarding welcome pane
                div {
                    class: "w-full max-w-md p-8 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl text-center flex flex-col items-center animate-fade-in",

                    // Glowing brand logo container
                    div {
                        class: "w-24 h-24 rounded-3xl bg-brand-blue/10 border border-brand-blue/30 flex items-center justify-center mb-8 text-brand-blue shadow-lg shadow-brand-blue/5 animate-pulse",
                        svg {
                            class: "w-12 h-12",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path { d: "M12 2C6.477 2 2 6.477 2 12c0 1.821.487 3.53 1.338 5.008L2.054 22l5.122-1.312C8.61 21.416 10.246 22 12 22c5.523 0 10-4.477 10-10S17.523 2 12 2zm1 14h-2v-2h2v2zm0-4h-2V7h2v5z" }
                        }
                    }

                    h2 { class: "text-2.5xl font-extrabold tracking-tight text-white mb-2", "Secure P2P Messenger" }
                    p { class: "text-slate-400 text-sm max-w-sm mb-10 leading-relaxed", "Serverless, local-first messaging powered by secure cryptographic identities." }

                    button {
                        class: "w-full py-3.5 bg-brand-blue hover:bg-brand-blue-hover text-white font-bold rounded-xl transition-all shadow-md shadow-brand-blue/20 active:scale-98 tracking-wide",
                        onclick: move |_| onboarding_step.set(1),
                        "Get Started"
                    }

                    div { class: "text-[10px] text-slate-500 mt-6 uppercase tracking-wider font-mono", "Local-first by design" }
                }
            } else if onboarding_step() == 1 {
                // Display name form pane
                div {
                    class: "w-full max-w-md p-8 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl text-left flex flex-col animate-fade-in",

                    // Back navigation button
                    button {
                        class: "w-9 h-9 rounded-xl bg-depth-light border border-depth-border flex items-center justify-center text-brand-blue mb-6 hover:text-white transition-colors active:scale-95",
                        onclick: move |_| onboarding_step.set(0),
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.0",
                            view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 19.5L8.25 12l7.5-7.5" }
                        }
                    }

                    h2 { class: "text-2xl font-bold tracking-tight text-white mb-2", "What's your name?" }
                    p { class: "text-slate-400 text-xs mb-8 leading-relaxed", "This is your display name. It stays on your device and is only shared with peers you choose to trust." }

                    div {
                        class: "flex flex-col space-y-2 mb-8",
                        label { class: "text-[10px] text-slate-500 font-bold uppercase tracking-wider", "Display Name" }
                        div {
                            class: "relative flex items-center bg-depth-black border border-depth-border focus-within:border-brand-blue/70 transition-colors duration-150 rounded-xl px-4 py-3 text-sm",
                            input {
                                class: "bg-transparent text-white border-none w-full outline-none placeholder-slate-600 text-sm font-semibold",
                                placeholder: "e.g. Satoshi Nakamoto",
                                value: "{temp_name_input}",
                                oninput: move |evt| temp_name_input.set(evt.value())
                            }
                        }
                    }

                    button {
                        class: format!(
                            "w-full py-3.5 rounded-xl font-bold transition-all shadow-md active:scale-98 tracking-wide {}",
                            if temp_name_input().trim().is_empty() {
                                "bg-depth-light border border-depth-border/50 text-slate-500 cursor-not-allowed shadow-none"
                            } else {
                                "bg-brand-blue hover:bg-brand-blue-hover text-white shadow-brand-blue/20"
                            }
                        ),
                        disabled: temp_name_input().trim().is_empty(),
                        onclick: move |_| onboarding_step.set(2),
                        "Create Identity & Start"
                    }
                }
            } else {
                // Keypair generation loading screen
                div {
                    class: "w-full max-w-md p-8 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl text-center flex flex-col items-center animate-fade-in",

                    // Spinning gear/radar loader
                    div {
                        class: "w-20 h-20 rounded-full border-2 border-brand-blue/20 border-t-brand-blue flex items-center justify-center mb-8 animate-spin-slow",
                        svg {
                            class: "w-8 h-8 text-brand-blue animate-pulse",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" }
                        }
                    }

                    h3 { class: "text-lg font-bold text-white mb-2", "Generating Identity..." }
                    p { class: "text-slate-400 text-xs max-w-xs leading-relaxed", "Deriving secure cryptographic Ed25519 and X25519 keypairs for decentralized E2EE communication." }
                }
            }
        }
    }
}
