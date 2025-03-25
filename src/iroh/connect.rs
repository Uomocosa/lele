use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::Result;
use futures::{FutureExt, future::BoxFuture};
use iroh::{Endpoint, NodeAddr, NodeId, RelayUrl};
use iroh_gossip::{
    net::{Event, GossipEvent, GossipReceiver, GossipTopic},
    proto::TopicId,
};
use n0_future::TryStreamExt;
use tokio::task::JoinHandle;
// use std::pin::Pin;
// use std::future::Future;

use crate::consts::{RELAY_VEC, SEED, TOPIC};

const DEBUG: bool = false;

use super::{Server, User, get_server_addresses};

#[rustfmt::skip]
pub fn connect() -> BoxFuture<'static, Result<(User, Server, GossipTopic)>> {
    async move {
        let (user, server, gossip_topic) = connect_init().await?;
        
        let random_number: u64 = rand::random();
        let radom_sleep_secs = random_number % 26; // sleep from 0 to 25 secs
        println!("> radom_sleep_secs: {radom_sleep_secs}");
        // There might be another user that lunched the software at the same time as us.
        let timer = Instant::now();
        while !user.is_any_other_user_online(RELAY_VEC, &SEED).await? {
            if timer.elapsed() < Duration::from_secs(radom_sleep_secs) { 
                tokio::time::sleep(Duration::from_millis(250)).await;
                continue; 
            }
            user.close().await?; println!("> closing user ...");
            server.close().await?; println!("> closing server ...");
            return connect().await;
        }
        Ok((user, server, gossip_topic)) 
    }.boxed()
}

#[rustfmt::skip]
fn connect_init() -> BoxFuture<'static, Result<(User, Server, GossipTopic)>> {
    async move {
        let topic_id = TopicId::from_str(TOPIC)?;
        let n_server_per_loop = 25;
        let mut countdown_cycles = 5;

        let id = 0;
        let mut is_own_server_started = false;

        if DEBUG { println!("> creating user ..."); }
        let mut user = User::random_with_topic(topic_id).await?;
        user.set_debug(DEBUG)?;
        let end_id = id + n_server_per_loop - 1;
        let id_vec: Vec<u64> = (id..end_id).collect();
        let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
        let mut server_addrs_map: HashMap<NodeId, u64> = HashMap::new();
        for (i, addr) in server_addrs.iter().enumerate() {
            server_addrs_map.insert(addr.node_id, i as u64);
        }
        let user_handle = user.connect_to_servers(server_addrs).await?;
        if DEBUG { println!("> server_addrs_map:\n{:#?}", server_addrs_map); }

        let joined_timer = Instant::now();
        if DEBUG { print!("> searching for servers "); io::stdout().flush()?; }
        while joined_timer.elapsed() <= Duration::from_secs(7) {
            if DEBUG {
                print!(".");
                io::stdout().flush()?;
            }
            if user_handle.is_finished() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        if DEBUG { println!(); }

        // Server side
        let mut server = Server::empty();
        let mut server_handle: Option<JoinHandle<Result<GossipTopic>>> = None;
        loop {
            let offline_peers = user.offline_peers()?;
            // println!("offline_peers:\n{:?}", offline_peers);
            if offline_peers.is_empty() {
                continue;
            }
            countdown_cycles -= 1;
            if countdown_cycles == 0 {
                break;
            }
            if is_own_server_started {
                continue;
            }
            let id = offline_peers
                .iter()
                .filter_map(|p| server_addrs_map.get(p))
                .min()
                .unwrap();
            if DEBUG { println!("> stating server Id({id})"); }
            let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;
            server = Server::create(*id, topic_id, relay_url, &SEED).await?;
            let server_clone = server.clone();
            server_handle = Some(tokio::spawn(async move {
                let server_gtopic = server_clone.subscribe_and_join(vec![]).await?;
                if DEBUG { println!("\n> server: connected!"); }
                anyhow::Ok(server_gtopic)
            }));
            is_own_server_started = true;
            if DEBUG { println!("> server is waiting for user to find it..."); }
        };
        // tokio::time::sleep(Duration::from_secs(10)).await;
        let handle_timer = Instant::now();
        if DEBUG { print!("> searching for servers "); io::stdout().flush()?; }
        while handle_timer.elapsed() <= Duration::from_secs(7) {
            if DEBUG { print!("."); io::stdout().flush()?; }
            if user_handle.is_finished() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        if user_handle.is_finished() {
            let user_gtopic = user_handle.await??;
            let server_gtopic = server_handle.unwrap().await??;
            let (_, receiver) = server_gtopic.split();
            let endpoint_clone = user.endpoint().unwrap().clone();
            tokio::spawn(async move { server_loop(receiver, endpoint_clone).await });
            if DEBUG { println!(); }
            if DEBUG { println!("> user.node_id(): {:?}", user.node_id()); }
            if DEBUG { println!("> online_peers:\n{:?}", user.online_peers()?.keys()); }
            Ok((user, server, user_gtopic))
        } else {
            println!("> closing server ...");
            server.close().await?;
            println!("> closing user ...");
            user.close().await?;
            connect_init().await
        }
    }.boxed()
}

#[rustfmt::skip]
async fn server_loop(mut receiver: GossipReceiver, endpoint_clone: Endpoint) -> Result<()> {
    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::NeighborUp(node_id)) = event {
            let relay_url = RelayUrl::from_str(RELAY_VEC[0]).expect("this should not fail");
            let node_addr = NodeAddr::new(node_id).with_relay_url(relay_url);
            match endpoint_clone.add_node_addr(node_addr)  {
                Ok(_) => if DEBUG { println!("> server: new neighbour {node_id}"); },
                Err(e) => if DEBUG { println!("> server: neighbour {node_id} gave error '{e}'"); },
            };
        }
    }
    Ok(())
}

#[tokio::test]
// run test by using: 'cargo test iroh::connect::test_connect -- --exact --nocapture'
async fn test_connect() -> Result<()> {
    let (user, server, gossip_topic) = connect().await?;
    println!("> user.online_peers(), {:#?}", user.online_peers());
    println!("> server.online_peers(), {:#?}", server.online_peers());
    println!(
        "> gossip_topic.is_joined(), {:#?}",
        gossip_topic.is_joined()
    );
    Ok(())
}
