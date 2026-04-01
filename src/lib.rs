#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;

pub mod core;
pub mod crypto;
pub mod p2p;
pub mod storage;
pub mod ui;

slint::include_modules!();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    let db_dir = app.internal_data_path();
    slint::android::init(app).unwrap();
    run_app(db_dir).unwrap();
}

/// Boot the Tokio runtime, spawn the backend worker, and start the Slint event loop.
///
/// * `db_dir` — optional base directory for the database file. If None, uses local directory.
pub fn run_app(db_dir: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build Tokio runtime")?;

    let _guard = runtime.enter();

    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(100);
    let (tx_events, _rx_events)    = tokio::sync::broadcast::channel(64);

    let tx_events_bg = tx_events.clone();
    let db_path = if let Some(dir) = db_dir {
        dir.join("nodechat.db")
    } else {
        std::path::PathBuf::from("nodechat.db")
    };

    let db_path_bg = db_path.clone();
    runtime.spawn(async move {
        match core::NodeChatWorker::new(rx_commands, tx_events_bg, &db_path_bg).await {
            Ok(worker) => worker.run().await,
            Err(e)     => tracing::error!("failed to start backend worker: {:?}", e),
        }
    });

    let app = AppWindow::new().context("failed to create Slint window")?;

    // Pre-flight check: if the user already has an identity, bypass the overlay (RULES.md UX-01)
    let startup_db = storage::initialize(&db_path).context("failed to open db for startup check")?;
    if let Ok(Some(identity)) = storage::queries::get_local_identity(&startup_db) {
        app.set_has_identity(true);
        app.set_my_display_name(identity.display_name.into());
        app.set_my_node_id(hex::encode(identity.node_id_bytes).into());
    }

    ui::wire_callbacks(&app, tx_commands, tx_events);
    app.run()?;

    Ok(())
}
