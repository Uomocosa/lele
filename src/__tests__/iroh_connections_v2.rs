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
// run test by using: 'cargo test __tests__::iroh_connections_v2::test_iroh_connections_v2 -- --exact --nocapture'
async fn test_iroh_connections_v2() -> Result<()> {
    iroh_connections_v2().await
}

// Need to try to connect the "proper way"
// Cannot seem to get GossipSender and GossipReciver working.
pub async fn iroh_connections_v2() -> Result<()> {
    tracing_subscriber::fmt::init();
    let start = Instant::now();
    let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
    println!("> server.public_key(): {:?}", server.public_key());
    let mut _server_handle = None;
    if !server.is_empty() {
        _server_handle = Some(tokio::spawn(server.clone().subscribe_and_join()));
        println!(
            "> server started — ip: {}, port: {}",
            server.ipv4, server.port
        );
    } else {
        println!("> cannot start a server");
    }
    let server_ip = Ipv4Addr::new(192, 168, 1, 55);
    println!("> getting hard_coded_server_node_addr ...");
    let _hard_coded_server_node_addr = NodeAddr::from_parts(
        PublicKey::from_str("01c639f0cd9424bde4895a7ddeb219598b0a98a7a910855ec4decb1880207cb8")?,
        None,
        vec![SocketAddr::new(IpAddr::V4(server_ip), SERVER_PORTS[0])],
    );
    println!("> creating user connection ...");
    let mut user = Connection::_empty();
    let chunk_len = 255 * 51;
    // user.peers = _generate_fake_node_addrs(chunk_len);
    user.peers.extend(vec![_hard_coded_server_node_addr]);
    // user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    user.set_name(&("user_".to_owned() + &random_string(5)));
    user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    user.set_topic(TOPIC);
    user.bind_endpoint().await?;
    // user.spawn_gossip().await?;
    // user.spawn_router().await?;
    println!("> creating vector of connections ");
    let adrrs_chunks = chunk_vector(&user.peers, chunk_len);
    let mut succesfull_connections = Vec::new();
    let mut i = 0;
    loop {
        let mut resulting_connections = Vec::new();
        let chunk = &adrrs_chunks[i];
        println!("> checking {} node addresses", chunk.len());
        for node_addr in chunk.clone() {
            let endpoint = user.endpoint.clone().unwrap();
            resulting_connections.push(tokio::spawn(async move {
                endpoint
                    .connect(node_addr, iroh_gossip::net::GOSSIP_ALPN)
                    .await
            }));
        }
        println!("> waiting for one connection to finish ...");
        let res = loop {
            while !resulting_connections.iter().any(|res| res.is_finished()) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            let index = resulting_connections
                .iter()
                .position(|x| x.is_finished())
                .unwrap();
            // find(|x| x.is_finished());
            println!("> index: {:?}", index);
            let handle = resulting_connections.remove(index);
            match handle.await? {
                Ok(connection) => break Ok((connection, chunk[index].clone())),
                // Err(e) if e.is::<ConnectionError>()  => break Err(e),
                Err(e) if Some(&ConnectionError::TimedOut) == e.downcast_ref() => break Err(e),
                Err(_) => continue,
            }
        };
        println!("> res.is_ok(): {}", res.is_ok());
        if res.is_ok() {
            succesfull_connections.push(res?);
            break;
        }
        // tokio::time::sleep(Duration::from_millis(500)).await;
        if i == adrrs_chunks.len() - 1 {
            i = 0;
        } // loop over
    }
    // println!(
    //     "> the first connection found has ALPN: {:?}",
    //     succesfull_connections[0].0.alpn().unwrap()
    // );
    assert_eq!(
        succesfull_connections[0].0.alpn().unwrap(),
        iroh_gossip::ALPN
    );
    // println!("> first connection:\n{:#?}", succesfull_connections[0].0);
    let (iroh_connections, _node_addrs): (Vec<_>, Vec<_>) =
        succesfull_connections.iter().cloned().unzip();
    // user.peers.extend(node_addrs);
    user.debug = true;
    server.debug = true;
    // user.peers = node_addrs;
    user.peers = vec![];
    user.spawn_gossip().await?;
    user.spawn_router().await?;
    let endpoint = user.endpoint.clone().unwrap();
    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
    gossip
        .handle_connection(iroh_connections[0].clone())
        .await?;
    println!("> user: trying to accept a connection");
    endpoint.clone().accept().await;
    if !server.is_empty() {
        println!("> server: trying to accept a connection");
        server.endpoint.clone().unwrap().accept().await;
    }
    // println!("> gossip:\n{:#?}", gossip);
    // user.gossip = Some(gossip);
    // user.spawn_router().await?;
    // let mut user_clone = user.clone();
    // let (user_tx, user_rx) = user.subscribe_and_join().await?;
    // // let (user_tx, user_rx) = user
    // //     .clone()
    // //     .handle_iroh_connections(iroh_connections)
    // //     .await?;
    // tokio::spawn(subscribe_loop(user_rx));
    // user_clone._send(user_tx, "hello").await?;

    // if let Some(server_handle) = _server_handle {
    //     println!("> trying to get server_handle ...");
    //     let (server_tx, server_rx) = server_handle.await??;
    //     tokio::spawn(subscribe_loop(server_rx));
    //     server._send(server_tx, "HI!").await?;
    //     println!("> server_handle obtained!");
    // } else {
    //     println!("> no server was established");
    // }
    // user.clone().connect_to_peers().await?;

    println!("> finished [{:?}]", start.elapsed());
    // println!("> press Ctrl+C to exit.");
    // tokio::signal::ctrl_c().await?;
    println!("> closing ...");
    user.close().await?;
    if !server.is_empty() {
        server.close().await?;
    }
    Ok(())
}
