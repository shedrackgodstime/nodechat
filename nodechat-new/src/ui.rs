use anyhow::Context;
use slint::ComponentHandle;
use crate::AppWindow;

pub fn run_app() -> anyhow::Result<()> {
    let app = AppWindow::new().context("failed to create Slint window")?;

    app.on_open_url(|url| {
        let _ = open::that(url.as_str());
    });

    let handle = app.as_weak();
    app.on_copy_to_clipboard(move |text| {
        if let Some(ui) = handle.upgrade() {
            ui.set_clipboard_buffer(text);
            ui.invoke_do_copy();
        }
    });

    app.run().context("failed to run Slint window")?;
    Ok(())
}
