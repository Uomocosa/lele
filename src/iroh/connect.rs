use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use futures::{FutureExt, future::BoxFuture};
use iroh::{Endpoint, NodeAddr, NodeId, RelayUrl};
use iroh_gossip::{
    net::{Event, GossipEvent, GossipReceiver, GossipTopic},
    proto::TopicId,
};
use n0_future::TryStreamExt;
use tokio::task::JoinHandle;

use crate::consts::{RELAY_VEC, SEED, TOPIC};
const DEBUG: bool = true;

use super::{get_server_addresses, ConnectOptions, Connection, Server, User};

pub async fn connect() -> Result<(User, Server, GossipTopic)> {
    let options = ConnectOptions { debug: DEBUG, ..Default::default()};
    let topic_id = TopicId::from_str(TOPIC)?;
    let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
    Ok((connection.user, connection.server, connection.user_gossip_topic))
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







// Old code 
const _SEARCH_LOOP_MAX_DURATION: Duration = Duration::from_secs(15); //secs

#[derive(Debug, Clone)]
struct _ConnectArgs {
    server_id: u64,
    user: User,
    topic_id: TopicId,
}

#[rustfmt::skip]
pub async fn _connect_old() -> Result<(User, Server, GossipTopic)> {
    let topic_id = TopicId::from_str(TOPIC)?;
    let mut user = User::random_with_topic(topic_id).await?;
    let server_id = 0;
    user.set_debug(DEBUG)?;
    let args = _ConnectArgs { user, topic_id, server_id};
    _recursion(args).await
}


#[rustfmt::skip]
fn _recursion(args: _ConnectArgs) -> BoxFuture<'static, Result<(User, Server, GossipTopic)>> {
    async move {
        let (user, server, gossip_topic) = match _core(args.clone()).await {
            Ok((user, server, gossip_topic)) => (user, server, gossip_topic),
            Err(e ) => { 
                if DEBUG { eprintln!("> connect::_core exited with error: '{}'", e); }
                return _recursion(args).await;
            }
        };
        
        let random_number: u64 = rand::random();
        let random_wait_secs = random_number % 26; // sleep from 0 to 25 secs
        println!("> random_wait_secs: {random_wait_secs}");
        // There might be another user that lunched the software at the same time as us.
        let timer = Instant::now();
        let users_ids: Vec<NodeId>  = user.online_peers()?.keys().cloned().collect();
        let other_users_ids: Vec<&NodeId>  = users_ids.iter().filter(|&node_id| node_id != &server.node_id().unwrap()).collect();
        println!("> other_users_ids: {:?}", other_users_ids);
        while other_users_ids.is_empty() { 
            if timer.elapsed() < Duration::from_secs(random_wait_secs) { 
                tokio::time::sleep(Duration::from_millis(250)).await;
                continue; 
            }
            // user.close().await?; println!("> closing user ...");
            server.close().await?; println!("> closing server ...");
            return _recursion(args).await;
        }
        Ok((user, server, gossip_topic)) 
    }.boxed()
}

#[rustfmt::skip]
async fn _core(mut args: _ConnectArgs) -> Result<(User, Server, GossipTopic)> {
    let topic_id = args.topic_id; 
    // let secret_key = args.user.secret_key()?.unwrap();
    // let relay_url = args.user.relay_url().unwrap();
    // let name = args.user.name().unwrap();
    args.user = args.user.recreate().await?;

    let n_server_per_loop = 25;
    let mut countdown_cycles = 5;

    let id = args.server_id;
    let mut is_own_server_started = false;

    if DEBUG { println!("> creating user ..."); }
    args.user.set_debug(DEBUG)?;

    let end_id = id + n_server_per_loop - 1;
    let id_vec: Vec<u64> = (id..end_id).collect();
    let relay_vec: Vec<String> = RELAY_VEC.iter().map(|s| s.to_string()).collect();
    let server_addrs = get_server_addresses(&id_vec, &relay_vec, &SEED).await?;
    let mut server_addrs_map: HashMap<NodeId, u64> = HashMap::new();
    for (i, addr) in server_addrs.iter().enumerate() {
        server_addrs_map.insert(addr.node_id, i as u64);
    }
    let user_handle = args.user.connect_to_servers(server_addrs).await?;
    if DEBUG { println!("> server_addrs_map:\n{:#?}", server_addrs_map); }

    let joined_timer = Instant::now();
    if DEBUG { print!("> searching for servers "); io::stdout().flush()?; }
    while joined_timer.elapsed() <= _SEARCH_LOOP_MAX_DURATION/2 {
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
    while handle_timer.elapsed() <= _SEARCH_LOOP_MAX_DURATION/2 {
        if DEBUG { print!("."); io::stdout().flush()?; }
        if !user_handle.is_finished() || !server_handle.as_ref().unwrap().is_finished() { 
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }
        if DEBUG { println!("\n> waiting for user_handle"); }
        let user_gtopic = user_handle.await??;
        if DEBUG { println!("> waiting for server_handle"); }
        let server_gtopic = server_handle.unwrap().await??;
        let (_, receiver) = server_gtopic.split();
        let endpoint_clone = args.user.endpoint().unwrap().clone();
        tokio::spawn(async move { _server_loop(receiver, endpoint_clone).await });
        if DEBUG { println!(); }
        if DEBUG { println!("> user.node_id(): {:?}", args.user.node_id()); }
        if DEBUG { println!("> online_peers:\n{:?}", args.user.online_peers()?.keys()); }
        return Ok((args.user, server, user_gtopic))
    }
    if DEBUG { println!("\n> closing server: Id({:?})", server.id()); }
    server.close().await?; 
    tokio::time::sleep(Duration::from_millis(250)).await;
    Err(anyhow!("connect::_core::NoOtherIrohInstanceFound"))
}

#[rustfmt::skip]
async fn _server_loop(mut receiver: GossipReceiver, endpoint_clone: Endpoint) -> Result<()> {
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
