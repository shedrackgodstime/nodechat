//! Slint UI wiring — callback registration and AppEvent listener.
//!
//! `wire_callbacks` is called once at startup from `lib.rs::run_app`.
//! All `on_*` callbacks are registered here and nowhere else (RULES.md U-02).
//! UI updates from the backend always come via `slint::invoke_from_event_loop` (RULES.md A-05).

pub mod models;

use slint::ComponentHandle;
use tokio::sync::{broadcast, mpsc};

use crate::core::commands::{AppEvent, Command};
use crate::AppWindow;

/// Wire all Slint callbacks to backend commands and start the AppEvent listener.
///
/// Called exactly once during startup (RULES.md U-02).
pub fn wire_callbacks(
    app: &AppWindow,
    tx: mpsc::Sender<Command>,
    rx_events: broadcast::Sender<AppEvent>,
) {
    wire_confirm_name(app, tx.clone());
    wire_setup_complete(app, tx.clone());
    wire_send_message(app, tx.clone());
    wire_send_group_message(app, tx.clone());
    wire_add_contact(app, tx.clone());
    wire_mark_verified(app, tx.clone());
    wire_copy_node_id(app, tx.clone());
    wire_launch_group(app, tx.clone());
    wire_settings_actions(app, tx.clone());

    spawn_event_listener(app.as_weak(), rx_events);
}

// ── Onboarding ───────────────────────────────────────────────────────────────

/// Fired when the user taps "Generate Private Vault" on the setup screen.
/// Rust: generates identity, then calls app.set_setup_step(2), app.set_my_display_name(), etc.
fn wire_confirm_name(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_confirm_name(move |name| {
        let _ = tx.try_send(Command::CreateIdentity {
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

// ── Chat ──────────────────────────────────────────────────────────────────────

/// Fired when the user presses Send in a 1:1 chat.
/// Rust: sends Command::SendDirectMessage to the backend worker.
fn wire_send_message(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let active_peer = app.get_active_peer_node_id().to_string();
    app.on_send_message(move |text| {
        let _ = tx.try_send(Command::SendDirectMessage {
            target:    active_peer.clone(),
            plaintext: text.to_string(),
        });
    });
}

/// Fired when the user presses Send in a group chat.
/// Rust: sends Command::SendGroupMessage to the backend worker.
fn wire_send_group_message(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let active_group = app.get_active_group_name().to_string(); // WIRE: use TopicId not name
    app.on_send_group_message(move |text| {
        let _ = tx.try_send(Command::SendGroupMessage {
            topic:     active_group.clone(),
            plaintext: text.to_string(),
        });
    });
}

/// Fired when the user confirms adding a new contact.
/// Rust: sends Command::SendDirectMessage with a handshake payload (to be defined).
fn wire_add_contact(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_add_contact(move |node_id, _name| {
        // WIRE: register peer in DB, initiate X25519 handshake
        let _ = tx.try_send(Command::SendDirectMessage {
            target:    node_id.to_string(),
            plaintext: String::from("{\"type\":\"handshake\"}"),
        });
    });
}

/// Fired when the user taps Mark as Verified on the key verification screen.
/// Rust: sends Command::MarkVerified to the backend worker.
fn wire_mark_verified(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let peer_id = app.get_active_peer_node_id().to_string();
    app.on_mark_verified(move || {
        let _ = tx.try_send(Command::MarkVerified {
            node_id: peer_id.clone(),
        });
    });
}

/// Fired when the user taps Copy Node ID.
/// Rust: copies the local node ID to the system clipboard.
fn wire_copy_node_id(app: &AppWindow, _tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_copy_node_id(move || {
        if let Some(app) = handle.upgrade() {
            let node_id = app.get_my_node_id().to_string();
            // Note: Manual clipboard set is disabled here due to API mismatch in this Slint version.
            // However, the Node ID is now a ReadOnly TextInput, so users can highlight and copy natively.
            tracing::info!("node id requested for copy: {}", node_id);
        }
    });
}

/// Fired when the user taps actions in the Settings screen.
fn wire_settings_actions(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let tx_clear = tx.clone();
    app.on_clear_messages(move || {
        let _ = tx_clear.try_send(Command::ClearMessages);
    });

    let tx_delete = tx.clone();
    app.on_delete_identity(move || {
        let _ = tx_delete.try_send(Command::DeleteIdentity);
    });

    // Note: show-node-id-qr and edit-display-name can be wired here too later
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
