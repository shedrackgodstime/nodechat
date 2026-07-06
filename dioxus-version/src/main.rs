mod bridge;
mod components;
mod contract;
mod helpers;
mod mock_data;
mod types;

use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Onboarding & Identity
    let has_identity = use_signal(|| false);
    let onboarding_step = use_signal(|| 0);
    let display_name_input = use_signal(|| "P2P Explorer".to_string());
    let temp_name_input = use_signal(String::new);

    // Settings
    let mut settings_name_draft = use_signal(String::new);
    let show_copied_toast = use_signal(|| false);
    let mut show_clear_confirm = use_signal(|| false);
    let mut show_reset_confirm = use_signal(|| false);
    let mut info_clear_confirm = use_signal(|| false);
    let mut info_remove_confirm = use_signal(|| false);

    // Lock
    let is_locked = use_signal(|| false);
    let pin_input = use_signal(String::new);
    let pin_error = use_signal(|| false);

    // Tabs & Selection
    let current_tab = use_signal(|| "chats".to_string());
    let previous_tab = use_signal(|| "chats".to_string());
    let active_chat_id = use_signal(|| Some("alice".to_string()));
    let search_query = use_signal(String::new);

    // Modals
    let show_create_group = use_signal(|| false);
    let show_add_contact = use_signal(|| false);
    let show_info_panel = use_signal(|| false);

    // Logs
    let mut logs = use_signal(mock_data::get_mock_logs);

    // Inputs
    let message_input = use_signal(String::new);
    let contact_ticket_input = use_signal(String::new);
    let group_name_input = use_signal(String::new);
    let group_desc_input = use_signal(String::new);
    let join_ticket_input = use_signal(String::new);
    let selected_group_contacts = use_signal(Vec::new);

    // Responsive
    let mobile_show_chat = use_signal(|| false);

    // Data
    let chats = use_signal(mock_data::get_mock_chats);
    let contacts = use_signal(mock_data::get_mock_contacts);

    // Backend bridge
    use_future(move || async move {
        let mut bridge = bridge::spawn::spawn_backend();
        logs.write().push("[SYSTEM] Backend started.".into());
        while let Some(_event) = bridge.recv().await {
            // TODO: wire events to signals
        }
    });

    // Sync settings draft on tab change
    use_effect(move || {
        if current_tab() == "settings" {
            settings_name_draft.set(display_name_input());
            show_clear_confirm.set(false);
            show_reset_confirm.set(false);
        }
    });

    // Reset info drawer confirmations
    use_effect(move || {
        if !show_info_panel() {
            info_clear_confirm.set(false);
            info_remove_confirm.set(false);
        }
    });

    let state = types::AppState {
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
                components::onboarding::Onboarding {}
            } else if is_locked() {
                components::lock_screen::LockScreen {}
            } else {
                components::sidebar::Sidebar {}
                if active_chat_id().is_some() {
                    components::chat_pane::ChatPane {}
                    components::info_drawer::InfoDrawer {}
                } else {
                    if current_tab() == "settings" {
                        components::settings_tab::SettingsTab {}
                    } else if current_tab() == "chats" || current_tab() == "contacts" {
                        components::dashboard::Dashboard {}
                    } else {
                        components::diagnostics_tab::DiagnosticsTab {}
                    }
                }
                components::modals::Modals {}
            }
        }
    }
}
