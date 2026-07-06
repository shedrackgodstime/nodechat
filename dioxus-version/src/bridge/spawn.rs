use crate::bridge::Bridge;
use crate::contract::{Command, Event};

pub fn spawn_backend() -> Bridge {
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<Command>(64);
    let (evt_tx, evt_rx) = tokio::sync::mpsc::channel::<Event>(64);

    spawn(async move {
        tracing::info!("backend: started");
        while let Some(cmd) = cmd_rx.recv().await {
            tracing::info!("backend: received command {:?}", cmd);
            match cmd {
                Command::RequestSnapshot => {
                    let _ = evt_tx.send(Event::SnapshotLoaded { snapshot: Default::default() }).await;
                }
                _ => {
                    tracing::warn!("backend: unhandled command {:?}", cmd);
                }
            }
        }
        tracing::info!("backend: stopped");
    });

    Bridge::new(cmd_tx, evt_rx)
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    tokio::task::spawn(f);
}

#[cfg(target_arch = "wasm32")]
fn spawn<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
