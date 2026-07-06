use crate::types::{Chat, Message};

pub fn create_chat(
    id: &str,
    name: &str,
    initials: &str,
    is_group: bool,
    is_online: bool,
    is_verified: bool,
) -> Chat {
    Chat {
        id: id.to_string(),
        name: name.to_string(),
        initials: initials.to_string(),
        last_message: "Chat established.".to_string(),
        timestamp: "Now".to_string(),
        unread: 0,
        is_group,
        is_online,
        is_verified,
        has_queued: false,
        messages: vec![],
    }
}

pub fn send_message(
    chats: &mut [Chat],
    chat_id: &str,
    text: &str,
    is_online: bool,
) -> Option<(String, bool)> {
    let pos = chats.iter().position(|c| c.id == chat_id)?;
    let msg_id = format!("m_{}", chats[pos].messages.len());

    let mut is_contact_share = false;
    let mut share_name = String::new();
    let mut share_node_id = String::new();
    let mut share_ticket = String::new();

    if text.starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
            if let (Some(n), Some(node)) = (
                val.get("name").and_then(|v| v.as_str()),
                val.get("node_id").and_then(|v| v.as_str()),
            ) {
                is_contact_share = true;
                share_name = n.to_string();
                share_node_id = node.to_string();
                share_ticket = val
                    .get("ticket")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    chats[pos].messages.push(Message {
        id: msg_id.clone(),
        sender: "You".to_string(),
        text: text.to_string(),
        timestamp: "Just now".to_string(),
        is_outgoing: true,
        status: "queued".to_string(),
        is_contact_share,
        share_name: share_name.clone(),
        share_node_id,
        share_ticket,
        ..Default::default()
    });

    if is_contact_share {
        chats[pos].last_message = format!("Shared contact: {}", share_name);
    } else {
        chats[pos].last_message = text.to_string();
    }

    Some((msg_id, is_online))
}

pub async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
    }
}
