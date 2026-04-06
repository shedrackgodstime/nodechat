//! P2P transport layer — direct unicast via Iroh, group broadcast via Iroh Gossip.
//!
//! IMPORTANT: This module is scaffolded. The internal `iroh` and `iroh-gossip` API
//! calls must be verified against the pinned iroh 0.97.0 documentation before
//! filling in the bodies marked `// WIRE:`. The public interface and types are final.
//!
//! See `src/p2p/direct.rs` for 1:1 unicast and `src/p2p/group.rs` for gossip.

use anyhow::{bail, Result};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh_tickets::endpoint::EndpointTicket;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub mod direct;
pub mod group;

/// Maximum bytes allowed in a single direct message payload (plaintext + overhead).
const MAX_PAYLOAD_BYTES: usize = 64 * 1024; // 64 KiB
/// ALPN used for direct NodeChat connections.
pub const DIRECT_ALPN: &[u8] = b"nodechat/direct/1";

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
    /// Iroh endpoint.
    endpoint: Option<iroh::Endpoint>,

    /// Router that keeps our ALPN handlers alive for the lifetime of the endpoint.
    router: Option<Router>,

    /// Iroh Gossip handle.
    gossip: Option<iroh_gossip::net::Gossip>,

    /// Reused connections keyed by hex-encoded NodeId (RULES.md P-04).
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,

    /// Topics currently subscribed (for idempotency guard — RULES.md P-05).
    subscribed_topics: HashSet<String>,

    /// Sender half for surfacing network events to the core worker.
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
}

impl NetworkManager {
    /// Construct a `NetworkManager`. Call `initialize` before any network operation.
    pub fn new(event_tx: tokio::sync::mpsc::Sender<NetworkEvent>) -> Self {
        Self {
            endpoint: None,
            router: None,
            gossip: None,
            connections: Arc::new(Mutex::new(HashMap::new())),
            subscribed_topics: HashSet::new(),
            event_tx,
        }
    }

    /// Bind the Iroh endpoint and start listening for incoming connections.
    ///
    /// # Errors
    /// Returns an error if the endpoint fails to bind.
    pub async fn initialize(&mut self, secret_key: Option<iroh::SecretKey>) -> Result<()> {
        let transport_config = iroh::endpoint::QuicTransportConfig::builder()
            .max_idle_timeout(Some(std::time::Duration::from_secs(15).try_into().unwrap()))
            .keep_alive_interval(std::time::Duration::from_secs(10))
            .build();

        let mut builder = iroh::Endpoint::builder(iroh::endpoint::presets::N0)
            .transport_config(transport_config);
        if let Some(s) = secret_key {
            builder = builder.secret_key(s);
        }
        let endpoint = builder.bind().await?;
        
        // Wait for the endpoint to be fully registered with relays and discovery
        // services (iroh 0.97.0). This is critical for peer-to-peer discovery.
        endpoint.online().await;

        // Initialize gossip first so we can mount it on the router
        let gossip: iroh_gossip::net::Gossip = iroh_gossip::net::Gossip::builder().spawn(endpoint.clone());

        let router = Router::builder(endpoint.clone())
            .accept(DIRECT_ALPN, DirectProtocolHandler {
                event_tx: self.event_tx.clone(),
                connections: self.connections.clone(),
            })
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let node_id = endpoint.id();
        self.endpoint = Some(endpoint.clone());
        self.router = Some(router);
        self.gossip = Some(gossip);

        tracing::info!(
            node_id = node_id.to_string(),
            "NetworkManager initialized with Iroh 0.97.0"
        );
        Ok(())
    }

    /// Returns the local node's hex-encoded NodeId.
    ///
    /// # Errors
    /// Returns an error if the endpoint has not been initialised.
    pub fn local_node_id(&self) -> Result<String> {
        self.endpoint
            .as_ref()
            .map(|e| e.id().to_string())
            .ok_or_else(|| anyhow::anyhow!("network not initialised — call initialize() first"))
    }

    /// Return this endpoint's shareable ticket string.
    ///
    /// The ticket packages the current endpoint address for manual peer discovery.
    pub fn endpoint_ticket(&self) -> Result<String> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("network not initialised — call initialize() first"))?;
        Ok(EndpointTicket::new(endpoint.addr()).to_string())
    }

    /// Send an encrypted payload to a peer directly (Iroh unicast).
    ///
    /// Reuses an existing connection if one exists (RULES.md P-04).
    /// If the peer is unreachable the caller should queue the message (RULES.md A-06).
    ///
    /// # Errors
    /// Returns an error if the peer is unreachable after Pkarr discovery (RULES.md P-02, P-03).
    pub async fn send_direct(
        &self,
        target_node_id: &str,
        dial_hint: Option<&str>,
        payload: Vec<u8>,
    ) -> Result<()> {
        if payload.len() > MAX_PAYLOAD_BYTES {
            bail!(
                "payload exceeds maximum size ({} > {})",
                payload.len(),
                MAX_PAYLOAD_BYTES
            );
        }
        direct::send(self, target_node_id, dial_hint, payload).await
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
    pub async fn subscribe_group(&mut self, topic_id: &str, bootstrap: Vec<String>) -> Result<()> {
        if self.subscribed_topics.contains(topic_id) {
            return Ok(()); // idempotent — RULES.md P-05
        }
        
        // Parse hex node IDs into binary IDs for iroh-gossip
        let mut bootstrap_ids = Vec::new();
        for id_hex in bootstrap {
            if let Ok(id) = id_hex.parse::<iroh::EndpointId>() {
                bootstrap_ids.push(id);
            }
        }

        group::subscribe(self, topic_id, bootstrap_ids).await?;
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

    /// Returns the number of currently active peer connections.
    pub fn connection_status(&self) -> NetworkStatusInfo {
        let direct = self.connections.lock().map(|map| map.len()).unwrap_or(0);
        // WIRE: filter by iroh::endpoint::Connection stats to distinguish direct vs relay
        NetworkStatusInfo { direct, relay: 0 }
    }

    /// Returns `true` if we currently have a cached direct connection for `target_node_id`.
    pub fn has_connection(&self, target_node_id: &str) -> bool {
        self.connections
            .lock()
            .map(|map| map.contains_key(target_node_id))
            .unwrap_or(false)
    }

    /// Returns the currently connected peer node ids.
    pub fn active_connections(&self) -> Vec<String> {
        self.connections
            .lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove a cached connection for a peer after the conversation is deleted.
    pub fn remove_connection(&self, target_node_id: &str) {
        if let Ok(mut connections) = self.connections.lock() {
            connections.remove(target_node_id);
        }
    }

    /// Spawn a background reader for a newly dialed direct connection.
    ///
    /// This is needed because direct replies can arrive on our outbound
    /// connection, not only on inbound accepts.
    pub(crate) fn spawn_direct_reader(&self, remote_id: String, connection: iroh::endpoint::Connection) {
        let event_tx = self.event_tx.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            if let Err(err) = drive_direct_connection(event_tx, connections, remote_id, connection, false).await {
                tracing::debug!("direct reader task ended: {:?}", err);
            }
        });
    }
}

pub struct NetworkStatusInfo {
    pub direct: usize,
    pub relay:  usize,
}

#[derive(Clone, Debug)]
struct DirectProtocolHandler {
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,
}

impl ProtocolHandler for DirectProtocolHandler {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), AcceptError> {
        let remote_id = connection.remote_id().to_string();
        tracing::info!(peer = %remote_id, "direct protocol accepted — spawning driver");
        
        let event_tx = self.event_tx.clone();
        let connections = self.connections.clone();
        
        tokio::spawn(async move {
            if let Err(e) = drive_direct_connection(
                event_tx,
                connections,
                remote_id,
                connection,
                true,
            )
            .await 
            {
                tracing::debug!("direct driver task ended: {:?}", e);
            }
        });

        Ok(())
    }
}

async fn drive_direct_connection(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,
    remote_id: String,
    connection: iroh::endpoint::Connection,
    via_relay: bool,
) -> Result<(), AcceptError> {
    {
        let mut connections = connections.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        connections.insert(remote_id.clone(), connection.clone());
    }

    let _ = event_tx
        .send(NetworkEvent::PeerConnected {
            node_id: remote_id.clone(),
            via_relay,
        })
        .await;

    while let Ok(mut recv) = connection.accept_uni().await {
        if let Ok(payload) = recv.read_to_end(MAX_PAYLOAD_BYTES).await {
            let _ = event_tx
                .send(NetworkEvent::DirectMessage {
                    from: remote_id.clone(),
                    payload,
                })
                .await;
        }
    }

    {
        let mut connections = connections.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        connections.remove(&remote_id);
    }

    let _ = event_tx.send(NetworkEvent::PeerDisconnected { node_id: remote_id }).await;

    Ok(())
}
