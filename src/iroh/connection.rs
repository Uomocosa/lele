use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use iroh::{Endpoint, NodeAddr, NodeId, RelayUrl};
use iroh_gossip::{net::{Event, GossipEvent, GossipReceiver, GossipTopic}, proto::TopicId};
use n0_future::StreamExt;

use super::{get_server_addresses, GossipFuture, Server, User};



#[derive(Debug)]
pub struct Connection {
    pub user: User,
    pub server: Server,
    pub user_gossip_topic: GossipTopic
}

#[derive(Debug, Clone)]
pub struct ConnectOptions {
    pub starting_server_id: u64,
    pub debug: bool,
    pub search_duration: Duration,
    pub n_server_to_search: u64,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        ConnectOptions {
            starting_server_id: 0,
            debug: false,
            search_duration: Duration::from_secs(7),
            n_server_to_search: 25,
        }
    }
}

#[derive(Debug, Clone)]
struct ConnectionArgs{
    topic_id: TopicId,
    relay_vec: Vec<String>,
    my_relay_url: RelayUrl,
    seed: [u8; 32],
    server_addrs_map: HashMap<NodeId, u64>,
}


impl Connection {
    pub async fn create(topic_id: TopicId, relays_str: &[&str], seed: &[u8; 32]) -> Result<Self> {
        Connection::create_with_opts(
            topic_id, relays_str, seed, 
            ConnectOptions::default()
        ).await
    }

    pub async fn create_with_opts(topic_id: TopicId, relays_str: &[&str], seed: &[u8; 32], options: ConnectOptions) -> Result<Self> {
        // if options.debug { println!(""); }
        let relay_vec: Vec<String> = relays_str.iter().map(|s| s.to_string()).collect();
        let user = User::random_with_topic(topic_id).await?;
        let my_relay_url = user.relay_url().unwrap();
        let mut args = ConnectionArgs{
            topic_id, relay_vec, seed: *seed, server_addrs_map: HashMap::new(), my_relay_url
        };

        let mut user_handle: GossipFuture = try_to_connect_to_already_existing_servers(&user, &mut args, &options).await?; 
        let connection = tokio::select! {
            finished_handle = &mut user_handle => {
                let user_gossip_topic = finished_handle??;
                let server_id: u64 = get_lowest_offline_server_id(&user, &args)?;
                let (server, server_handle) = start_your_own_server(
                    server_id, user.node_id().unwrap(), &args, &options
                ).await?;
                let server_gossip_topic = server_handle.await??;
                if options.debug { println!("> server connected!") };
                let (_, receiver) = server_gossip_topic.split();
                let endpoint_clone = user.endpoint().unwrap().clone();
                tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
                Connection {user, server, user_gossip_topic}
            }
            _ = tokio::time::sleep(options.search_duration) => {
                let server_id: u64 = rand::random::<u64>() % options.n_server_to_search;
                if options.debug { println!("> server_id Id({})", server_id) };
                let (server, server_handle) = start_your_own_server(
                    server_id, user.node_id().unwrap(), &args, &options
                ).await?;
                if options.debug { println!("> server started") };
                let user_gossip_topic = user_handle.await??;
                if options.debug { println!("> user connected!") };
                let server_gossip_topic = server_handle.await??;
                if options.debug { println!("> server connected!") };
                let (_, receiver) = server_gossip_topic.split();
                let endpoint_clone = user.endpoint().unwrap().clone();
                tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
                Connection {user, server, user_gossip_topic}
            }
        };


        // let connection = tokio::select! {
        //     finished_handle = user_handle => {
        //         let user_gossip_topic = finished_handle??;
        //         let server_id: u64 = get_lowest_offline_server_id(&user, &args)?;
        //         let (server, server_handle) = start_your_own_server(
        //             server_id, user.node_id().unwrap(), &args, &options
        //         ).await?;
        //         let server_gossip_topic = server_handle.await??;
        //         if options.debug { println!("> server connected!") };
        //         let (_, receiver) = server_gossip_topic.split();
        //         let endpoint_clone = user.endpoint().unwrap().clone();
        //         tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
        //         Connection {user, server, user_gossip_topic}
        //     }
        //     _ = tokio::time::sleep(options.search_duration) => {
        //         let server_id: u64 = rand::random::<u64>() % options.n_server_to_search;
        //         if options.debug { println!("> server_id Id({})", server_id) };
        //         let (server, server_handle) = start_your_own_server(
        //             server_id, user.node_id().unwrap(), &args, &options
        //         ).await?;
        //         if options.debug { println!("> server started") };
        //         let user_gossip_topic = user_handle.await??;
        //         if options.debug { println!("> user connected!") };
        //         let server_gossip_topic = server_handle.await??;
        //         if options.debug { println!("> server connected!") };
        //         let (_, receiver) = server_gossip_topic.split();
        //         let endpoint_clone = user.endpoint().unwrap().clone();
        //         tokio::spawn(async move { server_loop(receiver, endpoint_clone, options.debug).await });
        //         Connection {user, server, user_gossip_topic}
        //     }
        // };

        Ok(connection)
    }
}



#[rustfmt::skip]
async fn try_to_connect_to_already_existing_servers(user: &User, args: &mut ConnectionArgs, options: &ConnectOptions) -> Result<GossipFuture> {
    let end_id = options.starting_server_id + options.n_server_to_search;
    let id_vec: Vec<u64> = (options.starting_server_id..end_id).collect();
    let server_addrs = get_server_addresses(&id_vec, &args.relay_vec, &args.seed).await?;
    for (i, addr) in server_addrs.iter().enumerate() {
        args.server_addrs_map.insert(addr.node_id, i as u64);
    }
    // if options.debug { println!("> trying to connect to servers:\n>\t{:?}", id_vec) };
    if options.debug { println!("> trying to connect to servers:\n{:#?}", args.server_addrs_map) };
    let user_handle: GossipFuture = user.connect_to_servers(server_addrs).await?;
    Ok(user_handle)
}


fn get_lowest_offline_server_id(user: &User, args: &ConnectionArgs) -> Result<u64> {
    let offline_peers = user.offline_peers()?;
    let id = offline_peers
        .iter()
        .filter_map(|p| args.server_addrs_map.get(p))
        .min()
        .unwrap();
    Ok(*id)
}


async fn start_your_own_server(server_id:u64, user_node_id: NodeId, args: &ConnectionArgs, options: &ConnectOptions) -> Result<(Server, GossipFuture)> {
    let server = Server::create(
        server_id, args.topic_id, args.my_relay_url.clone(), &args.seed
    ).await?;
    let debug = options.debug;
    let server_clone = server.clone();
    let node_ids = vec![user_node_id];
    let server_handle: GossipFuture = tokio::spawn(async move {
        let mut server_gtopic = server_clone.subscribe(node_ids)?;
        server_gtopic.joined().await?;
        if debug { println!("\n> server_handle: connected!"); }
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
    use std::{str::FromStr, time::Instant};
    use crate::consts::{RELAY_VEC, SEED, TOPIC};
    
    #[tokio::test]
    // run test by using: 'cargo test iroh::connection::tests::connection -- --exact --nocapture'
    async fn connection() -> Result<()> {
        tracing_subscriber::fmt::init();
        let start = Instant::now();
        let options = ConnectOptions { debug: true, ..Default::default()};
        let topic_id = TopicId::from_str(TOPIC)?;
        let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
        println!("> user.online_peers(), {:#?}", connection.user.online_peers());
        println!("> server.online_peers(), {:#?}", connection.server.online_peers());
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
        connection.server.close().await?;
        Ok(())
    }
}

