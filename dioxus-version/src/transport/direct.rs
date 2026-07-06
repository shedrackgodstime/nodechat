use crate::backend::network::Network;
use anyhow::Result;

pub async fn send_direct(
    _network: &Network,
    _target_id: &str,
    _ciphertext: &[u8],
) -> Result<()> {
    // TODO: connect to peer, open stream, write MessageFrame::Direct
    tracing::info!("transport::direct: send to {}", _target_id);
    Ok(())
}
