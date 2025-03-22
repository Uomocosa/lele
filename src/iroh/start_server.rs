use std::{net::{SocketAddr, SocketAddrV4}, str::FromStr};

use anyhow::{anyhow, Result};
use iroh::{Endpoint, RelayMap, RelayMode, RelayUrl};

use crate::iroh::get_secret_key_from_ip;

use super::get_server_ip;

pub async fn create_server_endpoint(possible_ports: &[u16], seed: &[u8; 28]) -> Result<Endpoint> {
    for &port in possible_ports {
        // println!("> port: {}", port);
        let ip = match get_server_ip().await {
            Ok(ip) => ip,
            Err(_) => continue,
        };
        // println!("> creating connection with ip: {:?}", ip);
        let relay_mode = RelayMode::Custom(
            RelayMap::from_url(
                RelayUrl::from_str("https://euw1-1.relay.iroh.network./")?
            )
        );
        let server = Endpoint::builder()
            .secret_key(get_secret_key_from_ip(&ip, seed))
            .bind_addr_v4(SocketAddrV4::new(ip, port))
            .relay_mode(relay_mode)
            .bind()
            .await?;
        let server_addrs = server
            .node_addr()
            .await?;
        let server_sockets: Vec<&SocketAddr> = server_addrs
            .direct_addresses()
            .collect();
        if !server_sockets.contains(&&SocketAddr::from(SocketAddrV4::new(ip, port))) {
            continue;
        }
        // println!("> server NodeAddr: {:?}", server.node_addr().await?);
        return Ok(server);
    }
    Err(anyhow!("start_server::NoServerStarted"))
}





#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use super::*;
    use crate::consts::{SERVER_PORTS, SEED};

    #[tokio::test]
    // run test by using: 'cargo test iroh::start_server::tests::one -- --exact --nocapture'
    async fn one() -> Result<()> {
        // println!("SERVER_PORTS: {:?}", SERVER_PORTS);
        let server = create_server_endpoint(SERVER_PORTS, &SEED).await?;
        println!("> server NodeAddr: {:?}", server.node_addr().await?);
        Ok(())
    }

}