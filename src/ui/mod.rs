//! Slint UI wiring — callback registration and AppEvent listener.
//!
//! `wire_callbacks` is called once at startup from `lib.rs::run_app`.
//! All `on_*` callbacks are registered here and nowhere else (RULES.md U-02).
//! UI updates from the backend always come via `slint::invoke_from_event_loop` (RULES.md A-05).

pub mod models;

use slint::{ComponentHandle, Model, VecModel};
use tokio::sync::{broadcast, mpsc};

use crate::core::commands::{AppEvent, Command};
use crate::AppWindow;

fn active_conversation_id(app: &AppWindow) -> String {
    app.get_active_conversation().id.to_string()
}

fn active_conversation_kind(app: &AppWindow) -> String {
    app.get_active_conversation().kind.to_string()
}


/// Wire all Slint callbacks to backend commands and start the AppEvent listener.
///
/// Called exactly once during startup (RULES.md U-02).
pub fn wire_callbacks(
    app: &AppWindow,
    tx: mpsc::Sender<Command>,
    rx_events: broadcast::Sender<AppEvent>,
) {
    wire_create_identity(app, tx.clone());
    wire_setup_complete(app, tx.clone());
    wire_unlock(app, tx.clone());
    wire_confirm_name_change(app, tx.clone());
    wire_send_message(app, tx.clone());
    wire_send_group_message(app, tx.clone());
    wire_add_contact(app, tx.clone());
    wire_accept_group_invite(app, tx.clone());
    wire_toggle_verified(app, tx.clone());
    wire_toggle_group_member(app, tx.clone());
    wire_copy_node_id(app, tx.clone());
    wire_retry_queued_messages(app, tx.clone());
    wire_launch_group(app, tx.clone());
    wire_load_conversation(app, tx.clone());
    wire_destructive_actions(app, tx.clone());
    wire_change_password(app, tx.clone());
    wire_clear_debug_logs(app);
    wire_copy_debug_logs(app);
    wire_copy_text(app);

    spawn_event_listener(app.as_weak(), rx_events);
}

// ── Onboarding ───────────────────────────────────────────────────────────────

/// Fired when the user taps "Generate Private Vault" on the setup screen.
/// Rust: generates identity, then calls app.set_setup_step(2), app.set_my_display_name(), etc.
fn wire_create_identity(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_create_identity(move |name: slint::SharedString, pin: slint::SharedString| {
        if let Some(ui) = handle.upgrade() {
            ui.set_setup_step(2); // Show "Generating Identity..." immediately
        }
        let _ = tx.try_send(Command::CreateIdentity {
            name: name.to_string(),
            pin: pin.to_string(),
        });
    });
}

fn wire_confirm_name_change(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_confirm_name_change(move |name: slint::SharedString| {
        let _ = tx.try_send(Command::UpdateDisplayName {
            name: name.to_string(),
        });
    });
}

/// Fired when the user taps "Launch Chats" on the identity card.
/// Rust: persists the has-identity flag, then sets app.set_has_identity(true).
fn wire_setup_complete(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_setup_complete(move || {
        let _ = tx.try_send(Command::FinaliseIdentity);
    });
}

/// Fired when a returning user taps Unlock on the placeholder gate.
/// Rust: forwards a simple unlock command to the backend worker.
fn wire_unlock(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_unlock(move |pin: slint::SharedString| {
        let _ = tx.try_send(Command::UnlockApp { pin: pin.to_string() });
    });
}

// ── Chat ──────────────────────────────────────────────────────────────────────

/// Fired when the user presses Send in a 1:1 chat.
/// Rust: sends Command::SendDirectMessage to the backend worker.
fn wire_send_message(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_send_message(move |text| {
        if let Some(app) = handle.upgrade() {
            let text = text.trim().to_owned();
            if text.is_empty() {
                return;
            }
            let target = active_conversation_id(&app);
            if active_conversation_kind(&app) != "direct" {
                return;
            }
            if target.is_empty() {
                return;
            }
            let _ = tx.try_send(Command::SendDirectMessage {
                target,
                plaintext: text,
            });
        }
    });
}

// Handlers unified in wire_destructive_actions

/// Fired when the user taps Retry Now in a direct chat.
fn wire_retry_queued_messages(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_retry_queued(move || {
        if let Some(app) = handle.upgrade() {
            if active_conversation_kind(&app) != "direct" {
                return;
            }
            let target = active_conversation_id(&app);
            if target.is_empty() {
                return;
            }
            let _ = tx.try_send(Command::RetryQueuedMessages { target });
        }
    });
}

/// Fired when the user presses Send in a group chat.
/// Rust: sends Command::SendGroupMessage to the backend worker.
fn wire_send_group_message(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_send_group_message(move |text| {
        if let Some(app) = handle.upgrade() {
            let text = text.trim().to_owned();
            if text.is_empty() {
                return;
            }
            if active_conversation_kind(&app) != "group" {
                return;
            }
            let topic = active_conversation_id(&app);
            if topic.is_empty() {
                return;
            }
            let _ = tx.try_send(Command::SendGroupMessage {
                topic,
                plaintext: text,
            });
        }
    });
}

/// Fired when the user confirms adding a new contact.
/// Rust: sends Command::SendDirectMessage with a handshake payload (to be defined).
fn wire_add_contact(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_add_contact(move |ticket_or_id| {
        let _ = tx.try_send(Command::AddContact {
            ticket_or_id: ticket_or_id.to_string(),
        });
    });
}

fn wire_accept_group_invite(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_accept_group_invite(move |topic, group_name, symmetric_key| {
        let _ = tx.try_send(Command::AcceptGroupInvite {
            topic: topic.to_string(),
            group_name: group_name.to_string(),
            symmetric_key: symmetric_key.to_string(),
        });
    });
}

fn wire_clear_debug_logs(app: &AppWindow) {
    let handle = app.as_weak();
    app.on_clear_debug_logs(move || {
        if let Some(app) = handle.upgrade() {
            app.set_debug_logs(VecModel::from_slice(&[]));
        }
    });
}

fn wire_copy_debug_logs(app: &AppWindow) {
    let handle = app.as_weak();
    app.on_copy_debug_logs(move || {
        if let Some(app) = handle.upgrade() {
            let mut formatted = String::new();
            let logs = app.get_debug_logs();
            for i in 0..logs.row_count() {
                if let Some(entry) = logs.row_data(i) {
                    formatted.push_str(&format!(
                        "[{}] {} [{}] {}\n",
                        entry.timestamp, entry.level, entry.target, entry.message
                    ));
                }
            }
            app.set_clipboard_buffer(formatted.into());
            app.invoke_do_copy();
            tracing::info!("debug logs copied to clipboard ({} entries)", logs.row_count());
        }
    });
}

fn wire_copy_text(app: &AppWindow) {
    let handle = app.as_weak();
    app.on_copy_text(move |text| {
        if let Some(app) = handle.upgrade() {
            app.set_clipboard_buffer(text);
            app.invoke_do_copy();
        }
    });
}

/// Fired when the user toggles trust in contact details.
/// Rust: sends Command::ToggleVerified to the backend worker.
fn wire_toggle_verified(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_toggle_verified(move |node_id, verified| {
        let _ = tx.try_send(Command::ToggleVerified { 
            node_id: node_id.to_string(), 
            verified 
        });
    });
}

// Handlers moved to wire_destructive_actions for unified PIN security

// Handlers moved to wire_destructive_actions for unified PIN security

fn wire_toggle_group_member(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_toggle_group_member(move |peer_id: slint::SharedString| {
        let peer_id = peer_id.trim().to_owned();
        if peer_id.is_empty() {
            return;
        }
        let _ = tx.try_send(Command::ToggleGroupMemberSelection { peer_id });
    });
}

/// Fired when the user taps Copy Node ID.
/// Rust: copies the local node ID to the system clipboard.
fn wire_copy_node_id(app: &AppWindow, _tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_copy_node_id(move || {
        if let Some(app) = handle.upgrade() {
            let ticket = app.get_my_endpoint_ticket().to_string();
            let node_id = app.get_my_node_id().to_string();
            // Note: Actual clipboard copy is handled natively in Slint (settings.slint / identity_card.slint)
            // for maximum cross-platform (Desktop + Android) compatibility.
            tracing::info!(
                share_ticket = %ticket,
                node_id = %node_id,
                "identity share copy event logged from UI"
            );
        }
    });
}

/// Fired when the user taps destructive actions across the app.
fn wire_destructive_actions(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let tx_delete = tx.clone();
    app.on_delete_conversation(move |target: slint::SharedString, is_group: bool, pin: slint::SharedString| {
        let _ = tx_delete.try_send(Command::DeleteConversation {
            target: target.to_string(),
            is_group,
            pin: pin.to_string(),
        });
    });

    let tx_clear = tx.clone();
    app.on_clear_conversation(move |target: slint::SharedString, is_group: bool, pin: slint::SharedString| {
        let _ = tx_clear.try_send(Command::ClearConversationHistory {
            target: target.to_string(),
            is_group,
            pin: pin.to_string(),
        });
    });

    let tx_delete_id = tx.clone();
    app.on_delete_identity(move |pin: slint::SharedString| {
        let _ = tx_delete_id.try_send(Command::DeleteIdentity { pin: pin.to_string() });
    });

    let tx_clear_messages = tx.clone();
    app.on_clear_messages(move |pin: slint::SharedString| {
        let _ = tx_clear_messages.try_send(Command::ClearMessages { pin: pin.to_string() });
    });
}

/// Fired when the user confirms creating a group.
/// Rust: sends Command::CreateGroup to the backend worker.
fn wire_launch_group(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_launch_group(move |name| {
        let _ = tx.try_send(Command::CreateGroup {
            name: name.to_string(),
        });
    });
}

/// Fired when the UI wants the active conversation thread loaded from SQLite.
fn wire_load_conversation(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_load_conversation(move |target, is_group| {
        let _ = tx.try_send(Command::LoadConversation {
            target: target.to_string(),
            is_group,
        });
    });
}

/// Fired when the user submits the Change Password form.
/// Rust: verifies current PIN, hashes new PIN, updates DB.
fn wire_change_password(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_change_password(move |current_pin: slint::SharedString, new_pin: slint::SharedString| {
        let _ = tx.try_send(Command::ChangePassword {
            current_pin: current_pin.to_string(),
            new_pin: new_pin.to_string(),
        });
    });
}

// ── AppEvent Listener ─────────────────────────────────────────────────────────

/// Spawns a Tokio task that receives `AppEvent`s from the backend and pushes
/// each one to the Slint UI via `invoke_from_event_loop` (RULES.md A-05).
fn spawn_event_listener(handle: slint::Weak<AppWindow>, tx_events: broadcast::Sender<AppEvent>) {
    let mut rx = tx_events.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let h = handle.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = h.upgrade() {
                    models::apply_event(&ui, event);
                }
            });
        }
    });
}
