use anyhow::{Result, bail};
// use ed25519_dalek::Signature;
use iroh::{Endpoint, RelayMap, RelayMode, RelayUrl, SecretKey};
use iroh_gossip::proto::TopicId;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use iroh_gossip::net::{GOSSIP_ALPN, Gossip};

use crate::iroh::gossip::{
    Command, Message, SignedMessage, Ticket, fmt_relay_mode, input_loop, subscribe_loop,
};

pub async fn test_gossip(
    command: Command,
    secret_key: Option<SecretKey>,
    no_relay: bool,
    relay: Option<RelayUrl>,
    bind_port: u16,
    name: Option<String>,
) -> Result<()> {
    tracing_subscriber::fmt::init();
    let (topic, peers) = match &command {
        Command::Open { topic } => {
            let topic = topic.unwrap_or_else(|| TopicId::from_bytes(rand::random()));
            println!("> opening chat room for topic {topic}");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let Ticket { topic, peers } = Ticket::from_str(ticket)?;
            println!("> joining chat room for topic {topic}");
            (topic, peers)
        }
    };

    // parse or generate our secret key
    // let secret_key = secret_key.unwrap_or_default();
    let secret_key: SecretKey = match secret_key {
        None => SecretKey::generate(rand::rngs::OsRng),
        Some(key) => key,
    };
    println!("> our secret key: {:#?}", secret_key);

    // configure our relay map
    let relay_mode = match (no_relay, relay) {
        (false, None) => RelayMode::Default,
        (false, Some(url)) => RelayMode::Custom(RelayMap::from_url(url)),
        (true, None) => RelayMode::Disabled,
        (true, Some(_)) => bail!("You cannot set --no-relay and --relay at the same time"),
    };
    println!("> using relay servers: {}", fmt_relay_mode(&relay_mode));

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(relay_mode)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, bind_port))
        .bind()
        .await?;
    println!("> our node id: {}", endpoint.node_id());

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().await?;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { topic, peers }
    };
    println!("> ticket to join us: {ticket}");

    // setup router
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .spawn()
        .await?;

    // join the gossip topic by connecting to known peers, if any
    let peer_ids = peers.iter().map(|p| p.node_id).collect();
    if peers.is_empty() {
        println!("> waiting for peers to join us...");
    } else {
        println!("> trying to connect to {} peers...", peers.len());
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for peer in peers.into_iter() {
            endpoint.add_node_addr(peer)?;
        }
    };
    let (sender, receiver) = gossip.subscribe_and_join(topic, peer_ids).await?.split();
    println!("> connected!");

    // broadcast our name, if set
    if let Some(name) = name {
        let message = Message::AboutMe {
            name: name.to_string(),
        };
        let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
        sender.broadcast(encoded_message).await?;
    }

    // subscribe and print loop
    tokio::spawn(subscribe_loop(receiver));

    // spawn an input thread that reads stdin
    // not using tokio here because they recommend this for "technical reasons"
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    // broadcast each line we type
    println!("> type a message and hit enter to broadcast...");
    while let Some(text) = line_rx.recv().await {
        let message = Message::Message { text: text.clone() };
        let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
        sender.broadcast(encoded_message).await?;
        println!("> sent: {text}");
    }

    // shutdown
    router.shutdown().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Do not run this as tokio::test, it needs to be in the main.rs file
    // tokio::test is not meant to be used for printing on screen.
    #[tokio::test]
    async fn create_new_connection() {
        test_gossip(Command::Open { topic: None }, None, true, None, 0, None)
            .await
            .expect("Failed to test gossip protocol");
    }
}
