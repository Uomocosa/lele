#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{endpoint, protocol::Router, Endpoint, NodeAddr, NodeId, RelayUrl, SecretKey};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use n0_future::io::Empty;
use tokio::time::Instant;
use std::{net::{Ipv4Addr, SocketAddrV4}, str::FromStr, time::Duration};

use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{create_server_endpoint, generate_fake_node_addrs, get_all_192_168_server_addresses, get_secret_key_from_hex, gossip}};


#[tokio::test]
// run test by using: 'cargo test __tests__::twelve::test -- --exact --nocapture'
async fn test() -> Result<()> {
    twelve().await
}

#[derive(Debug, Clone)]
enum Server{
    Empty,
    Data {
        endpoint: Endpoint,
        gossip: Gossip,
        router: Router,
    },
}

#[derive(Debug, Clone)]
enum User{
    Empty,
    Data {
        endpoint: Endpoint,
        gossip: Gossip,
        router: Router,
    },
}

impl Server {
    pub async fn close(self) -> Result<()> {
        match self {
            Server::Empty => {},
            Server::Data { endpoint, router , ..} => {
                endpoint.close().await;
                router.shutdown().await?;
            }
        }
        Ok(())
    }
}

impl User {
    pub async fn close(self) -> Result<()> {
        match self {
            User::Empty => {},
            User::Data { endpoint, router , ..} => {
                endpoint.close().await;
                router.shutdown().await?;
            }
        }
        Ok(())
    }
}

// Of course it does not work
pub async fn twelve() -> Result<()> {
    println!("> file 'twelve.rs'");
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let topic_id = TopicId::from_str(TOPIC)?;
    let relay_url = RelayUrl::from_str("https://euw1-1.relay.iroh.network./")?;
    let server_secret_key = get_secret_key_from_hex("176120fa4c93f2d0621d12bdf4a957c19650456d388a4bd313a65c6c0f638e0d")?;
    let server_node_addr = NodeAddr::new(server_secret_key.public())
        .with_relay_url(relay_url);
    let addrs = vec![server_node_addr];

    println!("> constructing known server ... ");
    let server_endpoint = Endpoint::builder().secret_key(server_secret_key).bind().await?;
    let server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;
    let server_router = Router::builder(server_endpoint.clone()).accept(iroh_gossip::ALPN, server_gossip.clone()).spawn().await?;
    let server = Server::Data { endpoint: server_endpoint, gossip: server_gossip.clone(), router: server_router };
    tokio::task::spawn(async move {
        let mut i = 1;
            let mut server_gtopic = match server_gossip.subscribe(topic_id, vec![]) {
                Ok(gtopic) => gtopic,
                Err(e) => return e,
            };
            if let Err(e) = server_gtopic.joined().await { return e; }
        loop {
            let message = format!("message from server {i}");
            if let Err(e) = server_gtopic.broadcast(message.as_bytes().to_vec().into()).await {
                return e;
            };
            i += 1;
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });

    println!("> creating user ... ");
    let endpoint = Endpoint::builder().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
    let router = Router::builder(endpoint.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn()
        .await?;
    let user = User::Data { 
        endpoint: endpoint.clone(), 
        gossip: gossip.clone(), 
        router
    };

    println!("> adding {:?} addresses to user_endpoint ...", addrs.len());
    for addr in addrs.clone() {
        endpoint.add_node_addr(addr)?;
    }

    let node_ids: Vec<NodeId> = addrs.clone().iter().map(|a| a.node_id).collect();
    let time_to_subscribe = Instant::now();
    print!("> subscribing to {} node_ids ... ", node_ids.len());
    let mut user_gossip_topic = gossip.subscribe(topic_id, node_ids)?;
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
    println!("> closing server ..."); server.close().await?;
    println!("> closing user ..."); user.close().await?;
    Ok(())
}
