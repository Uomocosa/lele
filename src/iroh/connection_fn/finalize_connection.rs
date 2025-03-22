use anyhow::Result;

use crate::iroh::Connection;

pub async fn finalize_connection(connection: &mut Connection) -> Result<&mut Connection> {
    connection.bind_endpoint().await?;
    connection.spawn_gossip().await?;
    connection.spawn_router().await?;
    Ok(connection)
}