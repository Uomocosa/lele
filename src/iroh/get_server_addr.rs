use iroh::{NodeAddr, RelayUrl};

use super::generate_server_secret_key;

pub async fn get_server_addr(id: u64, relay_url: RelayUrl, seed: &[u8; 32]) -> NodeAddr {
    let secret_key = generate_server_secret_key(id, seed);
    let node_id = secret_key.public();
    NodeAddr::new(node_id).with_relay_url(relay_url)
}
