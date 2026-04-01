//! P2P transport layer — direct unicast via Iroh, group broadcast via Iroh Gossip.
//!
//! IMPORTANT: This module is scaffolded. The internal `iroh` and `iroh-gossip` API
//! calls must be verified against the pinned iroh 0.97.0 documentation before
//! filling in the bodies marked `// WIRE:`. The public interface and types are final.
//!
//! See `src/p2p/direct.rs` for 1:1 unicast and `src/p2p/group.rs` for gossip.

use anyhow::{bail, Result};
use std::collections::HashMap;

pub mod direct;
pub mod group;

/// Maximum bytes allowed in a single direct message payload (plaintext + overhead).
const MAX_PAYLOAD_BYTES: usize = 64 * 1024; // 64 KiB

/// Events produced by the network layer and consumed by the core worker.
#[derive(Debug)]
pub enum NetworkEvent {
    /// Raw encrypted bytes received from a direct P2P peer.
    DirectMessage {
        /// Hex-encoded NodeId of the sender.
        from: String,
        /// Raw ciphertext payload (nonce prepended).
        payload: Vec<u8>,
    },

    /// Raw encrypted bytes received from a gossip group swarm.
    GroupMessage {
        /// Hex-encoded TopicId of the originating group.
        topic: String,
        /// Hex-encoded NodeId of the sender within the group.
        from: String,
        /// Raw ciphertext payload.
        payload: Vec<u8>,
    },

    /// A previously unknown peer has connected.
    PeerConnected {
        /// Hex-encoded NodeId.
        node_id: String,
        /// `true` if the connection is via DERP relay (RULES.md P-06).
        via_relay: bool,
    },

    /// A peer has disconnected.
    PeerDisconnected {
        /// Hex-encoded NodeId.
        node_id: String,
    },
}

/// Manages all network connections: Iroh direct connections for 1:1 and Iroh Gossip for groups.
///
/// Iroh connections are reused per peer — not reopened per message (RULES.md P-04).
/// Gossip subscriptions are idempotent — subscribing twice is a no-op (RULES.md P-05).
// Scaffold: fields are unused until iroh 0.97.0 API is wired. Remove this allow before Phase 3.
#[allow(dead_code)]
pub struct NetworkManager {
    /// Iroh endpoint. Concrete type is iroh::Endpoint — kept as `Option` until
    /// `initialize` is called so the struct can be constructed before `async` context.
    // WIRE: replace with iroh::Endpoint once API is confirmed for 0.97.0
    endpoint: Option<()>,

    /// Iroh Gossip handle.
    // WIRE: replace with iroh_gossip::Gossip once API is confirmed for 0.97.0
    gossip: Option<()>,

    /// Reused connections keyed by hex-encoded NodeId (RULES.md P-04).
    // WIRE: replace value type with iroh::Connection once API is confirmed
    connections: HashMap<String, ()>,

    /// Topics currently subscribed (for idempotency guard — RULES.md P-05).
    subscribed_topics: std::collections::HashSet<String>,

    /// Sender half for surfacing network events to the core worker.
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
}

impl NetworkManager {
    /// Construct a `NetworkManager`. Call `initialize` before any network operation.
    pub fn new(event_tx: tokio::sync::mpsc::Sender<NetworkEvent>) -> Self {
        Self {
            endpoint: None,
            gossip: None,
            connections: HashMap::new(),
            subscribed_topics: std::collections::HashSet::new(),
            event_tx,
        }
    }

    /// Bind the Iroh endpoint and start listening for incoming connections.
    ///
    /// # Errors
    /// Returns an error if the endpoint fails to bind.
    pub async fn initialize(&mut self) -> Result<()> {
        // WIRE: iroh 0.97.0
        //   let endpoint = iroh::Endpoint::builder()
        //       .discovery(Box::new(iroh::discovery::ConcurrentDiscovery::from_services(vec![...])))
        //       .bind()
        //       .await?;
        //   self.endpoint = Some(endpoint);
        //   self.gossip = Some(iroh_gossip::Gossip::builder().spawn(endpoint.clone()).await?);
        tracing::warn!("NetworkManager::initialize — iroh 0.97.0 wiring pending");
        Ok(())
    }

    /// Returns the local node's hex-encoded NodeId.
    ///
    /// # Errors
    /// Returns an error if the endpoint has not been initialised.
    pub fn local_node_id(&self) -> Result<String> {
        if self.endpoint.is_none() {
            bail!("network not initialised — call initialize() first");
        }
        // WIRE: Ok(self.endpoint.as_ref().unwrap().node_id().to_string())
        Ok(String::from("00000000000000000000000000000000"))
    }

    /// Send an encrypted payload to a peer directly (Iroh unicast).
    ///
    /// Reuses an existing connection if one exists (RULES.md P-04).
    /// If the peer is unreachable the caller should queue the message (RULES.md A-06).
    ///
    /// # Errors
    /// Returns an error if the peer is unreachable after Pkarr discovery (RULES.md P-02, P-03).
    pub async fn send_direct(&mut self, target_node_id: &str, payload: Vec<u8>) -> Result<()> {
        if payload.len() > MAX_PAYLOAD_BYTES {
            bail!(
                "payload exceeds maximum size ({} > {})",
                payload.len(),
                MAX_PAYLOAD_BYTES
            );
        }
        direct::send(self, target_node_id, payload).await
    }

    /// Broadcast an encrypted payload to all subscribers of a gossip topic (Iroh Gossip).
    ///
    /// # Errors
    /// Returns an error if the group swarm broadcast fails (RULES.md P-02).
    pub async fn broadcast_group(&self, topic_id: &str, payload: Vec<u8>) -> Result<()> {
        if payload.len() > MAX_PAYLOAD_BYTES {
            bail!(
                "payload exceeds maximum size ({} > {})",
                payload.len(),
                MAX_PAYLOAD_BYTES
            );
        }
        group::broadcast(self, topic_id, payload).await
    }

    /// Join the gossip swarm for a group topic. No-op if already subscribed (RULES.md P-05).
    ///
    /// # Errors
    /// Returns an error if the subscription fails.
    pub async fn subscribe_group(&mut self, topic_id: &str) -> Result<()> {
        if self.subscribed_topics.contains(topic_id) {
            return Ok(()); // idempotent — RULES.md P-05
        }
        group::subscribe(self, topic_id).await?;
        self.subscribed_topics.insert(topic_id.to_owned());
        Ok(())
    }

    /// Emit a `NetworkEvent` to the core worker.
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    // Scaffold: remove allow when iroh event emission is wired.
    #[allow(dead_code)]
    pub(crate) async fn emit(&self, event: NetworkEvent) -> Result<()> {
        self.event_tx
            .send(event)
            .await
            .map_err(|_| anyhow::anyhow!("network event channel closed"))
    }
}
