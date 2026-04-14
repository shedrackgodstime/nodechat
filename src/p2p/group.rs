//! Group transport built on Iroh Gossip subscriptions and broadcasts.

use anyhow::Result;
use iroh_gossip::proto::TopicId;
use sha2::{Digest, Sha256};
use futures_lite::StreamExt;
use super::{NetworkManager, NetworkEvent};

/// Subscribe to the gossip swarm for `topic_id`.
///
/// The caller is responsible for ensuring repeated subscriptions are treated idempotently.
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

    // Start receiving traffic for this topic and forward events into the backend.
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
/// This variant does not spawn a receiver task and is used to refresh bootstrap peers.
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
    let _res = gossip.subscribe(id, bootstrap).await?;
    Ok(())
}

/// Broadcast `payload` to the gossip topic identified by `topic_id`.
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
    
    // Submit the payload even if no neighbors are currently visible. Gossip handles fan-out
    // once peers are available and this keeps the caller's send path simple.
    let res = gossip.subscribe(id, vec![]).await?;
    let (sender, _) = res.split();
    sender.broadcast(payload.into()).await?;
    
    tracing::info!(topic = %topic_id, "Gossip: broadcast payload pushed to engine");

    Ok(())
}

/// Leave the gossip swarm for `topic_id`.
pub(crate) async fn leave(
    manager: NetworkManager,
    topic_id: &str,
) -> Result<()> {
    let _gossip = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.gossip.clone().ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?
    };
    let _id = derive_topic_id(topic_id);
    // Subscription lifetime is currently managed by the spawned receiver task, so local
    // neighbor state is the only bookkeeping cleared here.
    {
        let mut inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.group_neighbors.insert(topic_id.to_string(), 0);
    }
    
    Ok(())
}

/// Derives a deterministic 32-byte gossip topic identifier from the application topic string.
fn derive_topic_id(name: &str) -> TopicId {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    TopicId::from_bytes(hasher.finalize().into())
}
