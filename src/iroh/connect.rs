use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::Result;
use iroh::{NodeId, RelayUrl};
use iroh_gossip::{net::GossipTopic, proto::TopicId};

use crate::consts::{RELAY_VEC, SEED, TOPIC};

use super::{Server, User, get_server_addresses};

#[rustfmt::skip] // Not the best, but it works
// #[rustfmt::fn_single_line] // This should be set to EACH function in this block ;; AND it doesn't even work
pub async fn connect() -> Result<(User, Server, GossipTopic)> {
    let debug = false;
    let topic_id = TopicId::from_str(TOPIC)?;
    let n_server_per_loop = 25;
    let mut countdown_cycles = 5;

    let id = 0;
    let mut is_own_server_started = false;

    if debug { println!("> creating user ..."); }
    let mut user = User::random(topic_id).await?;
    user.set_debug(debug)?;
    let end_id = id + n_server_per_loop - 1;
    let id_vec: Vec<u64> = (id..end_id).collect();
    let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
    let mut server_addrs_map: HashMap<NodeId, u64> = HashMap::new();
    for (i, addr) in server_addrs.iter().enumerate() {
        server_addrs_map.insert(addr.node_id, i as u64);
    }
    let user_handle = user.connect_to_servers(server_addrs).await?;
    if debug { println!("> server_addrs_map:\n{:#?}", server_addrs_map); }

    let joined_timer = Instant::now();
    if debug { print!("> searching for servers "); io::stdout().flush()?; }
    while joined_timer.elapsed() <= Duration::from_secs(7) {
        if debug {
            print!(".");
            io::stdout().flush()?;
        }
        if user_handle.is_finished() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if debug { println!(); }

    // Server side
    let mut server = Server::empty();
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
            .map(|p| server_addrs_map.get(p).unwrap())
            .min()
            .unwrap();
        if debug { println!("> stating server Id({id})"); }
        let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;
        server = Server::create(*id, topic_id, relay_url, &SEED).await?;
        let server_clone = server.clone();
        let _server_handle = tokio::spawn(async move {
            let server_gtopic = server_clone.subscribe_and_join(vec![]).await?;
            if debug { println!("\n> server: connected!"); }
            anyhow::Ok(server_gtopic)
        });
        is_own_server_started = true;
        if debug { println!("> server is waiting for user to find it..."); }
    }
    // tokio::time::sleep(Duration::from_secs(10)).await;
    let handle_timer = Instant::now();
    if debug { print!("> searching for servers "); io::stdout().flush()?; }
    while handle_timer.elapsed() <= Duration::from_secs(7) {
        if debug { print!("."); io::stdout().flush()?; }
        if user_handle.is_finished() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if user_handle.is_finished() {
        let user_gtopic = user_handle.await??;
        if debug { println!(); }
        if debug { println!("> user.node_id(): {:?}", user.node_id()); }
        if debug { println!("> online_peers:\n{:?}", user.online_peers()?.keys()); }
        return Ok((user, server, user_gtopic))
    } else {
        return connect().await;
    }
}
