use std::str::FromStr;

use anyhow::{Result, anyhow};
use iroh_gossip::net::{GossipReceiver, GossipSender};

use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage, Ticket},
};

pub async fn join(
    connection: Connection,
    ticket: String,
) -> Result<(GossipSender, GossipReceiver)> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("join::EndpointNotFound"));
    }
    if connection.gossip.is_none() {
        return Err(anyhow!("join::GossipNotFound"));
    }
    if connection.router.is_none() {
        return Err(anyhow!("join::RouterNotFound"));
    }
    let endpoint = connection.endpoint.unwrap();
    let gossip = connection.gossip.unwrap();
    let _router = connection.router.unwrap();
    let name = connection.name.unwrap_or("???".to_string());

    let debug = connection.debug;
    let Ticket { topic, peers } = Ticket::from_str(&ticket)?;
    println!("> {name}: joining chat room for topic {topic}");
    let peers_ids = peers.iter().map(|p| p.node_id).collect();
    if peers.is_empty() {
        if debug {
            println!("> {name}: waiting for peers to join us...");
        }
    } else {
        if debug {
            println!("> {name}: trying to connect to {} peers...", peers.len());
        }
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for peer in peers.into_iter() {
            match endpoint.add_node_addr(peer) {
                Ok(_) => {}
                Err(_) => continue,
            };
        }
    };

    let (sender, receiver) = gossip.subscribe_and_join(topic, peers_ids).await?.split();
    if debug {
        println!("> {name}: connected!");
    }
    let message = Message::AboutMe {
        name: name.to_string(),
    };
    if let Ok(encoded_message) = SignedMessage::sign_and_encode(endpoint.secret_key(), &message) {
        if let Err(e) = sender.broadcast(encoded_message).await {
            eprintln!("{}", e);
        }
    };
    Ok((sender, receiver))
}
