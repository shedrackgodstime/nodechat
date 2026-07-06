pub mod spawn;

use crate::contract::{Command, Event};

#[allow(dead_code)]
pub struct Bridge {
    cmd_tx: tokio::sync::mpsc::Sender<Command>,
    evt_rx: tokio::sync::mpsc::Receiver<Event>,
}

#[allow(dead_code)]
impl Bridge {
    pub fn new(
        cmd_tx: tokio::sync::mpsc::Sender<Command>,
        evt_rx: tokio::sync::mpsc::Receiver<Event>,
    ) -> Self {
        Self { cmd_tx, evt_rx }
    }

    pub async fn send(&self, cmd: Command) -> anyhow::Result<()> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| anyhow::anyhow!("bridge channel closed"))
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.evt_rx.recv().await
    }

    pub fn try_recv(&mut self) -> Option<Event> {
        self.evt_rx.try_recv().ok()
    }
}
