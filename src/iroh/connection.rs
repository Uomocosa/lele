use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use iroh::{Endpoint, NodeAddr, NodeId, RelayUrl};
use iroh_gossip::{
    net::{Event, GossipEvent, GossipReceiver, GossipTopic},
    proto::TopicId,
};
use n0_future::StreamExt;

use super::{get_server_addresses, GossipFuture, Server, ServerFuture, User};

#[derive(Debug)]
pub struct Connection {
    pub user: User,
    pub server_future: ServerFuture,
    pub user_gossip_topic: GossipTopic,
}

#[derive(Debug, Clone)]
pub struct ConnectOptions {
    pub starting_server_id: u64,
    pub debug: bool,
    pub search_duration: Duration,
    pub n_server_to_search: u64,
    pub known_addresses: Vec<NodeAddr>,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        ConnectOptions {
            starting_server_id: 0,
            debug: false,
            search_duration: Duration::from_secs(7),
            n_server_to_search: 1,
            known_addresses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct ConnectionArgs {
    topic_id: TopicId,
    relay_vec: Vec<String>,
    my_relay_url: RelayUrl,
    seed: [u8; 32],
    server_addrs_map: HashMap<NodeId, u64>,
    last_server_id: u64,
}

impl Connection {
    pub async fn create(topic_id: TopicId, relays_str: &[&str], seed: &[u8; 32]) -> Result<Self> {
        Connection::create_with_opts(topic_id, relays_str, seed, ConnectOptions::default()).await
    }

    pub async fn create_with_opts(
        topic_id: TopicId,
        relays_str: &[&str],
        seed: &[u8; 32],
        options: ConnectOptions,
    ) -> Result<Self> {
        // if options.debug { println!(""); }
        let relay_vec: Vec<String> = relays_str.iter().map(|s| s.to_string()).collect();
        let user = User::random_with_topic(topic_id).await?;
        let my_relay_url = user.relay_url().unwrap();
        let mut args = ConnectionArgs {
            topic_id,
            relay_vec,
            seed: *seed,
            server_addrs_map: HashMap::new(),
            my_relay_url,
            last_server_id: 0,
        };

        let mut user_handle: GossipFuture =
            create_user_join_handle(&user, &mut args, &options).await?;
        if options.debug {
            println!("> racing subscribing and timeout ...")
        };
        let connection = tokio::select! {
            finished_handle = &mut user_handle => {
                if options.debug { println!("> found other peer/s!") };
                let user_gossip_topic = finished_handle??;
                let starting_server_id: u64 = get_lowest_offline_server_id(&user, &mut args, &options)?;
                if options.debug { println!("> starting server Id({starting_server_id}) ...") };
                let user_clone = user.clone();
                let endpoint_clone = user.endpoint().unwrap().clone();
                let server_future_handle = tokio::spawn ( async move {
                    let (server, server_handle) = start_your_own_server(&user_clone, &mut args, &options).await?;
                    let server_gossip_topic = server_handle.await??;
                    if options.debug { println!("> server connected!") };
                    let (_, receiver) = server_gossip_topic.split();
                    tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
                    anyhow::Ok(server)
                });
                let server_future = ServerFuture{handle: server_future_handle};
                Connection {user, server_future, user_gossip_topic}
            }
            _ = tokio::time::sleep(options.search_duration) => {
                if options.debug { println!("> no other peer is found ;;") };
                let server_id: u64 = rand::random::<u64>() % options.n_server_to_search;
                if options.debug { println!("> server_id Id({})", server_id) };
                let (server, server_handle) = get_user_and_server_handle(
                    server_id, user.node_id().unwrap(), &args, &options
                ).await?;
                if options.debug { println!("> server started") };
                let user_gossip_topic = user_handle.await??;
                let endpoint_clone = user.endpoint().unwrap().clone();
                if options.debug { println!("> user connected!") };
                let server_future_handle = tokio::spawn ( async move {
                    let server_gossip_topic = server_handle.await??;
                    if options.debug { println!("> server connected!") };
                    let (_, receiver) = server_gossip_topic.split();
                    tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
                    anyhow::Ok(server)
                });
                let server_future = ServerFuture{handle: server_future_handle};
                Connection {user, server_future, user_gossip_topic}
            }
        };
        Ok(connection)
    }
}

#[rustfmt::skip]
async fn create_user_join_handle(user: &User, args: &mut ConnectionArgs, options: &ConnectOptions) -> Result<GossipFuture> {
    let n_known = options.known_addresses.len() as u64;
    if options.debug { println!("> known_addresses.len(): {:?}", n_known) };
    let addrs_to_search = match options.n_server_to_search < n_known {
        true => {
            if options.debug { println!("> searching only known_addresses ... ") };
            options.known_addresses.clone()
        },
        false => {
            let end_id = options.starting_server_id + options.n_server_to_search - n_known;
            let id_vec: Vec<u64> = (options.starting_server_id..end_id).collect();
            if options.debug { println!("> search server ids:\n{:#?}", id_vec) };
            args.last_server_id = end_id;
            let mut server_addrs = get_server_addresses(&id_vec, &args.relay_vec, &args.seed)?;
            for (i, addr) in id_vec.iter().zip(server_addrs.clone()) {
                args.server_addrs_map.insert(addr.node_id, *i);
            }
            server_addrs.extend(options.known_addresses.clone());
            server_addrs
        },
    };
    if options.debug { println!("> trying to connect to:\n{:#?}", addrs_to_search) };
    let user_handle: GossipFuture = user.connect_to_servers(addrs_to_search).await?;
    Ok(user_handle)
}

fn add_other_servers_to_user(
    user: &User,
    args: &mut ConnectionArgs,
    options: &ConnectOptions,
) -> Result<()> {
    let end_id = args.last_server_id + options.n_server_to_search;
    let id_vec: Vec<u64> = (args.last_server_id..end_id).collect();
    if options.debug {
        println!("> adding server ids:\n> {:?}", id_vec)
    };
    args.last_server_id = end_id;
    let server_addrs = get_server_addresses(&id_vec, &args.relay_vec, &args.seed)?;
    for (i, addr) in id_vec.iter().zip(server_addrs) {
        args.server_addrs_map.insert(addr.node_id, *i);
        user.add_node_addr(addr.clone())?;
    }
    Ok(())
}

fn get_lowest_offline_server_id(
    user: &User,
    args: &mut ConnectionArgs,
    options: &ConnectOptions,
) -> Result<u64> {
    let offline_peers = user.offline_peers()?;
    if offline_peers.is_empty() {
        if options.debug {
            println!("> get_lowest_offline_server_id: no offline peers found.");
        }
        add_other_servers_to_user(user, args, options)?;
        return get_lowest_offline_server_id(user, args, options);
    }
    let id = offline_peers
        .iter()
        .filter_map(|p| args.server_addrs_map.get(p))
        .min();
    if id.is_none() {
        if options.debug {
            println!("> get_lowest_offline_server_id: id.is_none().");
        }
        add_other_servers_to_user(user, args, options)?;
        return get_lowest_offline_server_id(user, args, options);
    }
    Ok(*id.unwrap())
}

async fn start_your_own_server(
    user: &User,
    args: &mut ConnectionArgs,
    options: &ConnectOptions,
) -> Result<(Server, GossipFuture)> {
    let server_id = get_lowest_offline_server_id(user, args, options)?;
    let (mut server, mut server_handle) =
        get_user_and_server_handle(server_id, user.node_id().unwrap(), args, options).await?;
    let mut last_server_id = server_id;
    let mut timer = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    while !server_handle.is_finished() {
        let server_id = get_lowest_offline_server_id(user, args, options)?;
        match (
            last_server_id == server_id,
            timer.elapsed() <= options.search_duration,
        ) {
            (true, true) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            _ => {
                if options.debug {
                    println!("> last Id({}) is no longer valid.", last_server_id)
                }
                if options.debug {
                    println!("> replacing it with new Id({}).", server_id)
                }
                // server.close().await?;
                // server_handle.abort();
                (server, server_handle) =
                    get_user_and_server_handle(server_id, user.node_id().unwrap(), args, options)
                        .await?;
                server_handle = tokio::spawn( async move {
                    let x = server_handle.await??;
                    Ok(x)
                });
                last_server_id = server_id;
                timer = Instant::now();
                // if options.debug { println!("> user.offline_peers: {:?}", user.offline_peers())}
                // if options.debug { println!("> user.online_peers: {:?}", user.online_peers())}
            }
        };
    }
    Ok((server, server_handle))
}

async fn get_user_and_server_handle(
    server_id: u64,
    user_node_id: NodeId,
    args: &ConnectionArgs,
    options: &ConnectOptions,
) -> Result<(Server, GossipFuture)> {
    let server = Server::create(
        server_id,
        args.topic_id,
        args.my_relay_url.clone(),
        &args.seed,
    ).await?;
    let debug = options.debug;
    let server_clone = server.clone();
    let node_addr = NodeAddr::new(user_node_id).with_relay_url(args.my_relay_url.clone());
    server_clone.add_node_addr(node_addr)?;
    let node_ids = vec![user_node_id];
    let server_handle: GossipFuture = tokio::spawn(async move {
        let server_gtopic = server_clone.subscribe_and_join(node_ids).await?;
        if debug {
            println!("\n> server_handle: connected!");
        }
        anyhow::Ok(server_gtopic)
    });
    Ok((server, server_handle))
}

#[rustfmt::skip]
async fn server_loop(mut receiver: GossipReceiver, endpoint_clone: Endpoint, debug: bool) -> Result<()> {
    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::NeighborUp(node_id)) = event {
            let relay_url = endpoint_clone.
                node_addr().
                await?.
                relay_url.
                unwrap();
            let node_addr = NodeAddr::new(node_id).with_relay_url(relay_url);
            match endpoint_clone.add_node_addr(node_addr)  {
                Ok(_) => if debug { println!("> server: new neighbour {node_id}"); },
                Err(e) => if debug { println!("> server: neighbour {node_id} gave error '{e}'"); },
            };
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{RELAY_VEC, SEED, TOPIC};
    use std::{str::FromStr, time::Instant};

    #[tokio::test]
    // run test by using: 'cargo test iroh::connection::tests::connection -- --exact --nocapture'
    async fn connection() -> Result<()> {
        tracing_subscriber::fmt::init();
        let start = Instant::now();
        let options = ConnectOptions {
            debug: true,
            ..Default::default()
        };
        let topic_id = TopicId::from_str(TOPIC)?;
        let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
        println!(
            "> user.online_peers(), {:#?}",
            connection.user.online_peers()
        );
        println!(
            "> gossip_topic.is_joined(), {:#?}",
            connection.user_gossip_topic.is_joined()
        );
        println!("> finished [{:?}]", start.elapsed());
        tokio::time::sleep(Duration::from_millis(250)).await;
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        println!("> closing connection ...");
        connection.user.close().await?;
        connection.server_future.close().await?;
        Ok(())
    }
}
