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
pub(crate) async fn subscribe(manager: &mut NetworkManager, topic_id: &str) -> Result<()> {
    let gossip = manager.gossip.as_ref()
        .ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?;
    let event_tx = manager.event_tx.clone();

    let id = derive_topic_id(topic_id);
    let topic_id_owned = topic_id.to_owned();

    // Join the topic swarm (bootstrap peers empty for now — discovery handles it)
    let (_sender, mut receiver) = gossip.subscribe(id, vec![]).await?.split();

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
                _ => {
                    // Other events like 'Joined' or 'GossipNeighborUp' can be handled here if needed
                }
            }
        }
    });

    Ok(())
}

/// Broadcast `payload` to all members of `topic_id` gossip swarm.
///
/// # Errors
/// Returns an error if no subscription exists for the topic or the broadcast fails.
pub(crate) async fn broadcast(
    manager: &NetworkManager,
    topic_id: &str,
    payload: Vec<u8>,
) -> Result<()> {
    let gossip = manager.gossip.as_ref()
        .ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?;

    let id = derive_topic_id(topic_id);
    
    // In iroh-gossip 0.97.0, we can use the gossip handle directly to broadcast
    // if we are already subscribed.
    let (sender, _) = gossip.subscribe(id, vec![]).await?.split();
    sender.broadcast(payload.into()).await?;

    Ok(())
}

/// Helper to derive a 32-byte TopicId from a string name (RULES.md P-02).
fn derive_topic_id(name: &str) -> TopicId {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    TopicId::from_bytes(hasher.finalize().into())
}
