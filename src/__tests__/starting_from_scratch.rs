#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{protocol::Router, Endpoint};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use std::time::Duration;

use crate::iroh::generate_fake_node_addrs;

#[tokio::test]
// run test by using: 'cargo test __tests__::starting_from_scratch::test_starting_from_scratch -- --exact --nocapture'
async fn test_starting_from_scratch() -> Result<()> {
    starting_from_scratch().await
}

pub async fn starting_from_scratch() -> Result<()> {
    tracing_subscriber::fmt::init();
    let topic_id = TopicId::from_bytes(rand::random());

    let server_endpoint = Endpoint::builder().bind().await?;
    let user_endpoint = Endpoint::builder().bind().await?;

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
    let server_node_id = server_endpoint.node_id();

    // this wouldn't be needed if using discoversy
    user_endpoint.add_node_addr(server_node_addr)?;

    // both peers need to join the topic!
    // one of them needs to add the addresses of the other as bootstrap.
    // in this example, the user knows the server's address, while the server joins without bootstrap
    // nodes and thus only waits for others to connect initially.

    // spawn a task for the server part
    tokio::task::spawn(async move {
        let mut topic = server_gossip.subscribe(topic_id, vec![])?;
        println!("> server is waiting for peers...");
        topic.joined().await?;
        println!("> server joined! ...");
        for i in 0.. {
            let message = format!("message from server {i}");
            topic.broadcast(message.as_bytes().to_vec().into()).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        anyhow::Ok(())
    });

    // join the topic on the user
    let mut user_gossip_topic = user_gossip.subscribe(topic_id, vec![server_node_id])?;
    user_gossip_topic.joined().await?;
    println!("> user joined!");

    while let Some(event) = user_gossip_topic.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(message)) = event {
            let message = std::str::from_utf8(&message.content)?;
            println!("> user received message: {message}",);
            if message == "message from server 3" { break; }
        } else {
            println!("> user gossip event: {event:?}");
        }
    }
    Ok(())
}