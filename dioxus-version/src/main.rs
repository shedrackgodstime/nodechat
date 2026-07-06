use components::{
    chat_pane::ChatPane, dashboard::Dashboard, diagnostics_tab::DiagnosticsTab,
    info_drawer::InfoDrawer, lock_screen::LockScreen, modals::Modals, onboarding::Onboarding,
    settings_tab::SettingsTab, sidebar::Sidebar,
};
use dioxus::prelude::*;
use helpers::sleep_ms;
use types::AppState;

mod components;
mod helpers;
mod mock_data;
mod types;

// Asset constants
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // 1. Onboarding & Identity State
    let mut has_identity = use_signal(|| false);
    let mut onboarding_step = use_signal(|| 0); // 0: Welcome, 1: SetupName, 2: KeyGen Loader
    let mut display_name_input = use_signal(|| "P2P Explorer".to_string());
    let mut temp_name_input = use_signal(String::new);

    // Settings Draft & Feedback States
    let mut settings_name_draft = use_signal(String::new);
    let show_copied_toast = use_signal(|| false);
    let mut show_clear_confirm = use_signal(|| false);
    let mut show_reset_confirm = use_signal(|| false);
    let mut info_clear_confirm = use_signal(|| false);
    let mut info_remove_confirm = use_signal(|| false);

    // 2. Lock & Authentication State
    let mut is_locked = use_signal(|| false);
    let pin_input = use_signal(String::new);
    let pin_error = use_signal(|| false);

    // 3. Tab and Selection State
    let current_tab = use_signal(|| "chats".to_string()); // "chats", "contacts", "settings", "diagnostics"
    let previous_tab = use_signal(|| "chats".to_string()); // sidebar shows this when on settings/diagnostics
    let active_chat_id = use_signal(|| Some("alice".to_string()));
    let search_query = use_signal(String::new);

    // 4. Modals and Drawers
    let show_create_group = use_signal(|| false);
    let show_add_contact = use_signal(|| false);
    let show_info_panel = use_signal(|| false); // Contact details slide drawer

    // 5. Diagnostics logs
    let mut logs = use_signal(mock_data::get_mock_logs);

    // 6. Input fields
    let message_input = use_signal(String::new);
    let contact_ticket_input = use_signal(String::new);
    let group_name_input = use_signal(String::new);
    let group_desc_input = use_signal(String::new);
    let join_ticket_input = use_signal(String::new);

    // 7. Selected contacts for group creation
    let selected_group_contacts = use_signal(Vec::new);

    // 8. Responsive helper
    let mobile_show_chat = use_signal(|| false);

    // Mock dataset
    let chats = use_signal(mock_data::get_mock_chats);
    let contacts = use_signal(mock_data::get_mock_contacts);

    // LocalStorage Loading future on startup
    use_future(move || async move {
        let mut name_found = false;
        let mut saved_name = String::new();
        let eval = document::eval("return localStorage.getItem('nodechat_display_name') || ''");
        if let Ok(val) = eval.await {
            if let Some(s) = val.as_str() {
                if !s.trim().is_empty() {
                    saved_name = s.to_string();
                    name_found = true;
                }
            }
        }
        if name_found {
            display_name_input.set(saved_name);
            has_identity.set(true);
        }
    });

    // LocalStorage Saving effect when name is successfully set
    use_effect(move || {
        let name = display_name_input();
        if !name.trim().is_empty() && has_identity() {
            let js = format!(
                "localStorage.setItem('nodechat_display_name', '{}');",
                name.replace('\'', "\\'")
            );
            let _ = document::eval(&js);
        }
    });

    // Synchronize settings draft and confirmations on tab enter
    use_effect(move || {
        if current_tab() == "settings" {
            settings_name_draft.set(display_name_input());
            show_clear_confirm.set(false);
            show_reset_confirm.set(false);
        }
    });

    // Reset contact info drawer confirmations when it is closed
    use_effect(move || {
        if !show_info_panel() {
            info_clear_confirm.set(false);
            info_remove_confirm.set(false);
        }
    });

    // Async log generator to make the interface feel alive
    use_future(move || async move {
        let mut counter = 0;
        loop {
            sleep_ms(12000).await;
            counter += 1;
            let log_text = match counter % 4 {
                0 => format!(
                    "[{:02}:{:02}:{:02}] [P2P] Relayed connection ping sent to bootstrap.",
                    0,
                    15 + counter,
                    counter * 7 % 60
                ),
                1 => format!(
                    "[{:02}:{:02}:{:02}] [CRYPTO] Ephemeral session key ratcheted successfully.",
                    0,
                    15 + counter,
                    counter * 7 % 60
                ),
                2 => format!(
                    "[{:02}:{:02}:{:02}] [DATABASE] Auto-checkpoint: 12 frames written to WAL.",
                    0,
                    15 + counter,
                    counter * 7 % 60
                ),
                _ => format!(
                    "[{:02}:{:02}:{:02}] [P2P] Verified peer connection active.",
                    0,
                    15 + counter,
                    counter * 7 % 60
                ),
            };
            logs.write().push(log_text);
            if logs.read().len() > 100 {
                logs.write().remove(0);
            }
        }
    });

    // Simulated Crypto Keypair Generation Timer
    use_effect(move || {
        if onboarding_step() == 2 {
            spawn(async move {
                // Simulate cryptographic key compilation delays
                sleep_ms(2800).await;

                // Derive display name from input
                let final_name = if temp_name_input().trim().is_empty() {
                    "Satoshi Nakamoto".to_string()
                } else {
                    temp_name_input().trim().to_string()
                };
                display_name_input.set(final_name);

                // Transition and unlock
                has_identity.set(true);
                is_locked.set(false);
                onboarding_step.set(0);
                temp_name_input.set(String::new());
            });
        }
    });

    // Bundle shared application state into a unified AppState context provider
    let state = AppState {
        has_identity,
        onboarding_step,
        display_name_input,
        temp_name_input,
        settings_name_draft,
        show_copied_toast,
        show_clear_confirm,
        show_reset_confirm,
        info_clear_confirm,
        info_remove_confirm,
        is_locked,
        pin_input,
        pin_error,
        current_tab,
        previous_tab,
        active_chat_id,
        search_query,
        show_create_group,
        show_add_contact,
        show_info_panel,
        logs,
        message_input,
        contact_ticket_input,
        group_name_input,
        group_desc_input,
        join_ticket_input,
        selected_group_contacts,
        mobile_show_chat,
        chats,
        contacts,
    };
    use_context_provider(|| state);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700;800&family=Plus+Jakarta+Sans:wght@300;400;500;600;700;800&display=swap" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div {
            class: "flex h-full w-full overflow-hidden bg-depth-black text-slate-200 font-sans antialiased selection:bg-brand-blue/30 selection:text-white",

            if !has_identity() {
                Onboarding {}
            } else if is_locked() {
                LockScreen {}
            } else {
                // Sidebar List View
                Sidebar {}

                // Main Panel Area
                if active_chat_id().is_some() {
                    ChatPane {}
                    InfoDrawer {}
                } else {
                    if current_tab() == "settings" {
                        SettingsTab {}
                    } else if current_tab() == "chats" || current_tab() == "contacts" {
                        Dashboard {}
                    } else {
                        DiagnosticsTab {}
                    }
                }

                // Global Modals Overlay
                Modals {}
            }
        }
    }
}
