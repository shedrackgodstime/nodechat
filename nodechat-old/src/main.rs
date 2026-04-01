use anyhow::Result;
use eframe::egui;
use nodechat::run_app;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("nodechat=info".parse().unwrap()))
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _enter = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    let db_path = "nodechat_local.sqlite".to_string();

    run_app(options, db_path)
}
