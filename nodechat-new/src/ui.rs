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

    let handle_name = app.as_weak();
    app.on_confirm_name_change(move |name| {
        if let Some(ui) = handle_name.upgrade() {
            ui.set_display_name(name);
        }
    });

    let handle_pass = app.as_weak();
    app.on_submit_password(move |_cur, new_pass| {
        if let Some(ui) = handle_pass.upgrade() {
            ui.set_has_password(!new_pass.is_empty());
        }
    });


    app.run().context("failed to run Slint window")?;
    Ok(())
}
