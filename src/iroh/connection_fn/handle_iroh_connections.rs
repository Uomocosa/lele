use anyhow::Result;
use iroh_gossip::net::{GossipReceiver, GossipSender};

use crate::iroh::Connection;

pub async fn handle_iroh_connections(
    mut connection: Connection,
    iroh_connections: Vec<iroh::endpoint::Connection>,
) -> Result<(GossipSender, GossipReceiver)> {
    for iroh_connection in iroh_connections {
        connection.handle_iroh_connection(iroh_connection).await?;
    }
    connection.subscribe().await
}
