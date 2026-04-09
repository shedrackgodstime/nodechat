use anyhow::Context;
use slint::ComponentHandle;
use crate::AppWindow;

pub fn run_app() -> anyhow::Result<()> {
    let app = AppWindow::new().context("failed to create Slint window")?;

    app.on_open_url(|url| {
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(url.as_str()).spawn();
        
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(&["/C", "start", url.as_str().replace("&", "^&")]).spawn();
        
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(url.as_str()).spawn();
    });

    app.run().context("failed to run Slint window")?;
    Ok(())
}
