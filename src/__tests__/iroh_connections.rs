#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use iroh::{NodeAddr, PublicKey, SecretKey, endpoint::ConnectionError};
use iroh_gossip::net::Gossip;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use crate::consts::{SEED, SERVER_PORTS, TOPIC};
use crate::{
    iroh::{Connection, get_all_192_168_server_addresses, gossip::create_known_server},
    string::random_string,
    vector::chunk_vector,
};

#[tokio::test]
// run test by using: 'cargo test __tests__::iroh_connections::test_iroh_connections -- --exact --nocapture'
async fn test_iroh_connections() -> Result<()> {
    iroh_connections().await
}

// Creating a connection is fast, however I couldn't menage to connect
// my two PC via a simple connection
pub async fn iroh_connections() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
    println!("> server.public_key(): {:?}", server.public_key());
    server.peers = vec![];
    let mut _server_handle = None;
    if !server.is_empty() {
        _server_handle = Some(tokio::spawn(server.clone().connect_to_peers()));
        println!(
            "> server started — ip: {}, port: {}",
            server.ipv4, server.port
        );
    } else {
        println!("> cannot start a server");
    }
    let ipv4 = Ipv4Addr::new(192, 168, 1, 55);
    println!("> getting hard_coded_server_node_addr ...");
    let _hard_coded_server_node_addr = NodeAddr::from_parts(
        PublicKey::from_str("01c639f0cd9424bde4895a7ddeb219598b0a98a7a910855ec4decb1880207cb8")?,
        None,
        vec![SocketAddr::new(IpAddr::V4(ipv4), SERVER_PORTS[0])],
    );
    println!("> creating user connection ...");
    let mut user = Connection::_empty();
    let chunk_len = 255 * 51;
    // user.peers = _generate_fake_node_addrs(chunk_len);
    // user.peers.extend(vec![hard_coded_server_node_addr]);
    user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    user.set_name(&("user_".to_owned() + &random_string(5)));
    user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    user.set_topic(TOPIC);
    user.bind_endpoint().await?;
    user.spawn_gossip().await?;
    user.spawn_router().await?;
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
        let x = loop {
            while !resulting_connections.iter().any(|res| res.is_finished()) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            let index = resulting_connections
                .iter()
                .position(|x| x.is_finished())
                .unwrap();
            // find(|x| x.is_finished());
            println!("index: {:?}", index);
            let handle = resulting_connections.remove(index);
            match handle.await? {
                Ok(x) => break Ok(x),
                // Err(e) if e.is::<ConnectionError>()  => break Err(e),
                Err(e) if Some(&ConnectionError::TimedOut) == e.downcast_ref() => break Err(e),
                Err(_) => continue,
            }
        };
        println!("x.is_ok(): {}", x.is_ok());
        if x.is_ok() {
            succesfull_connections.push(x?);
            break;
        }
        // tokio::time::sleep(Duration::from_millis(500)).await;
        if i == adrrs_chunks.len() - 1 {
            i = 0;
        } // loop over
    }
    println!(
        "fond a connection with alpn: {:?}",
        succesfull_connections[0].alpn()
    );
    let iroh_connection = succesfull_connections.remove(0);
    user.gossip
        .as_ref()
        .unwrap()
        .handle_connection(iroh_connection)
        .await?;
    println!("> finished [{:?}]", start.elapsed());
    user.close().await?;
    if !server.is_empty() {
        server.close().await?;
    }
    Ok(())
}
