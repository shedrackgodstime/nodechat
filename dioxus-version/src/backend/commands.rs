use crate::contract::Command;
use crate::contract::Event;

pub async fn handle_command(
    cmd: Command,
    // storage: &Database,
    // network: &Network,
    // evt_tx: &mpsc::Sender<Event>,
) -> Option<Event> {
    match cmd {
        Command::RequestSnapshot => {
            Some(Event::SnapshotLoaded {
                snapshot: Default::default(),
            })
        }
        Command::GenerateIdentity { display_name } => {
            tracing::info!("generating identity for '{}'", display_name);
            Some(Event::IdentityCreated {
                identity: crate::contract::IdentityView {
                    id: uuid::Uuid::new_v4().to_string(),
                    display_name,
                    public_key: String::new(),
                    created_at: chrono::Utc::now().timestamp(),
                },
            })
        }
        _ => {
            tracing::warn!("unhandled command: {:?}", cmd);
            None
        }
    }
}
