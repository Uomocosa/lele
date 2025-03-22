#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{
    discovery::Discovery, endpoint::{self, ConnectionError}, NodeAddr, PublicKey, SecretKey
};
use iroh_gossip::{net::Gossip, proto::TopicId};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{
    consts::{SEED, SERVER_PORTS, TOPIC},
    iroh::gossip::subscribe_loop,
};
use crate::{
    iroh::{Connection, get_all_192_168_server_addresses, gossip::create_known_server},
    string::random_string,
    vector::chunk_vector,
};

#[tokio::test]
// run test by using: 'cargo test __tests__::lele_find_connection_v2::test_lele_find_connection_v2 -- --exact --nocapture'
async fn test_lele_find_connection_v2() -> Result<()> {
    lele_find_connection_v2().await
}

// Need to try to connect the "proper way"
// Cannot seem to get GossipSender and GossipReciver working.
pub async fn lele_find_connection_v2() -> Result<()> {
    let start = Instant::now();
    let server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
    println!("> server.public_key(): {:?}", server.public_key());

    println!("> creating user connection ...");
    let mut user = Connection::_empty();
    user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    user.set_name(&("user_".to_owned() + &random_string(5)));
    user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    user.set_topic(TOPIC);
    user.bind_endpoint().await?;
    user.spawn_gossip().await?;
    user.spawn_router().await?;

    println!("> adding {} nodes to the discovery service ...", user.peers.len());
    for peer in user.peers.clone() {
        user
            .discovery()?
            .add_node_info(peer);
    }

    println!("> subscribing ...");
    let mut gossip_topic = user.clone().subscribe()?;
    println!("> joining ...");
    // tracing_subscriber::fmt::init(); // With this you'll see the mess you encounter.
    gossip_topic.joined().await?; // Not only it takes A LOT to finish, it does not even works
    
    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    // println!("> press Ctrl+C to exit.");  tokio::signal::ctrl_c().await?;
    println!("> closing user ..."); user.close().await?;
    println!("> closing server ..."); server.close().await?;
    Ok(())
}
