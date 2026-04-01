#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
pub mod core;
pub mod ui;

slint::include_modules!();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    run_app().unwrap();
}

pub fn run_app() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to build Tokio runtime")?;

    let (_tx_commands, rx_commands) = tokio::sync::mpsc::channel(100);

    runtime.spawn(async move {
        let worker = core::NodeChatWorker::new(rx_commands).await;
        worker.run().await;
    });

    let app = AppWindow::new()
        .context("Failed to create Slint window")?;

    app.run()?;

    Ok(())
}
