use std::str::FromStr;

use anyhow::Result;
use iroh::{NodeAddr, RelayUrl};

use super::get_server_addr;

pub async fn get_server_addresses(
    id_vec: &[u64], relay_vec: &[&str], seed: &[u8; 32]
) -> Result<Vec<NodeAddr>> {
    let mut addresses: Vec<NodeAddr> = Vec::new();
    for &id in id_vec {
        for relay_str in relay_vec {
            let relay_url = RelayUrl::from_str(relay_str)?;
            let server_addr = get_server_addr(id, relay_url, seed).await;
            addresses.push(server_addr);
        }
    }
    Ok(addresses)
}