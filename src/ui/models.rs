//! Data bridge between `AppEvent` backend types and Slint UI models/properties.
//!
//! `apply_event` is the single point where backend data becomes UI state (RULES.md U-05).
//! Only called from inside `slint::invoke_from_event_loop` closures — never directly.

use crate::core::commands::AppEvent;
use crate::AppWindow;

/// Translate a backend `AppEvent` into Slint property/model updates.
///
/// This function runs on the Slint event thread. Keep it short — no I/O, no blocking (RULES.md U-04).
pub fn apply_event(ui: &AppWindow, event: AppEvent) {
    match event {
        AppEvent::IncomingMessage { sender, id: _, plaintext: _, timestamp: _ } => {
            // WIRE: push message into the active chat ListView model
            tracing::debug!(peer = %sender, "incoming direct message — UI model update pending");
        }

        AppEvent::IncomingGroupMessage { topic, sender, id: _, plaintext: _, timestamp: _ } => {
            // WIRE: push message into the group chat ListView model
            tracing::debug!(topic = %topic, peer = %sender, "incoming group message — UI model update pending");
        }

        AppEvent::IncomingFile { sender, file_name, path: _ } => {
            // WIRE: show file received notification in chat
            tracing::debug!(peer = %sender, file = %file_name, "incoming file — UI update pending");
        }

        AppEvent::MessageStatusUpdate { id, status } => {
            // WIRE: find bubble in model by id and update its status indicator
            tracing::debug!(msg = %id, status = %status.as_str(), "status update — UI model update pending");
        }

        AppEvent::GroupInviteReceived { topic, group_name } => {
            // WIRE: show group invite notification / dialog
            tracing::debug!(topic = %topic, group = %group_name, "group invite — UI update pending");
        }

        AppEvent::PeerOnlineStatus { peer, online, via_relay } => {
            // WIRE: update the status dot and connection mode label
            tracing::debug!(peer = %peer, online, via_relay, "peer status — UI update pending");
            let _ = ui; // suppress unused warning until wired
        }

        AppEvent::SetupComplete => {
            ui.set_has_identity(true);
            tracing::debug!("setup complete — removing overlay");
        }

        AppEvent::IdentityGenerated { display_name, node_id } => {
            ui.set_my_display_name(display_name.into());
            ui.set_my_node_id(node_id.into());
            ui.set_setup_step(2); 
        }

        AppEvent::MessagesCleared => {
            // WIRE: clear the active message list model and notify user
            tracing::info!("messages cleared — UI refresh pending");
        }

        AppEvent::Error { message } => {
            // WIRE: surface message in a toast / banner component
            tracing::warn!("backend error surfaced to UI: {}", message);
        }
    }
}