//! P2P transport layer — direct unicast via Iroh, group broadcast via Iroh Gossip.
//!
//! IMPORTANT: This module is scaffolded. The internal `iroh` and `iroh-gossip` API
//! calls must be verified against the pinned iroh 0.97.0 documentation before
//! filling in the bodies marked `// WIRE:`. The public interface and types are final.
//!
//! See `src/p2p/direct.rs` for 1:1 unicast and `src/p2p/group.rs` for gossip.

use anyhow::{bail, Result};
use iroh::protocol::Router;
use iroh_tickets::endpoint::EndpointTicket;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub mod direct;
pub mod group;
pub mod protocol;

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
#[derive(Clone)]
pub struct NetworkManager {
    inner: Arc<Mutex<NetworkManagerInner>>,
}

struct NetworkManagerInner {
    /// Iroh endpoint.
    endpoint: Option<iroh::Endpoint>,

    /// Router that keeps our ALPN handlers alive for the lifetime of the endpoint.
    router: Option<Arc<Router>>,

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
            inner: Arc::new(Mutex::new(NetworkManagerInner {
                endpoint: None,
                router: None,
                gossip: None,
                connections: Arc::new(Mutex::new(HashMap::new())),
                subscribed_topics: HashSet::new(),
                event_tx,
            })),
        }
    }

    /// Bind the Iroh endpoint and start listening for incoming connections.
    pub async fn initialize(&self, secret_key: Option<iroh::SecretKey>) -> Result<()> {
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
        
        let _ = endpoint.online().await;
        
        let gossip: iroh_gossip::net::Gossip = iroh_gossip::net::Gossip::builder().spawn(endpoint.clone());

        let event_tx = {
            let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            inner.event_tx.clone()
        };

        let connections_ptr = {
            let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            inner.connections.clone()
        };

        let router = Router::builder(endpoint.clone())
            .accept(DIRECT_ALPN, DirectProtocolHandler {
                event_tx: event_tx.clone(),
                connections: connections_ptr.clone(),
            })
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let mut inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.endpoint = Some(endpoint.clone());
        inner.router = Some(Arc::new(router));
        inner.gossip = Some(gossip);

        tracing::info!("NetworkManager initialized with Iroh 0.97.0");
        Ok(())
    }

    /// Returns the local node's hex-encoded NodeId.
    ///
    /// # Errors
    /// Returns an error if the endpoint has not been initialised.
    pub fn local_node_id(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.endpoint
            .as_ref()
            .map(|e| e.id().to_string())
            .ok_or_else(|| anyhow::anyhow!("network not initialised"))
    }

    /// Return this endpoint's shareable ticket string.
    ///
    /// The ticket packages the current endpoint address for manual peer discovery.
    pub fn endpoint_ticket(&self) -> Result<String> {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let endpoint = inner.endpoint.as_ref()
            .ok_or_else(|| anyhow::anyhow!("network not initialised"))?;
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
            bail!("payload too large");
        }
        direct::send(self, target_node_id, dial_hint, payload).await
    }

    /// Broadcast an encrypted payload to all subscribers of a gossip topic (Iroh Gossip).
    ///
    /// # Errors
    /// Returns an error if the group swarm broadcast fails (RULES.md P-02).
    pub async fn broadcast_group(&self, topic_id: &str, payload: Vec<u8>) -> Result<()> {
        if payload.len() > MAX_PAYLOAD_BYTES {
            bail!("payload too large");
        }
        let _gossip = {
            let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            inner.gossip.clone().ok_or_else(|| anyhow::anyhow!("gossip not initialised"))?
        };
        group::broadcast(self.clone(), topic_id, payload).await
    }

    /// Join the gossip swarm for a group topic. No-op if already subscribed (RULES.md P-05).
    ///
    /// # Errors
    /// Returns an error if the subscription fails.
    pub async fn subscribe_group(&self, topic_id: &str, bootstrap: Vec<String>) -> Result<()> {
        {
            let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            if inner.subscribed_topics.contains(topic_id) {
                return Ok(());
            }
        }
        
        let mut bootstrap_ids = Vec::new();
        for id_hex in bootstrap {
            if let Ok(id) = id_hex.parse::<iroh::EndpointId>() {
                bootstrap_ids.push(id);
            }
        }

        group::subscribe(self.clone(), topic_id, bootstrap_ids).await?;
        {
            let mut inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            inner.subscribed_topics.insert(topic_id.to_owned());
        }
        Ok(())
    }

    /// Emit a `NetworkEvent` to the core worker.
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub(crate) async fn emit(&self, event: NetworkEvent) -> Result<()> {
        let event_tx = {
            let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            inner.event_tx.clone()
        };
        event_tx.send(event).await
            .map_err(|_| anyhow::anyhow!("network event channel closed"))
    }

    /// Returns the number of currently active peer connections.
    pub fn connection_status(&self) -> NetworkStatusInfo {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let direct = inner.connections.lock().map(|map| map.len()).unwrap_or(0);
        // WIRE: filter by iroh::endpoint::Connection stats to distinguish direct vs relay
        NetworkStatusInfo { direct, relay: 0 }
    }

    /// Returns `true` if we currently have a cached direct connection for `target_node_id`.
    pub fn has_connection(&self, target_node_id: &str) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.connections
            .lock()
            .map(|map| map.contains_key(target_node_id))
            .unwrap_or(false)
    }

    /// Returns the currently connected peer node ids.
    pub fn active_connections(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.connections
            .lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove a cached connection for a peer after the conversation is deleted.
    pub fn remove_connection(&self, target_node_id: &str) {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Ok(mut connections) = inner.connections.lock() {
            connections.remove(target_node_id);
        }
    }

    /// Spawn a background reader for a newly dialed direct connection.
    pub(crate) fn spawn_direct_reader(&self, remote_id: String, connection: iroh::endpoint::Connection) {
        let manager = self.clone();
        tokio::spawn(async move {
            if let Err(err) = drive_direct_connection(manager, remote_id, connection, false).await {
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

#[async_trait::async_trait]
impl iroh::protocol::ProtocolHandler for DirectProtocolHandler {
    fn accept(&self, connection: iroh::endpoint::Connection) -> impl futures::Future<Output = Result<(), iroh::protocol::AcceptError>> + Send {
        let event_tx = self.event_tx.clone();
        let connections_ptr = self.connections.clone();
        Box::pin(async move {
            let remote_id = connection.remote_id().to_string();
            tracing::info!(peer = %remote_id, "Direct: inbound connection accepted");
            
            {
                let mut conns = connections_ptr.lock().unwrap();
                conns.insert(remote_id.clone(), connection.clone());
            }

            tokio::spawn(async move {
                if let Err(e) = drive_direct_connection_raw(event_tx, connections_ptr, remote_id, connection, false).await {
                    tracing::debug!("Direct connection driver ended: {:?}", e);
                }
            });
            Ok(())
        })
    }
}

async fn drive_direct_connection(
    manager: NetworkManager,
    remote_id: String,
    connection: iroh::endpoint::Connection,
    via_relay: bool,
) -> Result<()> {
    let event_tx = {
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        inner.event_tx.clone()
    };

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
        let inner = manager.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut conns = inner.connections.lock().unwrap_or_else(|p| p.into_inner());
        conns.remove(&remote_id);
    }

    let _ = event_tx.send(NetworkEvent::PeerDisconnected { node_id: remote_id }).await;

    Ok(())
}

async fn drive_direct_connection_raw(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,
    remote_id: String,
    connection: iroh::endpoint::Connection,
    via_relay: bool,
) -> Result<()> {
    let _ = event_tx.send(NetworkEvent::PeerConnected { node_id: remote_id.clone(), via_relay }).await;
    while let Ok(mut recv) = connection.accept_uni().await {
        if let Ok(payload) = recv.read_to_end(MAX_PAYLOAD_BYTES).await {
            let _ = event_tx.send(NetworkEvent::DirectMessage { from: remote_id.clone(), payload }).await;
        }
    }
    {
        let mut conns = connections.lock().unwrap();
        conns.remove(&remote_id);
    }
    let _ = event_tx.send(NetworkEvent::PeerDisconnected { node_id: remote_id }).await;
    Ok(())
}
