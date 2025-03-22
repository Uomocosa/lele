#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{discovery::static_provider::StaticProvider, protocol::Router, Endpoint, NodeId, SecretKey};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use rand::rngs;
use tokio::time::Instant;
use std::{net::{Ipv4Addr, SocketAddrV4}, time::Duration};

use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{generate_fake_node_addrs, get_all_192_168_server_addresses, create_server_endpoint}};


#[tokio::test]
// run test by using: 'cargo test __tests__::two::test -- --exact --nocapture'
async fn test() -> Result<()> {
    two().await
}

// Using StaticProvider DOES NOT WORK!
pub async fn two() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let topic_id = TopicId::from_bytes(rand::random());
    let addrs = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    assert_eq!(addrs.len(), 65536);

    let discovery = StaticProvider::from_node_info(addrs.clone());

    let server_endpoint = create_server_endpoint(SERVER_PORTS, &SEED).await?;
    let user_endpoint = Endpoint::builder()
        .secret_key(SecretKey::generate(rand::rngs::OsRng))
        .add_discovery(move |_| Some(discovery))
        .bind()
        .await?;

    let user_gossip = Gossip::builder().spawn(user_endpoint.clone()).await?;
    let server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;

    let _user_router = Router::builder(user_endpoint.clone())
        .accept(iroh_gossip::ALPN, user_gossip.clone())
        .spawn()
        .await?;
    let _server_router = Router::builder(server_endpoint.clone())
        .accept(iroh_gossip::ALPN, server_gossip.clone())
        .spawn()
        .await?;

    let server_node_addr = server_endpoint.node_addr().await?;
    println!("> server_node_addr: {:?}", server_node_addr);
    let server_node_id = server_endpoint.node_id();
    println!("> server_node_id: {:?}", server_node_id);

    // for addr in addrs.clone() {
    //     user_endpoint.add_node_addr(addr)?;
    // }

    let time_to_join = Instant::now();
    // spawn a task for the server part
    tokio::task::spawn(async move {
        let mut server_gossip_topic = server_gossip.subscribe(topic_id, vec![])?;
        println!("> server is waiting for peers...");
        server_gossip_topic.joined().await?;
        println!("> server joined in [{:?}]", time_to_join.elapsed());
        for i in 0.. {
            let message = format!("message from server {i}");
            server_gossip_topic.broadcast(message.as_bytes().to_vec().into()).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        anyhow::Ok(())
    });

    // join the topic on the user
    let node_ids: Vec<NodeId> = addrs.clone().iter().map(|a| a.node_id).collect();
    println!("> adding {} node_ids", node_ids.len());
    let mut user_gossip_topic = user_gossip.subscribe(topic_id, node_ids)?;
    user_gossip_topic.joined().await?;
    println!("> user joined!");
    let elapsed= start.elapsed();

    while let Some(event) = user_gossip_topic.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(message)) = event {
            let message = std::str::from_utf8(&message.content)?;
            println!("> user received message: {message}",);
            // if message == "message from server 3" { break; }
        } else {
            println!("> user gossip event: {event:?}");
        }
    }
    println!("> finished [{:?}]", elapsed);
    Ok(())
}