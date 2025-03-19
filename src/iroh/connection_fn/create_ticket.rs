use anyhow::{Result, anyhow};

use crate::iroh::{Connection, gossip::Ticket};

pub async fn create_ticket(connection: &mut Connection) -> Result<Ticket> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("Connection::create_ticket::EndpointNotFound"));
    }
    if connection.topic.is_none() {
        return Err(anyhow!("Connection::create_ticket::TopicIdNotFound"));
    }
    let endpoint = connection.endpoint.clone().unwrap();
    let topic = connection.topic.unwrap();
    let me = endpoint.node_addr().await?;
    // let peers = connection.peers.iter().cloned().chain([me]).collect();
    Ok(Ticket {
        topic,
        peers: vec![me],
    })
}
