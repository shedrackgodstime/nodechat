#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use crate::core::commands::Command;
pub mod core;
pub mod crypto;
pub mod p2p;
pub mod storage;
pub mod ui;

slint::include_modules!();

#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static ANDROID_COMMAND_TX: OnceLock<tokio::sync::mpsc::Sender<Command>> = OnceLock::new();
#[cfg(target_os = "android")]
static ANDROID_FOREGROUND: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    let db_dir = app.internal_data_path();
    if let Err(e) = slint::android::init_with_event_listener(app, |event| {
        use slint::android::android_activity::{MainEvent, PollEvent};

        if let PollEvent::Main(main) = event {
            let foreground = matches!(main, MainEvent::GainedFocus | MainEvent::Resume { .. });
            let background = matches!(main, MainEvent::LostFocus | MainEvent::Pause | MainEvent::Stop | MainEvent::Destroy);
            if foreground || background {
                let is_foreground = foreground;
                ANDROID_FOREGROUND.store(is_foreground, std::sync::atomic::Ordering::Relaxed);
                if let Some(tx) = ANDROID_COMMAND_TX.get() {
                    let _ = tx.try_send(Command::SetAppForeground { foreground: is_foreground });
                }
            }
        }
    }) {
        eprintln!("failed to initialize Android backend: {e}");
        return;
    }

    if let Err(e) = run_app(db_dir) {
        eprintln!("NodeChat failed to start on Android: {e}");
    }
}

/// Boot the Tokio runtime, spawn the backend worker, and start the Slint UI.
///
/// * `db_dir` — optional base directory for the database file. If None, uses local directory.
pub fn run_app(db_dir: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    init_tracing();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build Tokio runtime")?;

    let _guard = runtime.enter();

    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(100);
    let (tx_events, _rx_events)    = tokio::sync::broadcast::channel(64);
    install_panic_hook(tx_events.clone());

    #[cfg(target_os = "android")]
    {
        let _ = ANDROID_COMMAND_TX.set(tx_commands.clone());
    }

    let tx_events_bg = tx_events.clone();
    let db_path = db_dir
        .map(|dir| dir.join("nodechat.db"))
        .unwrap_or_else(|| std::path::PathBuf::from("nodechat.db"));

    let db_path_bg = db_path.clone();
    runtime.spawn(async move {
        match core::NodeChatWorker::new(rx_commands, tx_events_bg, &db_path_bg).await {
            Ok(worker) => worker.run().await,
            Err(e)     => tracing::error!("failed to start backend worker: {:?}", e),
        }
    });

    let app = AppWindow::new().context("failed to create Slint window")?;

    // Pre-flight check: if the user already has an identity, bypass the overlay (RULES.md UX-01).
    // Keep this connection short-lived so the backend worker owns the live DB cleanly.
    {
        let startup_db =
            storage::initialize(&db_path).context("failed to open db for startup check")?;
        let startup_chats = storage::queries::list_chat_previews(&startup_db)
            .context("failed to load chat list at startup")?;
        let chat_rows: Vec<ChatPreview> = startup_chats
            .into_iter()
            .map(|chat| ChatPreview {
                id: chat.id.into(),
                name: chat.name.into(),
                initials: chat.initials.into(),
                last_message: chat.last_message.into(),
                timestamp: chat.timestamp.into(),
                unread: chat.unread,
                is_group: chat.is_group,
                is_online: chat.is_online,
                is_relay: chat.is_relay,
                is_queued: chat.is_queued,
                is_verified: chat.is_verified,
            })
            .collect();
        app.set_chats(slint::VecModel::from_slice(&chat_rows));

        if let Ok(Some(identity)) = storage::queries::get_local_identity(&startup_db) {
            app.set_has_identity(true);
            app.set_is_locked(true);
            app.set_setup_step(0);
            app.set_current_screen(0);
            app.set_my_display_name(identity.display_name.into());
            app.set_my_node_id(hex::encode(identity.node_id_bytes).into());
        }
    }

    ui::wire_callbacks(&app, tx_commands.clone(), tx_events);
    #[cfg(target_os = "android")]
    {
        let app_weak = app.as_weak();
        app.window().on_close_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                if app.get_has_identity() && !app.get_is_locked() && app.get_current_screen() != 0 {
                    app.set_current_screen(0);
                    return slint::CloseRequestResponse::KeepWindowShown;
                }
            }
            slint::CloseRequestResponse::HideWindow
        });
    }
    let _ = tx_commands.try_send(Command::RefreshLocalInfo);

    #[cfg(target_os = "android")]
    {
        let foreground = ANDROID_FOREGROUND.load(std::sync::atomic::Ordering::Relaxed);
        let _ = tx_commands.try_send(Command::SetAppForeground { foreground });
    }

    app.run()?;

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .try_init();
}

fn install_panic_hook(tx_events: tokio::sync::broadcast::Sender<core::commands::AppEvent>) {
    std::panic::set_hook(Box::new(move |info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".to_owned());
        let location = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_owned());
        let message = format!("panic at {location}: {payload}");
        let _ = tx_events.send(core::commands::AppEvent::Error { message: message.clone() });
        tracing::error!("{}", message);
    }));
}
