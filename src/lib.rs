pub mod contract;
pub mod storage;
pub mod p2p;
pub mod backend;
pub mod bridge;
pub mod crypto;
pub mod error;
pub mod ui;
pub mod ui_models;
pub mod notifications;

slint::include_modules!();

pub use bridge::{RealRuntime, UiBridge};
pub use contract::*;

pub static ANDROID_DATA_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    if let Some(path) = app.internal_data_path() {
        let _ = ANDROID_DATA_DIR.set(path);
    }

    if let Err(e) = slint::android::init(app) {
        eprintln!("failed to initialize Android backend: {e}");
        return;
    }
    if let Err(e) = ui::run_app() {
        eprintln!("NodeChatNew failed to start on Android: {e}");
    }
}
