use anyhow::Context;
use slint::ComponentHandle;
use crate::AppWindow;

pub fn run_app() -> anyhow::Result<()> {
    let app = AppWindow::new().context("failed to create Slint window")?;

    app.run().context("failed to run Slint window")?;
    Ok(())
}
