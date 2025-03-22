#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use iroh::{
    NodeAddr, PublicKey, SecretKey,
    endpoint::{self, ConnectionError},
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
// run test by using: 'cargo test __tests__::lele_find_connection::test_lele_find_connection -- --exact --nocapture'
async fn test_lele_find_connection() -> Result<()> {
    lele_find_connection().await
}

// Need to try to connect the "proper way"
// Cannot seem to get GossipSender and GossipReciver working.
pub async fn lele_find_connection() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
    if server.is_empty() { 
        println!("> creating private_server ...");
        server.set_name("private_server");
        server.set_topic(TOPIC);
        server.set_random_secret_key();
        server.bind_endpoint().await?;
        server.spawn_gossip().await?;
        server.spawn_router().await?;
    }
    println!("> server.public_key(): {:?}", server.public_key());

    println!("> creating user connection ...");
    let mut user = Connection::_empty();
    user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    user.set_name(&("user_".to_owned() + &random_string(5)));
    user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    user.set_topic(TOPIC);
    user.bind_endpoint().await?;

    // tokio::spawn(user.clone().reachable_peers()); // works!
    // println!("> searching for reachanble_peers ...");
    // let reachanble_peers = user.clone().reachable_peers().await?;
    // println!("> reachanble_peers:\n{:?}", reachanble_peers);

    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    println!("> closing user ..."); user.close().await?;
    println!("> closing server ..."); server.close().await?;
    Ok(())
}
