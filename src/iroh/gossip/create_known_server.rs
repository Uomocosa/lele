use std::net::SocketAddrV4;

use anyhow::Result;

use crate::iroh::{Connection, get_secret_key_from_ip, get_server_ip};

pub async fn create_known_server(
    possible_ports: &[u16],
    topic: &str,
    seed: &[u8; 28],
) -> Result<Connection> {
    let mut server_connection = Connection::_empty();
    for &port in possible_ports {
        println!("> port: {}", port);
        let ip = match get_server_ip(port).await {
            Ok(ip) => ip,
            Err(_) => continue,
        };
        println!("> creating connection with ip: {:?}", ip);
        server_connection.set_topic(topic);
        server_connection.port = port;
        server_connection.ipv4 = ip;
        server_connection.set_secret_key(get_secret_key_from_ip(&ip, seed));
        if server_connection.bind_endpoint().await.is_err() {
            server_connection = Connection::_empty();
            continue;
        }
        let server_sockets = server_connection.socket_addresses().await?;
        if !server_sockets.contains(&SocketAddrV4::new(ip, port)) {
            server_connection = Connection::_empty();
            continue;
        }
        server_connection.set_name("server");
        println!(
            "> server node id: {:?}",
            server_connection
                .endpoint
                .clone()
                .unwrap()
                .node_addr()
                .await?
        );
        println!(
            "> ticket to join server: {:?}",
            server_connection.create_ticket().await?.to_string()
        );
        println!("> server.spawn_gossip() ...");
        server_connection.spawn_gossip().await?;
        println!("> server.spawn_router() ...");
        server_connection.spawn_router().await?;
        break;
    }
    Ok(server_connection)
}
