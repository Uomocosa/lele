use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage},
};

use anyhow::{Result, bail};
// use ed25519_dalek::Signature;
use iroh::{Endpoint, RelayMap, RelayMode, SecretKey};
use iroh_gossip::proto::TopicId;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use iroh_gossip::net::{GOSSIP_ALPN, Gossip};

use crate::iroh::gossip::{Command, Ticket, fmt_relay_mode};

pub async fn establish_connection(connection: &mut Connection) -> Result<&mut Connection> {
    let debug = connection.debug;
    let (topic, peers) = match &connection.command {
        Command::Open { topic } => {
            let topic = topic.unwrap_or_else(|| TopicId::from_bytes(rand::random()));
            if debug {
                println!("> opening chat room for topic {topic}");
            }
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let Ticket { topic, peers } = Ticket::from_str(ticket)?;
            if debug {
                println!("> joining chat room for topic {topic}");
            }
            (topic, peers)
        }
    };

    // parse or generate our secret key
    // let secret_key = secret_key.unwrap_or_default();
    // if let
    let secret_key = match &connection.secret_key {
        None => {
            let key = SecretKey::generate(rand::rngs::OsRng);
            connection.secret_key = Some(key.clone());
            key
        }
        Some(key) => key.clone(),
    };
    if debug {
        println!("> our secret key: {:#?}", secret_key);
    } // NEVER PRINT YOUR PRIVATE KEY (it will end up in logs)

    // configure our relay map
    let relay_mode = match (connection.no_relay, connection.relay.clone()) {
        (false, None) => RelayMode::Default,
        (false, Some(url)) => RelayMode::Custom(RelayMap::from_url(url)),
        (true, None) => RelayMode::Disabled,
        (true, Some(_)) => bail!("You cannot set --no-relay and --relay at the same time"),
    };
    if debug {
        println!("> using relay servers: {}", fmt_relay_mode(&relay_mode));
    }

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(relay_mode)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, connection.port))
        .bind()
        .await?;
    if debug {
        println!("> our node id: {}", endpoint.node_id());
    }

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().await?;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { topic, peers }
    };
    connection.ticket = Some(ticket.clone());
    if debug {
        println!("> ticket to join us: {ticket}");
    }

    // setup router
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .spawn()
        .await?;

    // join the gossip topic by connecting to known peers, if any
    let peers_ids = peers.iter().map(|p| p.node_id).collect();
    if peers.is_empty() {
        if debug {
            println!("> waiting for peers to join us...");
        }
    } else {
        if debug {
            println!("> trying to connect to {} peers...", peers.len());
        }
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for peer in peers.into_iter() {
            endpoint.add_node_addr(peer)?;
        }
    };
    let (sender, _receiver) = gossip.subscribe_and_join(topic, peers_ids).await?.split();
    if debug {
        println!("> connected!");
    }

    // broadcast our name, if set
    if let Some(name) = &connection.name {
        let message = Message::AboutMe {
            name: name.to_string(),
        };
        let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
        sender.broadcast(encoded_message).await?;
    }

    connection.endpoint = Some(endpoint.clone());
    connection.router = Some(router);
    // connection.sender = Some(sender);
    // connection.receiver = Some(receiver);
    Ok(connection)
}
