#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use iroh::{Endpoint, NodeAddr, RelayMode, SecretKey, protocol::Router};
use iroh_gossip::{net::Gossip, proto::TopicId};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

#[tokio::test]
// run test by using: 'cargo test __tests__::iroh_connection_bug_v2::test_iroh_connection_bug_v2 -- --exact --nocapture'
async fn test_iroh_connection_bug_v2() -> Result<()> {
    iroh_connection_bug_v2().await
}

#[tokio::test]
// run test by using: 'cargo test __tests__::iroh_connection_bug_v2_v3::print_ips -- --exact --nocapture'
async fn print_ips() -> Result<()> {
    let fake_endpoint = Endpoint::builder()
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .bind()
        .await?;
    let fake_node_addr = fake_endpoint.node_addr().await?;
    let socket_addresses: Vec<&SocketAddr> = fake_node_addr.direct_addresses().collect();
    println!("> my ips: {:?}", socket_addresses);
    fake_endpoint.close().await;
    Ok(())
}

// Need to try to connect the "proper way"
// Cannot seem to get GossipSender and GossipReciver working.
pub async fn iroh_connection_bug_v2() -> Result<()> {
    tracing_subscriber::fmt::init();
    let topic_id = TopicId::from_bytes(rand::random());
    let server_socket_addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 55), 52807);

    println!("> creating endpoints ...");
    let server_endpoint = Endpoint::builder()
        .secret_key(SecretKey::generate(rand::rngs::OsRng))
        .relay_mode(RelayMode::Default)
        .bind_addr_v4(server_socket_addr)
        .alpns(vec![iroh_gossip::ALPN.to_vec()])
        .bind()
        .await?;
    let user_endpoint = Endpoint::builder()
        .secret_key(SecretKey::generate(rand::rngs::OsRng))
        .relay_mode(RelayMode::Default)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .alpns(vec![iroh_gossip::ALPN.to_vec()])
        .bind()
        .await?;

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("> creating known server_node_addr ...");
    let _server_node_addr = server_endpoint.node_addr().await?; // doing it like so it does not work.

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("> server_node_addr:\n{:#?}", _server_node_addr);
    let does_it_have_the_correct_socket_addr = _server_node_addr
        .direct_addresses()
        .any(|socket| socket == &SocketAddr::from(server_socket_addr));
    assert!(does_it_have_the_correct_socket_addr);

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("> creating gossip instances ...");
    let user_gossip = Gossip::builder().spawn(user_endpoint.clone()).await?;
    let _server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;

    // tokio::time::sleep(Duration::from_millis(100)).await;
    // println!("> creating router instances ...");
    // let _user_router = Router::builder(user_endpoint.clone())
    //     .accept(iroh_gossip::ALPN, user_gossip.clone())
    //     .spawn()
    //     .await?;
    // let _server_router = Router::builder(server_endpoint.clone())
    //     .accept(iroh_gossip::ALPN, server_gossip.clone())
    //     .spawn()
    //     .await?;

    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("> trying to connect ...");
    let iroh_connection = user_endpoint
        .connect(_server_node_addr, iroh_gossip::ALPN)
        .await?;
    user_gossip.handle_connection(iroh_connection).await?;
    let mut user_gossip_topic = user_gossip.subscribe(topic_id, vec![])?;

    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> trying to join topic ...");
    user_gossip_topic.joined().await?;

    println!("> never reaches this print statement");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("> press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    println!("> closing ...");
    Ok(())
}
