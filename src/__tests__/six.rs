#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{protocol::Router, Endpoint, NodeId};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use n0_future::io::Empty;
use tokio::time::Instant;
use std::{net::{Ipv4Addr, SocketAddrV4}, time::Duration};

use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{generate_fake_node_addrs, get_all_192_168_server_addresses, create_server_endpoint}};


#[tokio::test]
// run test by using: 'cargo test __tests__::six::test -- --exact --nocapture'
async fn test() -> Result<()> {
    six().await
}

#[derive(Debug, Clone)]
enum Server{
    Empty,
    Data {
        endpoint: Endpoint,
        // gossip: Gossip,
        router: Router,
    },
}

// Of course it does not work
pub async fn six() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let topic_id = TopicId::from_bytes(rand::random());
    print!("> generating ALL server addresses... ");
    let addrs = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    println!("finished in [{:?}]", start.elapsed());

    println!("> creating user ... ");
    let user_endpoint = Endpoint::builder().bind().await?;
    let user_gossip = Gossip::builder().spawn(user_endpoint.clone()).await?;
    let user_router = Router::builder(user_endpoint.clone())
        .accept(iroh_gossip::ALPN, user_gossip.clone())
        .spawn()
        .await?;

    let mut server =  Server::Empty;
    if let Ok(server_endpoint) = create_server_endpoint(SERVER_PORTS, &SEED).await {
        let server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;
        let server_router = Router::builder(server_endpoint.clone())
            .accept(iroh_gossip::ALPN, server_gossip.clone())
            .spawn()
            .await?;
        println!("> server started!");
    
        let server_node_addr = server_endpoint.node_addr().await?;
        println!("> server_node_addr:\n>\t{:?}", server_node_addr);
        let server_node_id = server_endpoint.node_id();
        println!("> server_node_id:\n>\t{:?}", server_node_id);

        server = Server::Data{ 
            endpoint: server_endpoint.clone(),
            router: server_router,
        };

        tokio::task::spawn(async move {
            let i = 1;
            loop {
                let mut server_gtopic = match server_gossip.subscribe(topic_id, vec![]) {
                    Ok(gtopic) => gtopic,
                    Err(e) => return e,
                };
                if let Err(e) = server_gtopic.joined().await { return e; }
                
                let message = format!("message from server {i}");
                if let Err(e) = server_gtopic.broadcast(message.as_bytes().to_vec().into()).await {
                    return e;
                };
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    match server {
        Server::Empty => println!("> no server was started"),
        Server::Data{..} => println!("> server started, waiting for peers to find it!"),
    }

    println!("> adding {:?} addresses to user_endpoint ...", addrs.len());
    for addr in addrs.clone() {
        user_endpoint.add_node_addr(addr)?;
    }

    // join the topic on the user
    let node_ids: Vec<NodeId> = addrs.clone().iter().map(|a| a.node_id).collect();
    let time_to_subscribe = Instant::now();
    print!("> subscribing to {} node_ids ... ", node_ids.len());
    let mut user_gossip_topic = user_gossip.subscribe(topic_id, node_ids)?;
    println!("finished in [{:?}]", time_to_subscribe.elapsed());
    print!("> user is trying to join any server ...");
    let time_to_join = Instant::now();
    user_gossip_topic.joined().await?;
    println!("finished in [{:?}]", time_to_join.elapsed());

    if let Some(event) = user_gossip_topic.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(message)) = event {
            let message = std::str::from_utf8(&message.content)?;
            println!("> user received message: {message}",);
        } else {
            println!("> user gossip event: {event:?}");
        }
    }
    println!("> finished [{:?}]", start.elapsed());

    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");  tokio::signal::ctrl_c().await?;
    println!("> closing user ...");
    user_endpoint.close().await;
    user_router.shutdown().await?;
    match server {
        Server::Empty => println!("> no server to close"),
        Server::Data { endpoint, router } => {
            println!("> closing server ...");
            endpoint.close().await;
            router.shutdown().await?;
        }
    }
    Ok(())
}