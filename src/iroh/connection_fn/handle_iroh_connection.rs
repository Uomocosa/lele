use anyhow::{Result, anyhow};

use crate::iroh::Connection;

pub async fn handle_iroh_connection(
    connection: &mut Connection,
    iroh_connection: iroh::endpoint::Connection,
) -> Result<&mut Connection> {
    if connection.gossip.is_none() {
        return Err(anyhow!("handle_iroh_connection::GossipNotFound"));
    }
    connection
        .gossip
        .as_ref()
        .unwrap()
        .handle_connection(iroh_connection)
        .await?;
    Ok(connection)
}
