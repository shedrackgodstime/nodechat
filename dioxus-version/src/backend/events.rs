use crate::contract::Event;

pub async fn handle_network_event(
    event: Event,
    // storage: &Database,
    // network: &Network,
) -> Option<Event> {
    match &event {
        Event::DirectMessageReceived { message } => {
            tracing::info!("received direct message from {}", message.sender_name);
            Some(event)
        }
        Event::GroupMessageReceived { message } => {
            tracing::info!("received group message in {}", message.chat_id);
            Some(event)
        }
        _ => None,
    }
}
