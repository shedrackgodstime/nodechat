use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

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
