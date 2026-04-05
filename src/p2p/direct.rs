//! Iroh unicast — direct 1:1 message delivery.

use anyhow::Result;

use super::{NetworkManager, DIRECT_ALPN};

/// Send `payload` to `target_node_id` over Iroh direct connection.
///
/// Reuses `manager.connections[target_node_id]` if available.
/// Otherwise attempts Pkarr discovery first (RULES.md P-03), then connects.
///
/// # Errors
/// Returns an error if the peer is unreachable. Caller is responsible for queuing.
pub(crate) async fn send(
    manager: &NetworkManager,
    target_node_id: &str,
    dial_hint: Option<&str>,
    payload: Vec<u8>,
) -> Result<()> {
    let endpoint = manager.endpoint.as_ref()
        .ok_or_else(|| anyhow::anyhow!("endpoint not initialised"))?;

    let target: iroh::EndpointAddr = if let Some(hint) = dial_hint {
        if let Ok(ticket) = hint.parse::<iroh_tickets::endpoint::EndpointTicket>() {
            ticket.endpoint_addr().clone()
        } else {
            let target: iroh::EndpointId = hint.parse()?;
            target.into()
        }
    } else {
        let target: iroh::EndpointId = target_node_id.parse()?;
        target.into()
    };

    // Reuse existing connection if possible (RULES.md P-04)
    let conn = {
        let maybe_conn = manager
            .connections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(target_node_id)
            .cloned();
        if let Some(conn) = maybe_conn {
            conn
        } else {
            let conn: iroh::endpoint::Connection = endpoint.connect(target, DIRECT_ALPN).await?;
            manager
                .connections
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(target_node_id.to_owned(), conn.clone());
            manager.spawn_direct_reader(target_node_id.to_owned(), conn.clone());
            conn
        }
    };

    // Open uni-directional stream for fire-and-forget message delivery
    let mut send_stream = conn.open_uni().await?;
    send_stream.write_all(&payload).await?;
    send_stream.finish()?;

    tracing::debug!(
        peer = target_node_id,
        payload_len = payload.len(),
        "Direct message sent successfully"
    );

    Ok(())
}
