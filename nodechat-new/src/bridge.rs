use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tracing::field::{Field, Visit};

use crate::backend::RealBackend;
use crate::contract::{AppEvent, Command};
use crate::MockBackend;

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

pub struct MockRuntime {
    pub ui: UiBridge,
}

impl MockRuntime {
    pub fn start() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>();
        let backend = MockBackend::new();

        thread::spawn(move || {
            let mut backend = backend;
            let _ = event_tx.send(AppEvent::SnapshotReady(backend.snapshot()));

            while let Ok(command) = command_rx.recv() {
                for event in backend.handle_command(command) {
                    if event_tx.send(event).is_err() {
                        return;
                    }
                }
            }
        });

        Self {
            ui: UiBridge {
                command_tx,
                event_rx: Arc::new(Mutex::new(event_rx)),
            },
        }
    }
}  // end impl MockRuntime

// ── UiLogLayer ───────────────────────────────────────────────────────────────
// A tracing Layer that pipes log events to both:
//   1. stderr   (so the terminal shows colour output while developing)
//   2. the UI   (so the in-app debug panel is live)

struct UiLogLayer {
    event_tx: mpsc::SyncSender<AppEvent>,
}

// Visitor that pulls the "message" field out of a tracing event.
struct MsgVisitor(String);
impl Visit for MsgVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" { self.0 = value.to_string(); }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let s = format!("{value:?}");
            // strip surrounding quotes that Debug adds to strings
            self.0 = if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len()-1].to_string()
            } else { s };
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for UiLogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let target = event.metadata().target();
        // Only capture our own crate's events to avoid flooding with iroh/quinn noise.
        if !target.starts_with("nodechat") {
            return;
        }
        let level = event.metadata().level().to_string().to_uppercase();
        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);
        let message = format!("[{}] [{}] {}", level, target, visitor.0);
        // Non-blocking: drop the log if the channel is full.
        let _ = self.event_tx.try_send(AppEvent::Log { level, message });
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

        // We still use std threads/channels for UI boundary since Slint is sync on one end
        let (command_tx, command_rx) = mpsc::channel::<Command>();
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

        // ── Tracing subscriber ──────────────────────────────────────────────
        // Sync channel (capacity 256) so the UI layer never blocks the backend.
        let (log_tx, log_rx) = mpsc::sync_channel::<AppEvent>(256);
        let ui_layer = UiLogLayer { event_tx: log_tx };

        // stderr layer: show ALL events from our crate at INFO+, noisy iroh libraries at WARN+
        let stderr_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "nodechat_new=debug,iroh=warn,quinn=warn,rustls=warn,warn"
                )
            });
        let stderr_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_filter(stderr_filter);

        let _ = tracing_subscriber::registry()
            .with(stderr_layer)
            .with(ui_layer)
            .try_init(); // try_init so a second call (tests) doesn't panic

        // Drain log events from the sync channel into the main event channel.
        let event_tx_log = event_tx.clone();
        std::thread::spawn(move || {
            while let Ok(event) = log_rx.recv() {
                if event_tx_log.send(event).is_err() { break; }
            }
        });

        // Bridge the std sync channel -> tokio async channel
        let (async_cmd_tx, mut async_cmd_rx) = tokio::sync::mpsc::channel::<Command>(100);
        thread::spawn(move || {
            while let Ok(cmd) = command_rx.recv() {
                if async_cmd_tx.blocking_send(cmd).is_err() {
                    break;
                }
            }
        });

        runtime.spawn(async move {
            let (net_tx, mut net_rx) = tokio::sync::mpsc::channel::<crate::p2p::NetworkEvent>(100);
            let mut backend = RealBackend::open(net_tx).expect("failed to open local database");
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
    use crate::contract::Command;

    #[test]
    fn runtime_emits_snapshot_then_command_results() {
        let runtime = MockRuntime::start();

        let snapshot = loop {
            if let Some(event) = runtime.ui.try_recv() {
                if let AppEvent::SnapshotReady(snapshot) = event {
                    break snapshot;
                }
            }
        };

        assert!(!snapshot.chat_list.is_empty());

        runtime.ui.send(Command::Refresh).expect("command send should work");

        let events = runtime
            .ui
            .recv_timeout(Duration::from_millis(200))
            .into_iter()
            .collect::<Vec<_>>();
        assert!(
            events
                .iter()
                .any(|event| matches!(event, AppEvent::SnapshotReady(_)))
        );
    }
}
