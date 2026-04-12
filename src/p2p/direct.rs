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
    let (endpoint, connections) = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        let ep = inner.endpoint.clone().ok_or_else(|| anyhow::anyhow!("endpoint not initialised"))?;
        // We can't return a reference under lock, so we clone common stuff
        (ep, manager.inner.clone()) 
    };

    let target: iroh::EndpointAddr = if let Some(hint) = dial_hint {
        if let Ok(ticket) = hint.parse::<iroh_tickets::endpoint::EndpointTicket>() {
            ticket.endpoint_addr().clone()
        } else {
            let target: iroh::EndpointId = hint.parse().map_err(|e| anyhow::anyhow!("Invalid peer ID: {}", e))?;
            target.into()
        }
    } else {
        let target: iroh::EndpointId = target_node_id.parse().map_err(|e| anyhow::anyhow!("Invalid target ID: {}", e))?;
        target.into()
    };

    // Reuse existing connection if possible (RULES.md P-04)
    let conn = {
        let maybe_conn = {
            let inner = connections.lock().unwrap_or_else(|p| p.into_inner());
            let conns = inner.connections.lock().unwrap_or_else(|p| p.into_inner());
            conns.get(target_node_id).cloned()
        };

        if let Some(conn) = maybe_conn {
            tracing::info!(peer = %target_node_id, "Direct: reusing existing connection");
            conn
        } else {
            tracing::info!(peer = %target_node_id, "Direct: no existing connection, dialing peer...");
            let connect_fut = endpoint.connect(target, DIRECT_ALPN);
            let conn: iroh::endpoint::Connection = match tokio::time::timeout(std::time::Duration::from_secs(10), connect_fut).await {
                Ok(Ok(c)) => {
                    tracing::info!(peer = %target_node_id, "Direct: connection established successfully");
                    c
                }
                Ok(Err(e)) => {
                    tracing::error!(peer = %target_node_id, "Direct: connection failed: {}", e);
                    return Err(e.into());
                }
                Err(_) => {
                    tracing::error!(peer = %target_node_id, "Direct: connection TIMED OUT after 10s");
                    return Err(anyhow::anyhow!("connection timeout"));
                }
            };
            
            {
                let inner = connections.lock().unwrap_or_else(|p| p.into_inner());
                let mut conns = inner.connections.lock().unwrap_or_else(|p| p.into_inner());
                conns.insert(target_node_id.to_owned(), conn.clone());
            }
            manager.spawn_direct_reader(target_node_id.to_owned(), conn.clone());
            conn
        }
    };

    tracing::info!(peer = %target_node_id, "Direct: opening stream for payload (len={})", payload.len());

    // Open uni-directional stream for fire-and-forget message delivery
    let result = async {
        let mut send_stream = conn.open_uni().await?;
        send_stream.write_all(&payload).await?;
        send_stream.finish()?;
        Ok::<(), anyhow::Error>(())
    }.await;

    if let Err(e) = result {
        tracing::error!(peer = %target_node_id, "Direct: stream failed (peer might be offline): {}. Purging dead connection.", e);
        // CRITICAL: if the connection failed, remove it from the map so we dial fresh next time
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut conns = inner.connections.lock().unwrap_or_else(|p| p.into_inner());
        conns.remove(target_node_id);
        return Err(e);
    }

    tracing::debug!(
        peer = target_node_id,
        payload_len = payload.len(),
        "Direct message sent successfully"
    );

    Ok(())
}
