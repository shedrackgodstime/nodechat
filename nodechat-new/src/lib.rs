pub mod contract;
pub mod bridge;
pub mod mock_backend;
pub mod ui;

slint::include_modules!();

pub use bridge::{MockRuntime, UiBridge};
pub use contract::*;
pub use mock_backend::MockBackend;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    if let Err(e) = slint::android::init(app) {
        eprintln!("failed to initialize Android backend: {e}");
        return;
    }
    if let Err(e) = ui::run_app() {
        eprintln!("NodeChatNew failed to start on Android: {e}");
    }
}
