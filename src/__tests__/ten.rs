#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{protocol::Router, Endpoint, NodeAddr, NodeId};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use n0_future::io::Empty;
use tokio::time::Instant;
use std::{net::{Ipv4Addr, SocketAddrV4}, str::FromStr, time::Duration};

use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{create_server_endpoint, generate_fake_node_addrs, get_all_192_168_server_addresses, gossip}};


#[tokio::test]
// run test by using: 'cargo test __tests__::ten::test -- --exact --nocapture'
async fn test() -> Result<()> {
    ten().await
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
pub async fn ten() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let topic_id = TopicId::from_str(TOPIC)?;
    print!("> generating ALL server addresses... ");
    let addrs = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    println!("finished in [{:?}]", start.elapsed());

    let server = start_server(topic_id).await?;
    match server {
        Server::Empty => println!("> no server was started"),
        Server::Data{..} => println!("> server started, waiting for peers to find it!"),
    }

    let user = create_user_and_join(topic_id, addrs.clone()).await?;

    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");  tokio::signal::ctrl_c().await?;
    println!("> closing server ..."); server.close().await?;
    println!("> closing user ..."); user.close().await?;
    Ok(())
}


async fn start_server(topic_id: TopicId,) -> Result<Server> {
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
            gossip: server_gossip.clone(),
            router: server_router,
        };

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
    }
    Ok(server)
}


async fn create_user_and_join(topic_id: TopicId, addrs: Vec<NodeAddr>) -> Result<User> {
    println!("> creating user ... ");
    let user_endpoint = Endpoint::builder().bind().await?;
    let user_gossip = Gossip::builder().spawn(user_endpoint.clone()).await?;
    let user_router = Router::builder(user_endpoint.clone())
        .accept(iroh_gossip::ALPN, user_gossip.clone())
        .spawn()
        .await?;

    println!("> adding {:?} addresses to user_endpoint ...", addrs.len());
    for addr in addrs.clone() {
        user_endpoint.add_node_addr(addr)?;
    }

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
    Ok(User::Data{
        endpoint: user_endpoint, 
        gossip: user_gossip,
        router: user_router,
    })
}