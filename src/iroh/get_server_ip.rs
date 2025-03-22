use anyhow::{Result, anyhow};
use iroh::Endpoint;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};


pub async fn get_server_ip() -> Result<Ipv4Addr> {
    let endpoint = Endpoint::builder().bind().await?;
    let node_addresses = endpoint
        .node_addr()
        .await?;
    // println!("> node_addresses: {:?}", node_addresses);

    let socket_addrs: Vec<&SocketAddr> = node_addresses
        .direct_addresses()
        .collect();
    // println!("> socket_addrs: {:?}", socket_addrs);

    let mut server_sockets: Vec<SocketAddrV4> = Vec::new();
    for addr in socket_addrs {
        match addr.ip() {
            IpAddr::V4(ipv4) => server_sockets.push(SocketAddrV4::new(ipv4, 8080)),
            IpAddr::V6(_) => continue,
        }
    };
    // println!("> server_sockets: {:?}", server_sockets);

    for socket in server_sockets {
        let server_ip = socket.ip();
        // let server_port = socket.port();
        // println!("> server_ip: {:?}", server_ip);
        // // println!("> server_port: {:?}", server_port);
        let octets = server_ip.octets();
        // if octets[0] == 192 && octets[1] == 168 && octets[2] >= 2 {
        if octets[0] == 192 && octets[1] == 168 {
            return Ok(*server_ip);
        }
    }
    Err(anyhow!("get_server_ip::NoValidSocketAddress"))
}
