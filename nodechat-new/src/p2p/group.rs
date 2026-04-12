//! Iroh Gossip — encrypted group broadcast swarm.
//!
//! Implements `NetworkManager::broadcast_group` and `NetworkManager::subscribe_group`.
//! Wired for Iroh 0.97.0 (RULES.md P-02).

use anyhow::Result;
use iroh_gossip::proto::TopicId;
use sha2::{Digest, Sha256};
use futures_lite::StreamExt;
use super::{NetworkManager, NetworkEvent};

/// Subscribe to the gossip swarm for `topic_id`.
///
/// Idempotency is enforced by the caller in `NetworkManager::subscribe_group` (RULES.md P-05).
///
/// # Errors
/// Returns an error if the gossip subscription fails.
pub(crate) async fn subscribe(
    manager: NetworkManager,
    topic_id: &str,
    bootstrap: Vec<iroh::EndpointId>,
) -> Result<()> {
    let (gossip, event_tx) = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        let g = inner.gossip.clone().ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?;
        let tx = inner.event_tx.clone();
        (g, tx)
    };

    let id = derive_topic_id(topic_id);
    let topic_id_owned = topic_id.to_owned();

    // Join the topic swarm with bootstrap peers (RULES.md P-02)
    let res = gossip.subscribe(id, bootstrap).await?;
    let (_sender, mut receiver) = res.split();

    tokio::spawn(async move {
        tracing::info!("Subscribed to gossip topic: {}", topic_id_owned);
        while let Some(event) = receiver.next().await {
            match event {
                Ok(iroh_gossip::api::Event::Received(msg)) => {
                    let _ = event_tx.send(NetworkEvent::GroupMessage {
                        topic: topic_id_owned.clone(),
                        from: msg.delivered_from.to_string(),
                        payload: msg.content.to_vec(),
                    }).await;
                }
                Ok(iroh_gossip::api::Event::NeighborUp(peer)) => {
                    manager.update_group_neighbor_count(&topic_id_owned, 1);
                    let _ = event_tx.send(NetworkEvent::GroupNeighborUp {
                        topic: topic_id_owned.clone(),
                        node_id: peer.to_string(),
                    }).await;
                }
                Ok(iroh_gossip::api::Event::NeighborDown(peer)) => {
                    manager.update_group_neighbor_count(&topic_id_owned, -1);
                    let _ = event_tx.send(NetworkEvent::GroupNeighborDown {
                        topic: topic_id_owned.clone(),
                        node_id: peer.to_string(),
                    }).await;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// Join the gossip swarm for `topic_id` with additional bootstrap peers.
/// Does not spawn a receiver task. Use this to update an existing subscription.
pub(crate) async fn join_topic(
    manager: NetworkManager,
    topic_id: &str,
    bootstrap: Vec<iroh::EndpointId>,
) -> Result<()> {
    let gossip = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.gossip.clone().ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?
    };
    let id = derive_topic_id(topic_id);
    let _Res = gossip.subscribe(id, bootstrap).await?;
    Ok(())
}

/// Broadcast `payload` to all members of `topic_id` gossip swarm.
pub(crate) async fn broadcast(
    manager: NetworkManager,
    topic_id: &str,
    payload: Vec<u8>,
) -> Result<()> {
    let gossip = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.gossip.clone().ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?
    };

    let id = derive_topic_id(topic_id);
    
    // We attempt to broadcast regardless of local neighbor count.
    // The Gossip engine will handle propagation if/when neighbors appear.
    // This prevents the flush loop from failing and allows the message to reach the engine.
    let res = gossip.subscribe(id, vec![]).await?;
    let (sender, _) = res.split();
    sender.broadcast(payload.into()).await?;
    
    tracing::info!(topic = %topic_id, "Gossip: broadcast payload pushed to engine");

    Ok(())
}

/// Helper to derive a 32-byte TopicId from a string name (RULES.md P-02).
fn derive_topic_id(name: &str) -> TopicId {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    TopicId::from_bytes(hasher.finalize().into())
}
