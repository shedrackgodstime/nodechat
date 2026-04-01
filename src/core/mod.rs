pub mod commands;

use tokio::sync::mpsc;

pub struct NodeChatWorker {
    rx_commands: mpsc::Receiver<commands::Command>,
}

impl NodeChatWorker {
    pub async fn new(rx_commands: mpsc::Receiver<commands::Command>) -> Self {
        Self { rx_commands }
    }

    pub async fn run(mut self) {
        while let Some(cmd) = self.rx_commands.recv().await {
            self.handle_command(cmd).await;
        }
    }

    async fn handle_command(&mut self, cmd: commands::Command) {
        match cmd {
            commands::Command::Ping { respond_to } => {
                let _ = respond_to.send(commands::AppEvent::Pong);
            }
        }
    }
}
