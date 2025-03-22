#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use iroh::{
    NodeAddr, PublicKey, SecretKey,
    endpoint::{self, ConnectionError},
};
use iroh_gossip::{net::Gossip, proto::TopicId};
use std::{
    any::Any, net::{IpAddr, Ipv4Addr, SocketAddr}, str::FromStr, time::{Duration, Instant}
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
// run test by using: 'cargo test __tests__::iroh_connections_v3::test_iroh_connections_v3 -- --exact --nocapture'
async fn test_iroh_connections_v3() -> Result<()> {
    iroh_connections_v3().await
}

// Need to try to connect the "proper way"
// Cannot seem to get GossipSender and GossipReciver working.
pub async fn iroh_connections_v3() -> Result<()> {
    tracing_subscriber::fmt::init();
    let start = Instant::now();
    let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
    println!("> server.public_key(): {:?}", server.public_key());
    let server_ip = Ipv4Addr::new(192, 168, 1, 55);
    println!("> getting hard_coded_server_node_addr ...");
    let _hard_coded_server_node_addr = NodeAddr::from_parts(
        PublicKey::from_str("01c639f0cd9424bde4895a7ddeb219598b0a98a7a910855ec4decb1880207cb8")?,
        None,
        vec![SocketAddr::new(IpAddr::V4(server_ip), SERVER_PORTS[0])],
    );
    println!("> creating user connection ...");
    let mut user = Connection::_empty();
    // user.peers.extend(vec![_hard_coded_server_node_addr]);
    // user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    user.set_name(&("user_".to_owned() + &random_string(5)));
    user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    user.set_topic(TOPIC);
    user.bind_endpoint().await?;
    user.spawn_gossip().await?;
    user.spawn_router().await?;

    let iroh_connection = user.endpoint
        .clone()
        .unwrap()
        .connect(server.node_addr().await?, iroh_gossip::ALPN)
        .await?;
    println!("> iroh_connection.stable_id():\n{:?}", iroh_connection.stable_id());
    println!("> iroh_connection.remote_node_id():\n{:?}", iroh_connection.remote_node_id());
    println!("> iroh_connection.type_id():\n{:?}", iroh_connection.type_id());
    println!("> iroh_connection.stats():\n{:?}", iroh_connection.stats());
    println!("> iroh_connection.alpn():\n{:?}", iroh_connection.alpn());
    println!("> closing iroh_connection ...");
    drop(iroh_connection);
    // iroh_connection.handshake_data()

    tokio::time::sleep(Duration::from_millis(100)).await;
    // println!("> trying to connect to server ...");
    user.peers = vec![_hard_coded_server_node_addr];
    server.peers = vec![];
    let mut user_gtopic = user.clone().subscribe()?;
    let mut _server_gtopic = server.clone().subscribe()?;
    println!("> user_gtopic:\n{:#?}", user_gtopic);
    println!("> user_gtopic.is_joined(): {:?}", user_gtopic.is_joined());
    user_gtopic.joined().await?;

    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(100)).await; 
    println!("> press Ctrl+C to exit."); tokio::signal::ctrl_c().await?;
    println!("> closing ...");
    user.close().await?;
    if !server.is_empty() {
        server.close().await?;
    }
    Ok(())
}
