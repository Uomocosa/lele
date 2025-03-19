use anyhow::{Result, anyhow};
use iroh_gossip::net::GOSSIP_ALPN;

use crate::iroh::Connection;

pub async fn spawn_router(connection: &mut Connection) -> Result<&mut Connection> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("Connection::spawn_router::EndpointNotFound"));
    }
    if connection.gossip.is_none() {
        return Err(anyhow!("Connection::spawn_router::GossipNotFound"));
    }
    let endpoint = connection.endpoint.clone().unwrap();
    let gossip = connection.gossip.clone().unwrap();
    connection.router = Some(
        iroh::protocol::Router::builder(endpoint.clone())
            .accept(GOSSIP_ALPN, gossip.clone())
            .spawn()
            .await?,
    );
    Ok(connection)
}
