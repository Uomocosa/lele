use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // lele::__tests__::lele_connections().await
    // lele::__tests__::iroh_connections().await
    lele::__tests__::iroh_connections_v2().await
}

#[cfg(test)]
// run test by using: '???'
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
        str::FromStr,
        time::{Duration, Instant},
    };

    use iroh::{
        Endpoint, NodeAddr, PublicKey, SecretKey, discovery::static_provider::StaticProvider,
    };
    use iroh_gossip::{net::Gossip, proto::TopicId};
    use lele::consts::{SEED, SERVER_PORTS, TOPIC};
    use lele::{
        iroh::{
            Connection, get_all_192_168_server_addresses, get_secret_key_from_ip,
            gossip::{Ticket, create_known_server, join, subscribe_loop},
        },
        string::random_string,
    };
    use rand::{Rng, SeedableRng, rngs::StdRng};

    use super::*;

    async fn get_random_connection() -> Result<Connection> {
        let mut connection = Connection::_empty();
        let secret_key = SecretKey::generate(rand::rngs::OsRng);
        // let _public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
        connection.set_name(&("user_".to_owned() + &random_string(5)));
        connection.secret_key = Some(secret_key);
        connection.set_topic(TOPIC);
        connection.debug = true;
        connection.bind_endpoint().await?;
        println!(
            "> node_addr:\n{:?}",
            connection.endpoint.clone().unwrap().node_addr().await?
        );
        connection.spawn_gossip().await?;
        connection.spawn_router().await?;
        Ok(connection)
    }

    #[test]
    // run test by using: 'cargo test get_random_seed -- --exact --nocapture'
    fn get_random_seed() -> Result<()> {
        let mut rng = rand::thread_rng();
        let random_seed: [u8; 28] = std::array::from_fn(|_| rng.r#gen());
        println!("const SEED: [u8; 28] = {:?};", random_seed);
        Ok(())
    }

    #[test]
    // run test by using: 'cargo test get_random_topic -- --exact --nocapture'
    fn get_random_topic() -> Result<()> {
        let topic = TopicId::from_bytes(rand::random());
        println!("const TOPIC: &str = {:?};", topic.to_string());
        Ok(())
    }

    #[test]
    fn secret_key_from_seed() -> Result<()> {
        let mut seed = [0u8; 32]; // Create new array with 32 elements
        let mut rng = rand::thread_rng();
        let random_seed: [u8; 28] = std::array::from_fn(|_| rng.r#gen());
        seed[0..4].copy_from_slice(&[192, 168, 0, 0]); // 
        seed[4..32].copy_from_slice(&random_seed); // Copy old values
        let rng = StdRng::from_seed(seed);
        let s1 = SecretKey::generate(rng.clone());
        let s2 = SecretKey::generate(rng.clone());
        assert_eq!(s1.public(), s2.public());
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test open_connection -- --exact --nocapture'
    async fn open_connection() -> Result<()> {
        tracing_subscriber::fmt::init();
        let mut connection = get_random_connection().await?;
        let user_ticket = connection.create_ticket().await?;
        println!("> user_ticket: '{user_ticket}'");
        // connection.peers = vec![];
        let _user_handle = tokio::spawn(connection.clone().connect_to_peers());

        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test join_by_ticket -- --exact --nocapture'
    async fn join_by_ticket() -> Result<()> {
        tracing_subscriber::fmt::init();
        let mut connection_1 = get_random_connection().await?;
        let ticket_1: Ticket = connection_1.create_ticket().await?;
        println!("> ticket_1: '{ticket_1}'");
        // connection_1.peers = vec![];
        let handle_1 = tokio::spawn(connection_1.clone().connect_to_peers());

        let mut connection_2 = get_random_connection().await?;
        let ticket_2 = connection_2.create_ticket().await?;
        println!("> ticket_2: '{ticket_2}'");
        // connection.peers = vec![];
        let handle_2 = tokio::spawn(join(connection_2.clone(), ticket_1.to_string()));
        handle_1.await??;
        handle_2.await??;
        connection_1.close().await?;
        connection_2.close().await?;
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test simple_subscribe_and_join -- --exact --nocapture'
    async fn simple_subscribe_and_join() -> Result<()> {
        tracing_subscriber::fmt::init();
        let start = Instant::now();
        let ip = Ipv4Addr::new(192, 168, 54, 42);
        let port: u16 = 52027;
        let server_key = get_secret_key_from_ip(&ip, &SEED);
        println!("> binding server_enpoint ...");
        let server_endpoint = Endpoint::builder()
            .secret_key(server_key.clone())
            .relay_mode(iroh::RelayMode::Default)
            .bind_addr_v4(SocketAddrV4::new(ip, port))
            .bind()
            .await?;
        println!("> spawining server_gossip ...");
        let server_gossip = Gossip::builder().spawn(server_endpoint.clone()).await?;
        println!("> spawining server_router ...");
        let _server_router = iroh::protocol::Router::builder(server_endpoint.clone())
            .accept(iroh_gossip::net::GOSSIP_ALPN, server_gossip.clone())
            .spawn()
            .await?;
        let hard_coded_server_node_addr = NodeAddr::from_parts(
            server_key.public(),
            None,
            vec![SocketAddr::new(IpAddr::V4(ip), port)],
        );
        let server_addrs = vec![hard_coded_server_node_addr];
        println!("> creating discovery service ...");
        let discovery = StaticProvider::from_node_info(server_addrs.clone());
        println!("> binding endpoint ...");
        let endpoint = Endpoint::builder()
            .add_discovery(|_| Some(discovery))
            .secret_key(SecretKey::generate(rand::rngs::OsRng))
            .relay_mode(iroh::RelayMode::Default)
            .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
            // .discovery_local_network()
            .bind()
            .await?;
        println!("> spawining gossip ...");
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        println!("> spawining router ...");
        let _router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(iroh_gossip::net::GOSSIP_ALPN, gossip.clone())
            .spawn()
            .await?;
        let topic_id = TopicId::from_str(TOPIC)?;
        let peer_ids = server_addrs.iter().map(|p| p.node_id).collect();
        println!("> subscribe_and_join to topic ...");
        let (_sender, _receiver) = gossip.subscribe_and_join(topic_id, peer_ids).await?.split();
        println!("> connected!");
        // println!("{:?}", endpoint.());
        server_endpoint.close().await;
        endpoint.close().await;
        println!("> finished [{:?}]", start.elapsed());
        return Ok(());
    }

    #[tokio::test]
    // run test by using: 'cargo test test_2 -- --exact --nocapture'
    async fn test_2() -> Result<()> {
        tracing_subscriber::fmt::init();
        let start = Instant::now();
        let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
        println!("> server.public_key(): {:?}", server.public_key());
        server.peers = vec![];
        let mut server_handle = None;
        if !server.is_empty() {
            println!("> trying to start server ...");
            server_handle = Some(tokio::spawn(server.clone().connect_to_peers()));
        } else {
            println!("> cannot start a server");
        }
        let ipv4 = Ipv4Addr::new(192, 168, 1, 55);
        println!("> getting hard_coded_server_node_addr ...");
        let hard_coded_server_node_addr = NodeAddr::from_parts(
            PublicKey::from_str(
                "01c639f0cd9424bde4895a7ddeb219598b0a98a7a910855ec4decb1880207cb8",
            )?,
            None,
            vec![SocketAddr::new(IpAddr::V4(ipv4), 52027)],
        );
        println!("> creating user connection ...");
        let mut user = Connection::_empty();
        user.peers = vec![hard_coded_server_node_addr];
        user.set_name(&("user_".to_owned() + &random_string(5)));
        user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
        user.set_topic(TOPIC);
        user.bind_endpoint().await?;
        user.spawn_gossip().await?;
        user.spawn_router().await?;
        println!("> connecting user ...");
        let user_handle = tokio::spawn(user.clone().connect_to_peers());
        let (user_tx, user_rx) = user_handle.await??;
        println!("> connected!");
        tokio::spawn(subscribe_loop(user_rx));
        user._send(user_tx, "hello").await?;

        if let Some(server_handle) = server_handle {
            println!("> trying to get server_handle ...");
            let (server_tx, server_rx) = server_handle.await??;
            tokio::spawn(subscribe_loop(server_rx));
            server._send(server_tx, "HI!").await?;
            println!("> server_handle obtained!");
        } else {
            println!("> no server was established");
        }

        println!("> finished [{:?}]", start.elapsed());
        std::thread::sleep(Duration::from_millis(500));
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        user.close().await?;
        if !server.is_empty() {
            server.close().await?;
        }
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test test_3 -- --exact --nocapture'
    async fn test_3() -> Result<()> {
        // tracing_subscriber::fmt::init();
        let start = Instant::now();
        let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
        println!("> server.public_key(): {:?}", server.public_key());
        server.peers = vec![];
        let mut server_handle = None;
        if !server.is_empty() {
            println!("> trying to start server ...");
            server_handle = Some(tokio::spawn(server.clone().connect_to_peers()));
        } else {
            println!("> cannot start a server");
        }
        let ipv4 = Ipv4Addr::new(192, 168, 1, 55);
        println!("> getting hard_coded_server_node_addr ...");
        let _hard_coded_server_node_addr = NodeAddr::from_parts(
            PublicKey::from_str(
                "01c639f0cd9424bde4895a7ddeb219598b0a98a7a910855ec4decb1880207cb8",
            )?,
            None,
            vec![SocketAddr::new(IpAddr::V4(ipv4), SERVER_PORTS[0])],
        );
        println!("> creating user connection ...");
        let mut user = Connection::_empty();
        // user.peers = vec![hard_coded_server_node_addr];
        user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
        user.set_name(&("user_".to_owned() + &random_string(5)));
        user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
        user.set_topic(TOPIC);
        user.bind_endpoint().await?;
        user.spawn_gossip().await?;
        user.spawn_router().await?;
        println!("> connecting user ...");
        let user_handle = tokio::spawn(user.clone().connect_to_peers());
        let (user_tx, user_rx) = user_handle.await??;
        println!("> connected in [{:?}]", start.elapsed());
        tokio::spawn(subscribe_loop(user_rx));
        user._send(user_tx, "hello").await?;

        if let Some(server_handle) = server_handle {
            println!("> trying to get server_handle ...");
            let (server_tx, server_rx) = server_handle.await??;
            tokio::spawn(subscribe_loop(server_rx));
            server._send(server_tx, "HI!").await?;
            println!("> server_handle obtained!");
        } else {
            println!("> no server was established");
        }

        std::thread::sleep(Duration::from_millis(500));
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        user.close().await?;
        if !server.is_empty() {
            server.close().await?;
        }
        Ok(())
    }
}

// THIS DOES NOT WORK!
// let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
// let sk = SecretKey::generate(rand::rngs::OsRng);
// let sk_bytes = sk.to_bytes();
// let psvk_bytes = public_server_key.to_bytes();
// let mut secret = [0_u8; 16];
// let mut public = [0_u8; 16];
// secret.copy_from_slice(&sk_bytes[..16]);
// public.copy_from_slice(&psvk_bytes[16..]);
// let mut server_key_bytes = [0_u8; 32];
// server_key_bytes[..16].copy_from_slice(&secret);
// server_key_bytes[16..].copy_from_slice(&public);
// let server_key = SecretKey::from_bytes(&server_key_bytes);
// println!("public_old: '{:?}'\npublic_new: '{:?}'", public_server_key.public(), server_key.public());

// DOES NOT WORK!
// let batch_size = 5;
// let n = all_server_addresses.len()/batch_size + 1;
// println!("> n: {n}");
// let mut handles = Vec::new();
// for i in 0..n {
//     let (mut start, mut end) = (batch_size*i, batch_size*(i+1));
//     if i == (n-1) { (start, end) = (batch_size*(i), all_server_addresses.len()) }
//     if start == end { continue; }
//     println!("> start: {start}, end: {end}");
//     let adrr_slice = all_server_addresses[start..end].to_vec();
//     user.peers = adrr_slice;
//     handles.push(tokio::spawn(user.clone().connect_to_peers()));
// }

// let user_ticket = user.create_ticket().await?;
// let mut server_handle = None;
// if !server.is_empty() {
//     println!("> trying to start server ...");
//     server_handle = Some(tokio::spawn(join(
//         server.clone(),
//         user_ticket.to_string(),
//     )));
// } else {
//     println!("> cannot start a server");
// }

// println!("> trying to connect to any server");
// while !handles.iter().any(|h| h.is_finished()) {
//     std::thread::sleep(Duration::from_millis(10));
// }

// let mut user_handle = None;
// for handle in handles {
//     if handle.is_finished() {
//         user_handle = Some(handle);
//         break;
//     }
// }
// let user_handle: tokio::task::JoinHandle<Option<(GossipSender, GossipReceiver)>> = user_handle.unwrap();

// fn _get_192_168_server_addresses_old(possible_ports: &[u16]) -> Result<Vec<NodeAddr>> {
//     let mut combinations: Vec<(u8, u8, u8, u8, u16)> = Vec::new();
//     for a in 192..=192 {
//         for b in 168..=168 {
//             for c in 50..=55 {
//                 for d in 0..=255 {
//                     for &p in possible_ports {
//                         combinations.push((a, b, c, d, p));
//                     }
//                 }
//             }
//         }
//     }
//     let mut server_addresses = Vec::new();
//     let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
//     for (a, b, c, d, p) in combinations {
//         let node_addr = NodeAddr::from_parts(
//             public_server_key.public(),
//             None,
//             vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a, b, c, d)), p)],
//         );
//         server_addresses.push(node_addr)
//     }
//     Ok(server_addresses)
// }

// #[tokio::test]
// // run test by using: 'cargo test topic_with_3_peers -- --nocapture'
// async fn topic_with_3_peers() -> Result<()> {
//     tracing_subscriber::fmt::init();
//     let mut server = Connection::_empty();
//     let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
//     server.set_name("server");
//     server.port = 52027;
//     server.secret_key = Some(public_server_key);
//     server.set_topic(TOPIC);
//     server.debug = true;
//     server.bind_endpoint().await?;
//     println!(
//         "> server node_addr:\n{:?}",
//         server.endpoint.clone().unwrap().node_addr().await?
//     );
//     server.spawn_gossip().await?;
//     server.spawn_router().await?;
//     let server_handle = tokio::spawn(server.clone().connect_to_peers());
//     println!("> searched address:\n{:?}", server.get_node_addr().await?);
//     println!(
//         "> first server address:\n{:?}",
//         get_server_addresses(&[52027])?[0]
//     );
//     println!(
//         "> last server address:\n{:?}",
//         get_server_addresses(&[52027])?.last()
//     );

//     let mut user_1 = get_random_connection().await?;
//     user_1.peers = get_server_addresses(&[52027])?;
//     let user1_handle = tokio::spawn(user_1.clone().connect_to_peers());

//     let mut user_2 = get_random_connection().await?;
//     user_2.peers = get_server_addresses(&[52027])?;
//     let user2_handle = tokio::spawn(user_2.clone().connect_to_peers());

//     user1_handle.await?;
//     user2_handle.await?;
//     server_handle.await?;
//     user_1.close().await?;
//     user_2.close().await?;
//     server.close().await?;
//     Ok(())
// }

// fn _get_localhost_server_addresses(possible_ports: &[u16]) -> Result<Vec<NodeAddr>> {
//     let mut server_addresses = Vec::new();
//     let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
//     for &p in possible_ports {
//         let node_addr = NodeAddr::from_parts(
//             public_server_key.public(),
//             None,
//             vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), p)],
//         );
//         server_addresses.push(node_addr)
//     }
//     Ok(server_addresses)
// }

// async fn _try_to_create_known_server_old(possible_ports: &[u16]) -> Result<Connection> {
//     let mut server_connection = Connection::_empty();
//     let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
//     for &port in possible_ports {
//         server_connection.set_topic(TOPIC);
//         server_connection.port = port;
//         server_connection.ipv4 = Ipv4Addr::UNSPECIFIED;
//         server_connection.secret_key = Some(public_server_key.clone());
//         if server_connection.bind_endpoint().await.is_err() {
//             server_connection = Connection::_empty();
//             continue;
//         }
//         let server_ports = server_connection.get_ports().await?;
//         let server_ips = server_connection.get_ips().await?;
//         let server_ip_octets = server_ips[0].octets();
//         println!("> server_ip_octets: {:?}", server_ip_octets);
//         if server_ports[0] != port {
//             server_connection = Connection::_empty();
//             continue;
//         }
//         // if server_ip_octets[0] != 192 || server_ip_octets[1] != 168 {
//         if server_ip_octets[0] != 192 || server_ip_octets[1] != 168 || server_ip_octets[2] <= 1 {
//             server_connection = Connection::_empty();
//             continue;
//         }
//         server_connection.set_name("server");
//         println!(
//             "> server node id: {:?}",
//             server_connection
//                 .endpoint
//                 .clone()
//                 .unwrap()
//                 .node_addr()
//                 .await?
//         );
//         println!(
//             "> ticket to join server: {:?}",
//             server_connection.create_ticket().await?.to_string()
//         );
//         server_connection.spawn_gossip().await?;
//         server_connection.spawn_router().await?;
//         break;
//     }
//     Ok(server_connection)
// }

// #[tokio::test]
//     // run test by using: 'cargo test join_by_knwon_peers -- --nocapture'
//     async fn join_by_knwon_peers() -> Result<()> {
//         tracing_subscriber::fmt::init();
//         let mut server = Connection::_empty();
//         let public_server_key = get_secret_key_from_hex(SERVER_KEY_HEX)?;
//         server.set_name("server");
//         server.port = 52027;
//         server.secret_key = Some(public_server_key);
//         server.set_topic(TOPIC);
//         server.debug = true;
//         server.bind_endpoint().await?;
//         println!(
//             "> server node_addr:\n{:?}",
//             server.endpoint.clone().unwrap().node_addr().await?
//         );
//         server.spawn_gossip().await?;
//         server.spawn_router().await?;
//         let server_handle = tokio::spawn(server.clone().connect_to_peers());

//         let mut user = get_random_connection().await?;
//         println!("> searched address:\n{:?}", server.get_node_addr().await?);
//         println!(
//             "> first server address:\n{:?}",
//             get_server_addresses(&[52027])?[0]
//         );
//         println!(
//             "> last server address:\n{:?}",
//             get_server_addresses(&[52027])?.last()
//         );
//         user.peers = get_server_addresses(&[52027])?;
//         // user.peers = vec![server.get_node_addr().await?];
//         // user.peers = vec![];
//         let user_handle = tokio::spawn(user.clone().connect_to_peers());

//         user_handle.await?;
//         server_handle.await?;
//         user.close().await?;
//         server.close().await?;
//         Ok(())
//     }
