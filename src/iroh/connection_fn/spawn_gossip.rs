use anyhow::{Result, anyhow};
use iroh_gossip::net::Gossip;

use crate::iroh::Connection;

pub async fn spawn_gossip(connection: &mut Connection) -> Result<&mut Connection> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("Connection::spawn_gossip::EndpointNotFound"));
    }
    let endpoint = connection.endpoint.clone().unwrap();
    connection.gossip = Some(Gossip::builder().spawn(endpoint.clone()).await?);
    Ok(connection)
}
