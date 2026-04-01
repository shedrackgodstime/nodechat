//! Iroh Gossip — encrypted group broadcast swarm.
//!
//! WIRE: Implements `NetworkManager::broadcast_group` and `NetworkManager::subscribe_group`.
//! All function bodies are scaffolded pending iroh-gossip 0.97.0 API verification.

use anyhow::Result;

use super::NetworkManager;

/// Subscribe to the gossip swarm for `topic_id`.
///
/// Idempotency is enforced by the caller in `NetworkManager::subscribe_group` (RULES.md P-05).
///
/// # Errors
/// Returns an error if the gossip subscription fails.
pub(crate) async fn subscribe(_manager: &mut NetworkManager, topic_id: &str) -> Result<()> {
    // WIRE: iroh-gossip 0.97.0
    //
    // let gossip = manager.gossip.as_ref()
    //     .ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?;
    //
    // let topic: iroh_gossip::TopicId = topic_id.parse()
    //     .context("invalid topic id")?;
    //
    // let (sink, stream) = gossip.subscribe(topic, vec![]).await?;
    //
    // // Spawn a task that reads events from `stream` and emits them to the core worker.
    // let event_tx = manager.event_tx.clone();
    // let topic_id_owned = topic_id.to_owned();
    // tokio::spawn(async move {
    //     while let Some(event) = stream.try_next().await {
    //         match event {
    //             iroh_gossip::Event::Received(msg) => {
    //                 let from = hex::encode(msg.delivered_from.as_bytes());
    //                 let _ = event_tx.send(super::NetworkEvent::GroupMessage {
    //                     topic: topic_id_owned.clone(),
    //                     from,
    //                     payload: msg.content.to_vec(),
    //                 }).await;
    //             }
    //             _ => {}
    //         }
    //     }
    // });

    tracing::warn!(topic = topic_id, "group::subscribe — iroh-gossip 0.97.0 wiring pending");
    Ok(())
}

/// Broadcast `payload` to all members of `topic_id` gossip swarm.
///
/// # Errors
/// Returns an error if no subscription exists for the topic or the broadcast fails.
pub(crate) async fn broadcast(
    _manager: &NetworkManager,
    topic_id: &str,
    payload: Vec<u8>,
) -> Result<()> {
    // WIRE: iroh-gossip 0.97.0
    //
    // let gossip = manager.gossip.as_ref()
    //     .ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?;
    //
    // let topic: iroh_gossip::TopicId = topic_id.parse()
    //     .context("invalid topic id")?;
    //
    // gossip.broadcast(topic, bytes::Bytes::from(payload)).await
    //     .context("gossip broadcast failed")?;

    tracing::warn!(
        topic = topic_id,
        payload_len = payload.len(),
        "group::broadcast — iroh-gossip 0.97.0 wiring pending"
    );
    Ok(())
}
