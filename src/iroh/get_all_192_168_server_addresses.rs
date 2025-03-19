use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use iroh::NodeAddr;

use super::get_secret_key_from_ip;

pub fn get_all_192_168_server_addresses(
    possible_ports: &[u16],
    seed: &[u8; 28],
) -> Result<Vec<NodeAddr>> {
    let mut combinations: Vec<(Ipv4Addr, u16)> = Vec::new();
    for a in 192..=192 {
        for b in 168..=168 {
            for c in 0..=255 {
                for d in 0..=255 {
                    for &p in possible_ports {
                        combinations.push((Ipv4Addr::new(a, b, c, d), p));
                    }
                }
            }
        }
    }
    let mut server_addresses = Vec::new();
    for (ip, port) in combinations {
        let node_addr = NodeAddr::from_parts(
            get_secret_key_from_ip(&ip, seed).public(),
            None,
            vec![SocketAddr::new(IpAddr::from(ip), port)],
        );
        server_addresses.push(node_addr)
    }
    Ok(server_addresses)
}
