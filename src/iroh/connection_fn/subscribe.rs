use anyhow::{Result, anyhow};
use iroh_gossip::net::{GossipReceiver, GossipSender};

use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage},
};

pub async fn subscribe(connection: Connection) -> Result<(GossipSender, GossipReceiver)> {
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

    let gossip = connection.gossip.as_ref().unwrap();
    if debug {
        println!("> connection.peers: {:#?}", &connection.peers);
    }
    let peers_ids = connection.peers.iter().map(|p| p.node_id).collect();
    let gossip_topic = gossip.subscribe(topic_id, peers_ids)?;
    if debug {
        println!("> {name}: subscribed!");
        println!("> {name} joined? {}", gossip_topic.is_joined());
    }
    let (_sender, _receiver) = gossip_topic.split();
    let message = Message::AboutMe { name };
    let encoded_message = SignedMessage::sign_and_encode(connection.secret_key()?, &message)?;
    _sender.broadcast(encoded_message).await?;
    Ok((_sender, _receiver))
}
