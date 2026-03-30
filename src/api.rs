//! Core API for NodeChat - designed to work with any UI framework (egui, Dioxus, etc.)
//!
//! ## Usage with Dioxus
//!
//! ```ignore
//! use nodechat::api::{BackendHandles, Command, AppEvent};
//!
//! // Initialize backend
//! let handles = nodechat::api::init_backend(db_path).await?;
//!
//! // Spawn the background worker (done automatically by init_backend)
//!
//! // Subscribe to events
//! let mut rx = handles.subscribe();
//! while let Ok(event) = rx.recv().await {
//!     match event {
//!         AppEvent::IncomingMessage { sender, plaintext } => { ... }
//!         AppEvent::IdentityCreated { node_id, ticket, .. } => { ... }
//!         // ...
//!     }
//! }
//!
//! // Send commands
//! handles.send_command(Command::SendDirectMessage { 
//!     target: "...", 
//!     plaintext: "...".to_string() 
//! }).await?;
//! ```

pub use crate::core::commands::{Command, AppEvent};
pub use crate::api_types::{ChatMessage, Peer, Identity};

use crate::core::NodeChatWorker;
use tokio::sync::{mpsc, broadcast};

/// Initialize the NodeChat backend
/// 
/// Returns handles for communicating with the background worker
pub async fn init_backend(
    db_path: String,
) -> Result<BackendHandles, Box<dyn std::error::Error + Send + Sync>> {
    let (tx_cmd, rx_cmd) = mpsc::channel::<Command>(100);
    let (tx_event, _) = broadcast::channel::<AppEvent>(100);
    
    // Ensure the database directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let worker = NodeChatWorker::new(db_path, rx_cmd, tx_event.clone());
    tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            eprintln!("[Worker Error] {}", e);
        }
    });
    
    Ok(BackendHandles {
        command_sender: tx_cmd,
        event_receiver: tx_event,
    })
}

/// Handles for communicating with the NodeChat backend
pub struct BackendHandles {
    pub command_sender: mpsc::Sender<Command>,
    pub event_receiver: broadcast::Sender<AppEvent>,
}

impl BackendHandles {
    /// Subscribe to backend events (for Dioxus reactivity)
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.event_receiver.subscribe()
    }
    
    /// Send a command to the backend
    pub async fn send_command(&self, cmd: Command) -> Result<(), mpsc::error::SendError<Command>> {
        self.command_sender.send(cmd).await
    }
}
