//! Slint UI wiring — callback registration and AppEvent listener.
//!
//! `wire_callbacks` is called once at startup from `lib.rs::run_app`.
//! All `on_*` callbacks are registered here and nowhere else (RULES.md U-02).
//! UI updates from the backend always come via `slint::invoke_from_event_loop` (RULES.md A-05).

pub mod models;

use slint::VecModel;
use slint::ComponentHandle;
use tokio::sync::{broadcast, mpsc};

use crate::core::commands::{AppEvent, Command};
use crate::AppWindow;

fn active_conversation_id(app: &AppWindow) -> String {
    app.get_active_conversation().id.to_string()
}

fn active_conversation_kind(app: &AppWindow) -> String {
    app.get_active_conversation().kind.to_string()
}

fn clear_active_conversation(app: &AppWindow) {
    let mut convo = app.get_active_conversation();
    convo.kind = "direct".into();
    convo.id = String::new().into();
    convo.title = String::new().into();
    convo.initials = String::new().into();
    convo.ticket = String::new().into();
    convo.is_online = false;
    convo.is_verified = false;
    convo.connection_stage = String::new().into();
    convo.member_count = "0".into();
    convo.return_screen = 0;
    app.set_active_conversation(convo);
    app.set_active_conversation_messages(VecModel::from_slice(&[]));
}

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
    wire_unlock(app, tx.clone());
    wire_send_message(app, tx.clone());
    wire_send_group_message(app, tx.clone());
    wire_add_contact(app, tx.clone());
    wire_accept_group_invite(app, tx.clone());
    wire_mark_verified(app, tx.clone());
    wire_clear_conversation(app, tx.clone());
    wire_remove_contact(app, tx.clone());
    wire_toggle_group_member(app, tx.clone());
    wire_copy_node_id(app, tx.clone());
    wire_delete_conversation(app, tx.clone());
    wire_retry_queued_messages(app, tx.clone());
    wire_launch_group(app, tx.clone());
    wire_load_conversation(app, tx.clone());
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

/// Fired when a returning user taps Unlock on the placeholder gate.
/// Rust: forwards a simple unlock command to the backend worker.
fn wire_unlock(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_unlock(move || {
        let _ = tx.try_send(Command::UnlockApp);
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

fn wire_delete_conversation(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_delete_conversation(move |target, is_group| {
        if let Some(app) = handle.upgrade() {
            let target = target.trim().to_owned();
            if target.is_empty() {
                return;
            }
            let should_clear =
                active_conversation_id(&app) == target
                && (active_conversation_kind(&app) == "group") == is_group;
            let _ = tx.try_send(Command::DeleteConversation {
                target,
                is_group,
            });
            if should_clear {
                clear_active_conversation(&app);
                app.set_current_screen(0);
            }
        }
    });
}

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

/// Fired when the user taps Mark as Verified on the key verification screen.
/// Rust: sends Command::MarkVerified to the backend worker.
fn wire_mark_verified(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_mark_verified(move || {
        if let Some(app) = handle.upgrade() {
            if active_conversation_kind(&app) != "direct" {
                return;
            }
            let node_id = active_conversation_id(&app);
            if node_id.is_empty() {
                return;
            }
            let _ = tx.try_send(Command::MarkVerified { node_id });
        }
    });
}

fn wire_clear_conversation(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_clear_conversation(move |target, is_group| {
        let target = target.trim().to_owned();
        if target.is_empty() {
            return;
        }
        let _ = tx.try_send(Command::ClearConversationHistory { target, is_group });
    });
}

fn wire_remove_contact(app: &AppWindow, tx: mpsc::Sender<Command>) {
    let handle = app.as_weak();
    app.on_remove_contact(move || {
        if let Some(app) = handle.upgrade() {
            if active_conversation_kind(&app) != "direct" {
                return;
            }
            let target = active_conversation_id(&app);
            if target.is_empty() {
                return;
            }
            let _ = tx.try_send(Command::DeleteConversation {
                target,
                is_group: false,
            });
        }
    });
}

fn wire_toggle_group_member(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_toggle_group_member(move |peer_id| {
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

/// Fired when the UI wants the active conversation thread loaded from SQLite.
fn wire_load_conversation(app: &AppWindow, tx: mpsc::Sender<Command>) {
    app.on_load_conversation(move |target, is_group| {
        let _ = tx.try_send(Command::LoadConversation {
            target: target.to_string(),
            is_group,
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
