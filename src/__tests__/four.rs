#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::{protocol::Router, Endpoint, NodeAddr, NodeId};
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent},
    proto::TopicId,
};
use tokio::time::Instant;
use std::{net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};

use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{generate_fake_node_addrs, get_all_192_168_server_addresses, get_secret_key_from_ip, get_server_ip, create_server_endpoint}};


#[tokio::test]
// run test by using: 'cargo test __tests__::four::test -- --exact --nocapture'
async fn test() -> Result<()> {
    four().await
}

pub async fn four() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();
    let topic_id = TopicId::from_bytes(rand::random());

    let server_node_addrs = get_server_node_addrs(SERVER_PORTS, &SEED).await?;
    println!("> server_node_addrs: {:?}", server_node_addrs);

    println!("> starting user ... ");
    let user_endpoint = Endpoint::builder().bind().await?;
    let user_gossip = Gossip::builder().spawn(user_endpoint.clone()).await?;
    let _user_router = Router::builder(user_endpoint.clone())
        .accept(iroh_gossip::ALPN, user_gossip.clone())
        .spawn()
        .await?;

    println!("> try to connect to known servers ...");
    // join the topic on the user
    for node_addr in server_node_addrs.clone() { user_endpoint.add_node_addr(node_addr)?; }
    let node_ids: Vec<NodeId> = server_node_addrs.clone().iter().map(|a| a.node_id).collect();
    println!("> adding {} node_ids", node_ids.len());
    let mut user_gtopic_1 = user_gossip.subscribe(topic_id, node_ids)?;
    let handle = tokio::spawn(async move {user_gtopic_1.joined().await});
    let time = Instant::now();
    println!("> try to join server with the same ip ...");
    while time.elapsed() <= Duration::from_secs(5) {
        if handle.is_finished() {break;}
    }
    println!("> user joined? {}", handle.is_finished());

    println!("> starting server ... ");
    match create_server_endpoint(SERVER_PORTS, &SEED).await {
        Err(_) => println!("> server could not start ;;"),
        Ok(server_endpoint) => {
            let server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;
            let _server_router = Router::builder(server_endpoint.clone())
                .accept(iroh_gossip::ALPN, server_gossip.clone())
                .spawn()
                .await?;
        
            println!("> server started!");
            let server_node_addr = server_endpoint.node_addr().await?;
            println!("> server_node_addr: {:?}", server_node_addr);
            let server_node_id = server_endpoint.node_id();
            println!("> server_node_id: {:?}", server_node_id);
        
            // loop:
            tokio::task::spawn(async move {
                loop {
                    let mut server_gtopic =  match server_gossip.subscribe(topic_id, vec![]) {
                        Ok(gtopic) => gtopic,
                        Err(e) => return e,
                    };
                    // println!("> server is waiting for peers...");
                    if let Err(e) = server_gtopic.joined().await { return e; }
                    println!("> new peer joined!");
                    // println!("> server joined in [{:?}]", time_to_join.elapsed());
                    // println!("> server joined in [{:?}]", time_to_join.elapsed());
                    for i in 0.. {
                        let message = format!("message from server {i}");
                        if let Err(e) = server_gtopic.broadcast(message.as_bytes().to_vec().into()).await {
                            return e;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            });
        }
    }
    

    println!("> creating ALL node addresses ...");
    let addrs = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
    let time_to_join = Instant::now();
    println!("> try to connect to ANY servers ...");
    for addr in addrs.clone() {
        user_endpoint.add_node_addr(addr)?;
    }
    let node_ids: Vec<NodeId> = addrs.clone().iter().map(|a| a.node_id).collect();
    println!("> adding {} node_ids", node_ids.len());
    let mut user_gossip_topic = user_gossip.subscribe(topic_id, node_ids)?;
    user_gossip_topic.joined().await?;
    println!("> user joined in [{:?}]", time_to_join.elapsed());
    while let Some(event) = user_gossip_topic.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(message)) = event {
            let message = std::str::from_utf8(&message.content)?;
            println!("> user received message: {message}",);
            if message == "message from server 3" { break; }
        } else {
            println!("> user gossip event: {event:?}");
        }
    }

    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");  tokio::signal::ctrl_c().await?;
    
    println!("> closing user ..."); 
    user_endpoint.close().await; 
    _user_router.shutdown().await?;
    // println!("> closing server ...");
    // server_endpoint.close().await; 
    // _server_router.shutdown().await?;
    Ok(())
}


pub async fn get_server_node_addrs(ports: &[u16], seed: &[u8; 28]) -> Result<Vec<NodeAddr>> {
    let ipv4 = get_server_ip().await?;
    let ip = IpAddr::V4(ipv4);
    let secret_key = get_secret_key_from_ip(&ipv4, seed);
    let mut vec_addrs: Vec<NodeAddr> = Vec::new();
    for &port in ports {
        vec_addrs.push(NodeAddr::from_parts(
            secret_key.public(),
            None,
            vec![SocketAddr::new(ip, port)],
        ));
    }
    Ok(vec_addrs)
}