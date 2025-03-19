use anyhow::{Result, anyhow};
use iroh_gossip::net::{GossipReceiver, GossipSender};

use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage},
};

pub async fn connect_to_peers(connection: Connection) -> Result<(GossipSender, GossipReceiver)> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::EndpointNotFound"));
    }
    if connection.gossip.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::GossipNotFound"));
    }
    if connection.router.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::RouterNotFound"));
    }
    if connection.topic.is_none() {
        return Err(anyhow!("Connection::connect_to_peers::TopicNotFound"));
    }
    let endpoint = connection.endpoint.unwrap();
    let gossip = connection.gossip.as_ref().unwrap();
    let _router = connection.router.as_ref().unwrap();
    let topic_id = connection.topic.unwrap();
    let name = connection.name.unwrap_or("???".to_string());
    let debug = connection.debug;
    if debug {
        println!("> adding {} peers ...", connection.peers.len())
    };
    let peers_ids = connection.peers.iter().map(|p| p.node_id).collect();
    let (sender, receiver) = gossip
        .subscribe_and_join(topic_id, peers_ids)
        .await?
        .split();
    if debug {
        println!("> {name}: connected!");
    }
    let message = Message::AboutMe {
        name: name.to_string(),
    };
    let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
    sender.broadcast(encoded_message).await?;
    Ok((sender, receiver))
}
