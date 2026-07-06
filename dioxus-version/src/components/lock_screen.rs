use crate::types::AppState;
use dioxus::prelude::*;

#[component]
pub fn LockScreen() -> Element {
    let state = use_context::<AppState>();
    let mut is_locked = state.is_locked;
    let mut pin_input = state.pin_input;
    let mut pin_error = state.pin_error;

    // Automatically focus the lock screen overlay to enable keyboard inputs on mount
    use_effect(move || {
        spawn(async move {
            // Small delay to ensure render finishes
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            document::eval(
                r#"
                var el = document.getElementById('pin-lock-screen');
                if (el) el.focus();
            "#,
            );
        });
    });

    // Handle physical keyboard inputs
    let onkeydown = move |evt: KeyboardEvent| {
        let mut s = pin_input();
        match evt.key() {
            Key::Character(ref c)
                if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() && s.len() < 4 =>
            {
                s.push_str(c);
                pin_input.set(s.clone());
                pin_error.set(false);
                if s.len() == 4 {
                    if s == "1234" {
                        is_locked.set(false);
                        pin_input.set(String::new());
                    } else {
                        pin_error.set(true);
                        pin_input.set(String::new());
                    }
                }
            }
            Key::Backspace if !s.is_empty() => {
                s.pop();
                pin_input.set(s);
                pin_error.set(false);
            }
            Key::Escape => {
                // Bypass shortcut on Escape
                is_locked.set(false);
                pin_input.set(String::new());
                pin_error.set(false);
            }
            _ => {}
        }
    };

    rsx! {
        div {
            id: "pin-lock-screen",
            tabindex: "0",
            class: "absolute inset-0 z-50 flex flex-col items-center justify-center bg-depth-black/95 backdrop-blur-md transition-all duration-300 outline-none",
            onkeydown,

            div {
                class: "w-full max-w-sm p-8 rounded-3xl bg-depth-dark border border-depth-border shadow-2xl shadow-brand-blue/5 text-center flex flex-col items-center",

                // Shield Lock Icon
                div {
                    class: "w-20 h-20 rounded-2xl bg-brand-blue/10 border border-brand-blue/20 flex items-center justify-center mb-6 text-brand-blue animate-pulse",
                    svg {
                        class: "w-10 h-10",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "1.5",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z"
                        }
                    }
                }

                h2 { class: "text-2xl font-semibold tracking-tight text-white mb-2", "Identity Vault Locked" }
                p { class: "text-slate-400 text-sm mb-8", "Enter your security PIN to unlock local messages and database keys." }

                // Input Dot indicators
                div {
                    class: "flex space-x-3 mb-8 justify-center",
                    for i in 0..4 {
                        div {
                            class: format!(
                                "w-4 h-4 rounded-full border transition-all duration-150 {}",
                                if pin_input().len() > i { "bg-brand-blue border-brand-blue scale-110 shadow-lg shadow-brand-blue/50" }
                                else if pin_error() { "border-red-500 bg-red-500/20" }
                                else { "border-depth-border bg-depth-light" }
                            )
                        }
                    }
                }

                // Keypad grid
                div {
                    class: "grid grid-cols-3 gap-4 w-full max-w-[280px] mb-6",
                    for num in ["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
                        button {
                            class: "h-14 rounded-2xl bg-depth-light hover:bg-depth-light/80 border border-depth-border/30 hover:border-depth-border/70 text-white font-medium text-lg flex items-center justify-center transition-all active:scale-95 duration-100",
                            onclick: move |_| {
                                if pin_input().len() < 4 {
                                    pin_input.write().push_str(num);
                                    pin_error.set(false);
                                    if pin_input().len() == 4 {
                                        if pin_input() == "1234" {
                                            is_locked.set(false);
                                            pin_input.set(String::new());
                                        } else {
                                            pin_error.set(true);
                                            pin_input.set(String::new());
                                        }
                                    }
                                }
                            },
                            "{num}"
                        }
                    }
                    // Backspace
                    button {
                        class: "h-14 rounded-2xl bg-depth-light hover:bg-depth-light/80 border border-depth-border/30 hover:border-depth-border/70 text-slate-400 font-medium text-lg flex items-center justify-center transition-all active:scale-95 duration-100",
                        onclick: move |_| {
                            let mut s = pin_input();
                            if !s.is_empty() {
                                s.pop();
                                pin_input.set(s);
                                pin_error.set(false);
                            }
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M12 9.75L14.25 12m0 0l2.25 2.25M14.25 12l2.25-2.25M14.25 12L12 14.25m-2.58 4.92l-6.375-6.375a1.125 1.125 0 010-1.59L9.42 4.83c.211-.211.498-.33.796-.33H19.5a2.25 2.25 0 012.25 2.25v10.5a2.25 2.25 0 01-2.25 2.25h-9.284c-.298 0-.585-.119-.796-.33z"
                            }
                        }
                    }
                    // Zero
                    button {
                        class: "h-14 rounded-2xl bg-depth-light hover:bg-depth-light/80 border border-depth-border/30 hover:border-depth-border/70 text-white font-medium text-lg flex items-center justify-center transition-all active:scale-95 duration-100",
                        onclick: move |_| {
                            if pin_input().len() < 4 {
                                pin_input.write().push('0');
                                pin_error.set(false);
                                if pin_input().len() == 4 {
                                    if pin_input() == "1234" {
                                        is_locked.set(false);
                                        pin_input.set(String::new());
                                    } else {
                                        pin_error.set(true);
                                        pin_input.set(String::new());
                                    }
                                }
                            }
                        },
                        "0"
                    }
                    // Fast unlock shortcut
                    button {
                        class: "h-14 rounded-2xl bg-brand-blue/10 hover:bg-brand-blue/20 border border-brand-blue/30 text-brand-blue text-xs font-bold uppercase tracking-wider flex items-center justify-center transition-all active:scale-95 duration-100",
                        onclick: move |_| {
                            is_locked.set(false);
                            pin_input.set(String::new());
                            pin_error.set(false);
                        },
                        "Bypass"
                    }
                }

                if pin_error() {
                    div { class: "text-red-500 text-xs font-medium animate-bounce mt-2", "Incorrect PIN code. Try '1234' or Bypass." }
                } else {
                    div { class: "text-slate-500 text-xs", "Hint: Default PIN is '1234' or click Bypass (Supports typing)" }
                }
            }
        }
    }
}
