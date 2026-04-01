use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Command {
    Ping { respond_to: mpsc::Sender<AppEvent> },
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Pong,
}
