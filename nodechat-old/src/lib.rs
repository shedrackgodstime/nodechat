pub mod core;
pub mod crypto;
pub mod p2p;
pub mod storage;
pub mod ui;
pub mod api;
pub mod api_types;

use anyhow::Result;
use tokio::sync::{mpsc, broadcast};

use crate::core::commands::{Command, AppEvent};
use crate::core::NodeChatWorker;
use crate::storage::Database;
use crate::ui::NodeChatApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("PANIC: {}", panic_info);
    }));

    let internal_path = app.internal_data_path().unwrap_or_else(|| std::path::PathBuf::from("/data/user/0/com.nodechat/files"));
    let db_path = internal_path.join("nodechat_local.sqlite");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _enter = rt.enter();

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    if let Err(e) = run_app(options, db_path.to_string_lossy().to_string()) {
        log::error!("App error: {}", e);
    }
}

pub fn run_app(options: eframe::NativeOptions, db_path: String) -> Result<()> {
    // 1. Establish the Actor Channels
    let (tx_cmd, rx_cmd) = mpsc::channel::<Command>(100);
    let (tx_event, tx_event_unused) = broadcast::channel::<AppEvent>(100);
    // Explicitly drop the receiver we don't need for the broadcast chain
    drop(tx_event_unused);

    // Ensure the database directory exists (vital for Android internal storage)
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // 2. Check for Existing Identity (Non-blocking local check for UI flow)
    let has_identity = Database::has_identity(&db_path).unwrap_or(false);

    // 3. Initialize & Spawn the Backend Worker (Now handles its own DB and Network boot)
    let worker = NodeChatWorker::new(db_path, rx_cmd, tx_event.clone());
    tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            eprintln!("[Worker Error] {}", e);
        }
    });

    // 5. Launch the eframe UI
    eframe::run_native(
        "NodeChat",
        options,
        Box::new(move |cc| {
            Ok(Box::new(NodeChatApp::new(
                cc,
                tx_cmd,
                tx_event.subscribe(),
                has_identity,
            )))
        }),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}
