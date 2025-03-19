use anyhow::Result;

use crate::iroh::Connection;

pub async fn handle_iroh_connection(
    connection: &mut Connection,
    iroh_connection: iroh::endpoint::Connection,
) -> Result<&mut Connection> {
    connection
        .gossip
        .as_ref()
        .unwrap()
        .handle_connection(iroh_connection)
        .await?;
    Ok(connection)
}
