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
    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(100);
    let (tx_events, _rx_events)    = tokio::sync::broadcast::channel(512);
    init_tracing(tx_events.clone());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build Tokio runtime")?;

    let _guard = runtime.enter();

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
    #[cfg(target_os = "android")]
    app.set_is_desktop(false);
    #[cfg(not(target_os = "android"))]
    app.set_is_desktop(true);

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
                is_session_ready: false,
                is_relay: chat.is_relay,
                is_queued: chat.is_queued,
                is_verified: chat.is_verified,
            })
            .collect();
        app.set_chats(slint::VecModel::from_slice(&chat_rows));

        match storage::queries::get_local_identity(&startup_db) {
            Ok(Some(identity)) => {
                let initials = storage::queries::derive_initials(&identity.display_name);
                app.set_has_identity(true);
                app.set_is_locked(true);
                app.set_setup_step(0);
                app.set_current_screen(0);
                app.set_my_display_name(identity.display_name.into());
                app.set_my_initials(initials.into());
                app.set_my_node_id(hex::encode(identity.node_id_bytes).into());
                tracing::info!(node_id = %app.get_my_node_id(), "identity found at startup, showing lock screen");
            }
            Ok(None) => {
                app.set_has_identity(false);
                app.set_setup_step(0);
                tracing::info!("no identity found at startup, showing onboarding");
            }
            Err(e) => {
                tracing::error!("failed to check local identity at startup: {:?}", e);
                // Fallback to onboarding but log the disaster
                app.set_has_identity(false);
                app.set_setup_step(0);
            }
        }
    }

    ui::wire_callbacks(&app, tx_commands.clone(), tx_events);
    #[cfg(target_os = "android")]
    {
        let app_weak = app.as_weak();
        app.window().on_close_requested(move || {
            if let Some(app) = app_weak.upgrade() {
                // If on Home (0) or Welcome/Setup screens, allow minimizing.
                if app.get_current_screen() == 0 || (!app.get_has_identity() && app.get_setup_step() == 0) {
                    return slint::CloseRequestResponse::HideWindow;
                }

                // Mirror the Slint back-navigation rules here because Android close
                // requests can arrive before the UI handles a key event.
                let next_screen = match app.get_current_screen() {
                    9 | 10 => app.get_active_conversation().return_screen,
                    1 | 2 | 5 | 6 => 0,
                    3 => app.get_navigation_return_screen(),
                    7 => 6,
                    8 => 7,
                    11 => 5,
                    _ => 0,
                };
                app.set_current_screen(next_screen);
                return slint::CloseRequestResponse::KeepWindowShown;
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

fn init_tracing(tx_events: tokio::sync::broadcast::Sender<core::commands::AppEvent>) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    // 🎯 Terminal/Console Filter (cleaner cargo run)
    // - Libraries (iroh, quinn, winit, rustls) default to INFO
    // - NodeChat code defaults to DEBUG for development visibility
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,nodechat=debug,NodeChat=debug"));

    let ui_layer = UIEventLayer { tx: tx_events };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_ansi(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(ui_layer)
        .try_init();
}

struct UIEventLayer {
    tx: tokio::sync::broadcast::Sender<core::commands::AppEvent>,
}

impl<S> tracing_subscriber::Layer<S> for UIEventLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = metadata.level();
        let target = metadata.target();

        // 🧠 Filtering Logic for UI Console (RULES.md P-01/U-04 compliance)
        // We only want logs that are relevant to the user/developer of this app.
        // Deep library traces from iroh/quinn/rustls will flood the Slint thread and cause lag.
        
        let is_nodechat = target.contains("nodechat") || target.contains("NodeChat");
        
        let should_log = match *level {
            tracing::Level::ERROR | tracing::Level::WARN => true,
            tracing::Level::INFO => {
                // For INFO, we want our logs + important library events, but NOT the noisy ones.
                is_nodechat || (!target.starts_with("iroh") && !target.starts_with("quinn") && !target.starts_with("rustls"))
            }
            tracing::Level::DEBUG | tracing::Level::TRACE => {
                // For DEBUG/TRACE, ONLY show logs from our own project.
                is_nodechat
            }
        };

        if !should_log {
            return;
        }

        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message;
        let level_str = level.to_string();
        let target_str = target.to_string();

        let _ = self.tx.send(core::commands::AppEvent::Log {
            level: level_str,
            target: target_str,
            message,
        });
    }
}

#[derive(Default)]
struct LogVisitor {
    message: String,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Strip quotes from debug-printed strings
            if self.message.starts_with('\"') && self.message.ends_with('\"') && self.message.len() >= 2 {
                self.message = self.message[1..self.message.len()-1].to_string();
            }
        }
    }
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
