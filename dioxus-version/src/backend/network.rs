use anyhow::Result;
use iroh::{Endpoint, RelayMode, SecretKey};

pub const ALPN: &[u8] = b"nodechat/1";

pub struct Network {
    pub endpoint: Endpoint,
    pub secret_key: SecretKey,
}

impl Network {
    pub async fn new(secret_key: SecretKey) -> Result<Self> {
        let endpoint = Endpoint::builder(iroh::endpoint::presets::N0)
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN.to_vec()])
            .relay_mode(RelayMode::Default)
            .bind()
            .await?;

        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), endpoint.online()).await;

        tracing::info!("network: endpoint bound, id={}", endpoint.id());

        Ok(Self { endpoint, secret_key })
    }

    pub fn endpoint_id(&self) -> String {
        self.endpoint.id().to_string()
    }

    pub fn addr(&self) -> iroh::EndpointAddr {
        self.endpoint.addr()
    }

    pub async fn close(self) -> Result<()> {
        self.endpoint.close().await;
        Ok(())
    }
}
