use anyhow::{Result, anyhow};
use iroh_gossip::net::GossipTopic;

use crate::iroh::Connection;

pub fn subscribe(connection: Connection) -> Result<GossipTopic> {
    if connection.gossip.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::GossipNotFound"));
    }
    if connection.router.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::RouterNotFound"));
    }
    if connection.topic.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::TopicNotFound"));
    }
    // let _router = connection.router.as_ref().unwrap();
    let topic_id = connection.topic.unwrap();
    let name = connection.name.clone().unwrap_or("???".to_string());
    let debug = connection.debug;

    let gossip = connection.gossip.clone().unwrap();
    if debug {
        println!("> connection.peers: {:#?}", &connection.peers);
    }
    let peers_ids = connection.peers.iter().map(|p| p.node_id).collect();
    let gossip_topic = gossip.subscribe(topic_id, peers_ids)?;
    if debug {
        println!("> {name}: subscribed!");
        println!("> {name} joined? {}", gossip_topic.is_joined());
    }
    Ok(gossip_topic)
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[tokio::test]
    // run test by using: 'cargo test iroh::connection_fn::subscribe::tests::one -- --exact --nocapture'
    async fn one() -> Result<()> {
        tracing_subscriber::fmt::init();
        let mut server = Connection::random();
        server.finalize_connection().await?;

        let mut user = Connection::random();
        user.set_topic(&server.topic()?);
        user.finalize_connection().await?;

        let server_node_addr = server.node_addr().await?;
        user.peers = vec![server_node_addr.clone()];
        user.discovery()?.add_node_info(server_node_addr.clone());
        // user.endpoint.clone().unwrap().add_node_addr(server_node_addr.clone())?;

        let _ = subscribe(server)?;
        let mut user_gossip_topic = subscribe(user)?;
        user_gossip_topic.joined().await?;

        Ok(())
    }
}
