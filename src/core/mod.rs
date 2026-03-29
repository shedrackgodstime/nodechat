pub mod commands;

use commands::{Command, AppEvent};
use tokio::sync::{mpsc, broadcast};
use std::time::Duration;
use crate::storage::Database;

pub struct NodeChatWorker {
    db: Database,
    rx_commands: mpsc::Receiver<Command>,
    tx_events: broadcast::Sender<AppEvent>,
}

impl NodeChatWorker {
    pub fn new(
        db: Database,
        rx_commands: mpsc::Receiver<Command>,
        tx_events: broadcast::Sender<AppEvent>,
    ) -> Self {
        Self {
            db,
            rx_commands,
            tx_events,
        }
    }

    /// Primary asynchronous routine executing networking, crypto, and DB commands.
    pub async fn run(mut self) {
        println!("[Worker] Secure Backend Initializing...");
        
        // Let the UI know the backend is running and ready.
        let _ = self.tx_events.send(AppEvent::BackendReady);

        loop {
            tokio::select! {
                // 1. Process Frontend Commands
                cmd = self.rx_commands.recv() => {
                    match cmd {
                        Some(Command::Quit) => {
                            println!("[Worker] Graceful shutdown initiated.");
                            break;
                        }
                        Some(other) => {
                            println!("[Worker] Received UI command: {:?}", other);
                            // Process actions: encrypt, write to db, route to network
                        }
                        None => {
                            println!("[Worker] Command channel closed. Shutting down.");
                            break;
                        }
                    }
                }

                // 2. Scheduled maintenance (e.g., flushing offline message queue)
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    // query database for 'queued' messages and attempt delivery
                }
            }
        }
    }
}
