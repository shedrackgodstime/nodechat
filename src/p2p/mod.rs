use anyhow::{Result, anyhow};
use iroh::{Endpoint, protocol::Router, endpoint::presets, EndpointId, EndpointAddr, SecretKey, Watcher};
use iroh_gossip::{net::Gossip, api::{GossipSender, GossipReceiver}, proto::TopicId};
use pkarr::Client as PkarrClient;

use iroh::endpoint::Connection;
use crate::core::commands::AppEvent;
use tokio::sync::broadcast;

pub const ALPN_1_1: &[u8] = b"nodechat/1:1";

/// Main networking backend struct for NodeChat.
/// Manages Iroh endpoints, Gossip swarms, and Pkarr discovery.
pub struct NetworkManager {
    pub endpoint: Endpoint,
    pub gossip: Gossip,
    pub pkarr: PkarrClient,
    pub router: Router,
}

use iroh::protocol::AcceptError;

/// Handler for incoming 1:1 unicast messages.
#[derive(Debug, Clone)]
struct DirectProtocolHandler {
    tx_event: broadcast::Sender<AppEvent>,
}

impl iroh::protocol::ProtocolHandler for DirectProtocolHandler {
    fn accept(&self, connection: Connection) -> impl futures::Future<Output = std::result::Result<(), AcceptError>> + std::marker::Send {
        let tx = self.tx_event.clone();
        async move {
            let remote_id = connection.remote_id();
            let mut stream = match connection.accept_uni().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[P2P] Failed to accept uni stream: {}", e);
                    return Ok(());
                }
            };

            let mut buffer = Vec::new();
            if let Err(e) = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buffer).await {
                eprintln!("[P2P] Failed to read incoming stream: {}", e);
                return Ok(());
            }
            
            // Forward the raw ciphertext to the worker for decryption
            let _ = tx.send(AppEvent::InternalIncomingMessage { 
                sender: remote_id.to_string(), 
                ciphertext: buffer
            });
            Ok(())
        }
    }
}

use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt;

/// A shareable token containing everything needed to connect and start an E2EE session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTicket {
    pub addr: EndpointAddr,
    pub x25519_public: [u8; 32],
}

impl ChatTicket {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serialization failed")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("invalid ticket bytes: {}", e))
    }
}

impl fmt::Display for ChatTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = BASE32_NOPAD.encode(&self.to_bytes());
        write!(f, "{}", encoded.to_lowercase())
    }
}

impl FromStr for ChatTicket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = BASE32_NOPAD.decode(s.to_uppercase().as_bytes())
            .map_err(|e| anyhow!("base32 decode failed: {}", e))?;
        Self::from_bytes(&bytes)
    }
}

impl NetworkManager {
    /// Initialize a new P2P network stack with a persistent identity.
    pub async fn new(secret_key: SecretKey, tx_event: broadcast::Sender<AppEvent>) -> Result<Self> {
        // 1. Build and bind the Iroh Endpoint
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(secret_key)
            .alpns(vec![ALPN_1_1.to_vec(), iroh_gossip::ALPN.to_vec()])
            .bind()
            .await?;
        
        // 2. Initialize Gossip background state machine
        let gossip = Gossip::builder().spawn(endpoint.clone());
        
        // 3. Initialize Pkarr
        let pkarr = PkarrClient::builder().build()?;
        
        // 4. Build and spawn the Router
        let handler = DirectProtocolHandler { tx_event };
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .accept(ALPN_1_1, handler)
            .spawn();

        let node_id = endpoint.id();
        println!("[P2P] iroh endpoint bound. NodeID: {}", node_id);

        Ok(Self {
            endpoint,
            gossip,
            pkarr,
            router,
        })
    }

    /// Returns the public NodeID of this instance.
    pub fn node_id(&self) -> EndpointId {
        self.endpoint.id()
    }

    /// Generate a ChatTicket for this node.
    pub async fn create_ticket(&self, x25519_public: [u8; 32]) -> Result<ChatTicket> {
        let addr = self.endpoint.watch_addr().get();
        Ok(ChatTicket {
            addr,
            x25519_public,
        })
    }

    /// 1. Direct 1:1 Message (Iroh Unicast)
    pub async fn send_direct(&self, target: EndpointAddr, payload: Vec<u8>) -> Result<()> {
        let connection = self.endpoint.connect(target, ALPN_1_1).await?;
        let mut send_stream = connection.open_uni().await?;
        tokio::io::AsyncWriteExt::write_all(&mut send_stream, &payload).await?;
        send_stream.finish()?;
        Ok(())
    }

    /// Join a Gossip swarm for a newly invited/created group.
    pub async fn subscribe_group(
        &self, 
        topic_id: TopicId, 
        bootstrap_peers: Vec<EndpointId>
    ) -> Result<(GossipSender, GossipReceiver)> {
        let topic = self.gossip.subscribe(topic_id, bootstrap_peers).await
            .map_err(|e| anyhow!("Gossip subscription failed: {}", e))?;

        Ok(topic.split())
    }

    /// Shutdown the network stack gracefully.
    pub async fn shutdown(self) -> Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }
}
