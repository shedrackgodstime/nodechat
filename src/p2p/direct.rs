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
    _manager: &mut NetworkManager,
    target_node_id: &str,
    payload: Vec<u8>,
) -> Result<()> {
    // WIRE: iroh 0.97.0
    //
    // let endpoint = manager.endpoint.as_ref()
    //     .ok_or_else(|| anyhow::anyhow!("endpoint not initialised"))?;
    //
    // // Reuse existing connection (RULES.md P-04).
    // if let Some(conn) = manager.connections.get(target_node_id) {
    //     let mut send_stream = conn.open_uni().await?;
    //     send_stream.write_all(&payload).await?;
    //     send_stream.finish().await?;
    //     return Ok(());
    // }
    //
    // // Pkarr discovery before queuing (RULES.md P-03).
    // let node_id: iroh::NodeId = target_node_id.parse()
    //     .context("invalid node id")?;
    // let addr = endpoint.resolve(node_id).await
    //     .context("pkarr discovery failed for peer")?;
    //
    // let conn = endpoint.connect(addr, b"nodechat/alpha").await
    //     .context("failed to connect to peer")?;
    // let mut send_stream = conn.open_uni().await?;
    // send_stream.write_all(&payload).await?;
    // send_stream.finish().await?;
    // manager.connections.insert(target_node_id.to_owned(), conn);

    tracing::warn!(
        peer = target_node_id,
        payload_len = payload.len(),
        "direct::send — iroh 0.97.0 wiring pending"
    );
    Ok(())
}
