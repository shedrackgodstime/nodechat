use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tracing::field::{Field, Visit};

use crate::backend::RealBackend;
use crate::contract::{AppEvent, Command};

#[derive(Clone)]
pub struct UiBridge {
    command_tx: mpsc::Sender<Command>,
    event_rx: Arc<Mutex<mpsc::Receiver<AppEvent>>>,
}

impl UiBridge {
    pub fn send(&self, command: Command) -> Result<(), mpsc::SendError<Command>> {
        self.command_tx.send(command)
    }

    pub fn try_recv(&self) -> Option<AppEvent> {
        self.event_rx.lock().ok()?.try_recv().ok()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<AppEvent> {
        self.event_rx
            .lock()
            .ok()?
            .recv_timeout(timeout)
            .ok()
    }

    pub fn drain_events(&self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let guard = match self.event_rx.lock() {
            Ok(guard) => guard,
            Err(_) => return events,
        };

        while let Ok(event) = guard.try_recv() {
            events.push(event);
        }

        events
    }
}
// Tracing layer that mirrors selected log events into the in-app debug console.

struct UiLogLayer {
    event_tx: mpsc::SyncSender<AppEvent>,
}

// Extracts the formatted `message` field from tracing events.
struct MsgVisitor(String);
impl Visit for MsgVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" { self.0 = value.to_string(); }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let s = format!("{value:?}");
            // Remove the extra quotes added by `Debug` formatting for string values.
            self.0 = if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len()-1].to_string()
            } else { s };
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for UiLogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let target = event.metadata().target();
        let level  = event.metadata().level();

        // Keep full visibility for application logs and only surface warnings from dependencies.
        let is_ours  = target.to_lowercase().contains("nodechat");
        
        let is_net   = target.starts_with("iroh")
                    || target.starts_with("quinn")
                    || target.starts_with("rustls")
                    || target.starts_with("n0_")
                    || target.starts_with("noq")
                    || target.starts_with("hickory");
        let is_noise = target.starts_with("h2")
                    || target.starts_with("hyper")
                    || target.starts_with("tokio_util")
                    || target.starts_with("want")
                    || target.starts_with("mio")
                    || target.starts_with("polling");

        let pass = if is_noise {
            false
        } else if is_ours {
            true                              // Our code = everything
        } else if is_net {
            *level <= tracing::Level::WARN    // Net libs = WARN/ERR only
        } else {
            *level <= tracing::Level::WARN    // Others = WARN/ERR only
        };

        if !pass { return; }

        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);
        let msg = visitor.0;
        if msg.is_empty() { return; }

        // Collapse the tracing target to a short module tag for the debug panel.
        let tag = {
            let parts: Vec<&str> = target.split("::").collect();
            if parts.len() <= 2 { target.to_string() }
            else { parts[parts.len()-2..].join("::") }
        };

        let lvl = match *level {
            tracing::Level::ERROR => "ERR ",
            tracing::Level::WARN  => "WARN",
            tracing::Level::INFO  => "INFO",
            tracing::Level::DEBUG => "DEBG",
            tracing::Level::TRACE => "TRCE",
        };

        let message = format!("[{}] {} | {}", lvl, tag, msg);
        let _ = self.event_tx.try_send(AppEvent::Log {
            level: lvl.trim().to_string(),
            message,
        });
    }
}


/// Production runtime — uses the real SQLite database and Tokio for P2P.
pub struct RealRuntime {
    pub ui: UiBridge,
    _runtime: tokio::runtime::Runtime,
}

impl RealRuntime {
    pub fn start() -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        // The Slint boundary is synchronous, so the UI bridge uses std channels and threads.
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

        // Buffer log forwarding separately so diagnostics never block application work.
        let (log_tx, log_rx) = mpsc::sync_channel::<AppEvent>(256);
        let ui_layer = UiLogLayer { event_tx: log_tx };

        // Keep stderr output aligned with the same filtering strategy used by the in-app panel.
        let stderr_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    // Application logs stay verbose; dependency output is reduced to actionable signals.
                    "nodechat_new=debug,\
                     iroh=warn,quinn=warn,rustls=warn,\
                     n0_=warn,noq=warn,hickory=warn,\
                     h2=off,hyper=off,tokio_util=off,mio=off,polling=off,want=off,\
                     warn"
                )
            });
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_filter(stderr_filter);

        match tracing_subscriber::registry()
            .with(stderr_layer)
            .with(ui_layer)
            .try_init() {
                Ok(_) => eprintln!("[BRIDGE] Tracing subscriber installed successfully"),
                Err(e) => eprintln!("[BRIDGE] FAILED to install tracing subscriber: {}. It might already be set.", e),
            }

        // Forward accepted log lines into the same event stream consumed by the UI.
        let event_tx_log = event_tx.clone();
        std::thread::spawn(move || {
            while let Ok(event) = log_rx.recv() {
                if event_tx_log.send(event).is_err() { break; }
            }
        });

        // Relay UI commands into Tokio so backend tasks can remain asynchronous.
        let (async_cmd_tx, mut async_cmd_rx) = tokio::sync::mpsc::channel::<Command>(100);
        thread::spawn(move || {
            while let Ok(cmd) = command_rx.recv() {
                if async_cmd_tx.blocking_send(cmd).is_err() {
                    break;
                }
            }
        });

        runtime.spawn(async move {
            eprintln!("[BRIDGE] Spawning backend worker thread...");
            let (net_tx, mut net_rx) = tokio::sync::mpsc::channel::<crate::p2p::NetworkEvent>(100);
            let mut backend = match RealBackend::open(net_tx, event_tx.clone()) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[BRIDGE] FATAL: Failed to open backend: {}", e);
                    return;
                }
            };
            eprintln!("[BRIDGE] Backend opened successfully. Sending initial snapshot.");
            let _ = event_tx.send(AppEvent::SnapshotReady(backend.snapshot()));
            let ext = event_tx.clone();
            
            loop {
                tokio::select! {
                    cmd_opt = async_cmd_rx.recv() => {
                        match cmd_opt {
                            Some(command) => {
                                for event in backend.handle_command(command).await {
                                    if ext.send(event).is_err() {
                                        return;
                                    }
                                }
                            }
                            None => break, // UI Closed
                        }
                    }
                    net_opt = net_rx.recv() => {
                        match net_opt {
                            Some(net_event) => {
                                for event in backend.handle_network_event(net_event).await {
                                    if ext.send(event).is_err() {
                                        return;
                                    }
                                }
                            }
                            None => break, // Network Closed
                        }
                    }
                }
            }
        });

        Ok(Self {
            ui: UiBridge {
                command_tx,
                event_rx: Arc::new(Mutex::new(event_rx)),
            },
            _runtime: runtime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::AppEvent;
    use std::sync::{mpsc, Arc, Mutex};

    #[test]
    fn drain_events_collects_pending_events() {
        let (command_tx, _command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let bridge = UiBridge {
            command_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        };

        event_tx.send(AppEvent::StatusNotice("first".to_string())).unwrap();
        event_tx.send(AppEvent::StatusNotice("second".to_string())).unwrap();

        let events = bridge.drain_events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], AppEvent::StatusNotice(_)));
        assert!(bridge.drain_events().is_empty());
    }
}
