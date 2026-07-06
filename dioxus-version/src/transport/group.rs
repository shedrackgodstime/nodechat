use anyhow::Result;

pub async fn subscribe(_topic: &[u8]) -> Result<()> {
    // TODO: iroh-gossip subscribe
    tracing::info!("transport::group: subscribe to topic");
    Ok(())
}

pub async fn broadcast(_topic: &[u8], _data: &[u8]) -> Result<()> {
    // TODO: iroh-gossip broadcast
    tracing::info!("transport::group: broadcast on topic");
    Ok(())
}

pub async fn leave(_topic: &[u8]) -> Result<()> {
    // TODO: iroh-gossip leave
    tracing::info!("transport::group: leave topic");
    Ok(())
}
