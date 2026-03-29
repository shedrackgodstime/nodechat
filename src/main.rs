use anyhow::Result;
use eframe::egui;
use tokio::sync::{mpsc, broadcast};

use NodeChat::core::commands::{Command, AppEvent};
use NodeChat::core::NodeChatWorker;
use NodeChat::storage::Database;
use NodeChat::ui::NodeChatUI;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Bootstrapping NodeChat Phase 1 Architecture...");

    // 1. Establish the Actor Channels (cross-thread communication)
    let (tx_cmd, rx_cmd) = mpsc::channel::<Command>(100);
    let (tx_event, rx_event) = broadcast::channel::<AppEvent>(100);

    // 2. Initialize the Embedded SQLite Database
    let db_path = "nodechat_local.sqlite"; // Creates a permanent local file 
    let db = Database::new(db_path)?;

    // 3. Initialize & Spawn the Tokio Backend Worker
    let worker = NodeChatWorker::new(db, rx_cmd, tx_event.clone());
    
    // Start worker in a distinct asynchronous task layout
    tokio::spawn(async move {
        worker.run().await;
    });

    // 4. Fire up the Synchronous Graphical Frontend (egui)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("NodeChat - Secure Identity Framework"),
        ..Default::default()
    };

    let ui = NodeChatUI::new(tx_cmd, rx_event);
    eframe::run_native(
        "NodeChat",
        options,
        Box::new(|_cc| Ok(Box::new(ui))),
    ).unwrap();

    Ok(())
}
