//! Iroh unicast — direct 1:1 message delivery.
//!
//! WIRE: Implements `NetworkManager::send_direct`.
//! All function bodies are scaffolded pending iroh 0.97.0 API verification.

use anyhow::Result;

use super::NetworkManager;

/// Send `payload` to `target_node_id` over Iroh direct connection.
///
/// Reuses `manager.connections[target_node_id]` if available.
/// Otherwise attempts Pkarr discovery first (RULES.md P-03), then connects.
///
/// # Errors
/// Returns an error if the peer is unreachable. Caller is responsible for queuing.
pub(crate) async fn send(
    manager: &mut NetworkManager,
    target_node_id: &str,
    payload: Vec<u8>,
) -> Result<()> {
    let endpoint = manager.endpoint.as_ref()
        .ok_or_else(|| anyhow::anyhow!("endpoint not initialised"))?;

    let target: iroh::EndpointId = target_node_id.parse()?;

    // Reuse existing connection if possible (RULES.md P-04)
    let conn = if let Some(conn) = manager.connections.get(target_node_id) {
        conn.clone()
    } else {
        // Iroh 0.97.0 connect with ALPN
        let conn = endpoint.connect(iroh::EndpointAddr::from(target), b"nodechat/alpha").await?;
        manager.connections.insert(target_node_id.to_owned(), conn.clone());
        conn
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
