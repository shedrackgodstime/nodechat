use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn DiagnosticsTab() -> Element {
    let state = use_context::<AppState>();
    let mut current_tab = state.current_tab;
    let mut logs = state.logs;

    rsx! {
        div {
            class: "flex-grow flex flex-col bg-depth-black h-full",

            // Top status banner
            div {
                class: "h-16 flex items-center justify-between px-6 border-b border-depth-border bg-depth-dark/80 backdrop-blur-md",

                div {
                    class: "flex items-center space-x-3",
                    // Back button to Settings
                    button {
                        class: "p-2 rounded-xl text-slate-400 hover:text-white hover:bg-depth-light border border-transparent transition-all",
                        onclick: move |_| {
                            current_tab.set("settings".to_string());
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
                    div {
                        h2 { class: "font-semibold text-sm text-white", "Diagnostics Console" }
                        p { class: "text-[10px] text-slate-500", "Realtime cryptographic and network gossip telemetry feed" }
                    }
                }

                button {
                    class: "px-3 py-1.5 rounded-xl border border-depth-border bg-depth-light/20 text-xs text-slate-300 hover:text-white flex items-center space-x-1.5 transition-colors",
                    onclick: move |_| {
                        logs.write().clear();
                        logs.write().push("[00:00:00] [SYSTEM] Diagnostics logs cleared manually.".to_string());
                    },
                    svg {
                        class: "w-3.5 h-3.5",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.8",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M19.5 12c0-1.232-.046-2.453-.138-3.662a4.006 4.006 0 00-3.7-3.7 48.656 48.656 0 00-7.324 0 4.006 4.006 0 00-3.7 3.7C4.796 9.48 4.75 10.7 4.75 12s.046 2.453.138 3.662a4.006 4.006 0 003.7 3.7 48.656 48.656 0 007.324 0 4.006 4.006 0 003.7-3.7c.092-1.209.138-2.43.138-3.662zM9 10.5l6 6m0-6l-6 6"
                        }
                    }
                    span { "Clear Logs" }
                }
            }

            // Scrollable terminal
            div {
                class: "flex-grow p-6 overflow-y-auto overscroll-y-contain font-mono text-[11px] text-emerald-400 bg-depth-black/95 select-text selection:bg-emerald-500/20 selection:text-emerald-300 space-y-1.5 scrollbar-thin scrollbar-thumb-depth-border",

                for log_line in logs().iter() {
                    div {
                        class: "leading-relaxed hover:bg-depth-light/20 px-2 py-0.5 rounded transition-all",
                        "{log_line}"
                    }
                }
            }
        }
    }
}
