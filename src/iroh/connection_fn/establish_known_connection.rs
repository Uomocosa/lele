use std::net::{Ipv4Addr, SocketAddrV4};

use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage, Ticket, fmt_relay_mode},
};
use anyhow::{Result, bail};
use iroh::{Endpoint, RelayMap, RelayMode};
use iroh_gossip::net::{GOSSIP_ALPN, Gossip};

pub async fn establish_known_connection(connection: Connection, ticket: Ticket) -> Result<()> {
    let debug = connection.debug;
    let name = connection.name.unwrap_or("???".to_string());
    assert!(connection.secret_key.is_some());
    let secret_key = connection.secret_key.clone().unwrap();
    if debug {
        println!("> {name}: secret_key.public(): {:#?}", secret_key.public());
    }

    let (topic, peers) = (ticket.topic, ticket.peers);
    if debug {
        println!("> {name}: topic: {:#?}", topic);
    }
    if debug {
        println!("> {name}: peers:\n{:#?}", peers);
    }

    // configure our relay map
    let relay_mode = match (connection.no_relay, connection.relay.clone()) {
        (false, None) => RelayMode::Default,
        (false, Some(url)) => RelayMode::Custom(RelayMap::from_url(url)),
        (true, None) => RelayMode::Disabled,
        (true, Some(_)) => bail!("You cannot set --no-relay and --relay at the same time"),
    };
    if debug {
        println!(
            "> {name}: using relay servers: {}",
            fmt_relay_mode(&relay_mode)
        );
    }

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .relay_mode(relay_mode)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, connection.port))
        .bind()
        .await?;
    if debug {
        println!("> {name}: our node id: {}", endpoint.node_id());
    }

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().await?;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { topic, peers }
    };
    if debug {
        println!("> {name}: ticket to join us: {ticket}");
    }

    // setup router
    let _router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .spawn()
        .await?;

    // join the gossip topic by connecting to known peers, if any
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
            endpoint.add_node_addr(peer)?;
        }
    };
    let (sender, _receiver) = gossip.subscribe_and_join(topic, peers_ids).await?.split();
    if debug {
        println!("> {name}: connected!");
    }

    // broadcast our name, if set
    let message = Message::AboutMe { name };
    let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
    sender.broadcast(encoded_message).await?;

    Ok(())
}
