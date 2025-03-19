use anyhow::{Result, anyhow};
use iroh::SecretKey;
use std::net::Ipv4Addr;

use super::Connection;

pub async fn get_server_ip(port: u16) -> Result<Ipv4Addr> {
    let mut server_connection = Connection::_empty();
    // server_connection.set_topic();
    server_connection.port = port;
    server_connection.ipv4 = Ipv4Addr::UNSPECIFIED;
    server_connection.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    if server_connection.bind_endpoint().await.is_err() {
        return Err(anyhow!("get_server_ip_and_port::EndpointNotBinded"));
    }
    let server_sockets = server_connection.socket_addresses().await?;
    for socket in server_sockets {
        let server_ip = socket.ip();
        let server_port = socket.port();
        if server_port != port {
            continue;
        }
        let octets = server_ip.octets();
        // if octets[0] == 192 && octets[1] == 168 && octets[2] >= 2 {
        if octets[0] == 192 && octets[1] == 168 {
            return Ok(*server_ip);
        }
    }
    Err(anyhow!("get_server_ip_and_port::NoValidSocketAddress"))
}
