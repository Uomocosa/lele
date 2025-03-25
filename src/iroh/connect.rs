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
const DEBUG: bool = true;

use super::{Server, User, get_server_addresses};

#[derive(Debug, Clone)]
struct ConnectArgs {
    user: User,
    topic_id: TopicId,

}

#[rustfmt::skip]
pub async fn connect() -> Result<(User, Server, GossipTopic)> {
    let topic_id = TopicId::from_str(TOPIC)?;
    let mut user = User::random_with_topic(topic_id).await?;
    user.set_debug(DEBUG)?;
    let args = ConnectArgs { user, topic_id, };
    second_recursive_connect(args).await
}


#[rustfmt::skip]
fn second_recursive_connect(args: ConnectArgs) -> BoxFuture<'static, Result<(User, Server, GossipTopic)>> {
    async move {
        let (user, server, gossip_topic) = first_recursive_connect(args.clone()).await?;
        
        let random_number: u64 = rand::random();
        let random_wait_secs = random_number % 26; // sleep from 0 to 25 secs
        println!("> random_wait_secs: {random_wait_secs}");
        // There might be another user that lunched the software at the same time as us.
        let timer = Instant::now();
        while user.online_peers()?.len() < 2 { // 1 user is our own server
            if timer.elapsed() < Duration::from_secs(random_wait_secs) { 
                tokio::time::sleep(Duration::from_millis(250)).await;
                continue; 
            }
            // user.close().await?; println!("> closing user ...");
            server.close().await?; println!("> closing server ...");
            return second_recursive_connect(args).await;
        }
        Ok((user, server, gossip_topic)) 
    }.boxed()
}

#[rustfmt::skip]
fn first_recursive_connect(mut args: ConnectArgs) -> BoxFuture<'static, Result<(User, Server, GossipTopic)>> {
    async move {
        let topic_id = args.topic_id; 
        // let secret_key = match args.user.secret_key()? {
        //     None => return Err(anyhow!("connect::first_recursive_connect::NoSecretKeyFoundInArgsUser")),
        //     Some(secret_key) => secret_key,
        // };
        // let relay_url = match args.user.relay_url() {
        //     None => return Err(anyhow!("connect::first_recursive_connect::NoRelayUrlFoundInArgsUser")),
        //     Some(relay_url) => relay_url,
        // };
        // let name= match args.user.name() {
        //     None => return Err(anyhow!("connect::first_recursive_connect::NoNameFoundInArgsUser")),
        //     Some(name) => name,
        // };
        // let mut user = User::create(secret_key, topic_id, relay_url, &name).await?;

        let n_server_per_loop = 25;
        let mut countdown_cycles = 5;

        let id = 0;
        let mut is_own_server_started = false;

        if DEBUG { println!("> creating user ..."); }
        args.user.set_debug(DEBUG)?;
        let end_id = id + n_server_per_loop - 1;
        let id_vec: Vec<u64> = (id..end_id).collect();
        let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
        let mut server_addrs_map: HashMap<NodeId, u64> = HashMap::new();
        for (i, addr) in server_addrs.iter().enumerate() {
            server_addrs_map.insert(addr.node_id, i as u64);
        }
        let user_handle = args.user.connect_to_servers(server_addrs).await?;
        if DEBUG { println!("> server_addrs_map:\n{:#?}", server_addrs_map); }

        let joined_timer = Instant::now();
        if DEBUG { print!("> searching for servers "); io::stdout().flush()?; }
        while joined_timer.elapsed() <= Duration::from_secs(15) {
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
            let offline_peers = args.user.offline_peers()?;
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

        let handle_timer = Instant::now();
        if DEBUG { print!("> searching for servers "); io::stdout().flush()?; }
        while handle_timer.elapsed() <= Duration::from_secs(15) {
            if DEBUG { print!("."); io::stdout().flush()?; }
            if user_handle.is_finished() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        if user_handle.is_finished() || args.user.is_any_other_user_online(RELAY_VEC, &SEED).await? {
        // if user.is_any_other_user_online(RELAY_VEC, &SEED).await? {
            let user_gtopic = user_handle.await??;
            let server_gtopic = server_handle.unwrap().await??;
            let (_, receiver) = server_gtopic.split();
            let endpoint_clone = args.user.endpoint().unwrap().clone();
            tokio::spawn(async move { server_loop(receiver, endpoint_clone).await });
            if DEBUG { println!(); }
            if DEBUG { println!("> user.node_id(): {:?}", args.user.node_id()); }
            if DEBUG { println!("> online_peers:\n{:?}", args.user.online_peers()?.keys()); }
            Ok((args.user, server, user_gtopic))
        } else {
            println!("> closing server ...");
            server.close().await?;
            println!("> old user.node_id(): {:?}", args.user.node_id());
            // println!("> closing user ...");
            // user.close().await?;
            // args.user = user;
            first_recursive_connect(args).await
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
