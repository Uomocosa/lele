use anyhow::{Result, anyhow, bail};
use iroh::{Endpoint, RelayMap, RelayMode, discovery::static_provider::StaticProvider};

use crate::iroh::Connection;

pub async fn bind_endpoint(connection: &mut Connection) -> Result<&mut Connection> {
    if connection.secret_key.is_none() {
        return Err(anyhow!("Connection::bind_endpoint::SecretKeyNotFound"));
    }
    let name = connection.name.clone().unwrap_or("???".to_string());
    let secret_key = connection.secret_key.clone().unwrap();
    let relay_mode = match (connection.no_relay, connection.relay.clone()) {
        (false, None) => RelayMode::Default,
        (false, Some(url)) => RelayMode::Custom(RelayMap::from_url(url)),
        (true, None) => RelayMode::Disabled,
        (true, Some(_)) => bail!("You cannot set --no-relay and --relay at the same time"),
    };
    // let discovery = StaticProvider::from_node_info(connection.peers.clone());
    let discovery = StaticProvider::new();
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(relay_mode)
        .bind_addr_v4(connection.socket_addr())
        .add_discovery({
            let discovery = discovery.clone();
            move |_| Some(discovery)
        })
        .discovery_local_network()
        .bind()
        .await?;
    connection.endpoint = Some(endpoint.clone());
    connection.discovery = Some(discovery.clone());
    if connection.debug {
        println!("> {name}: our node id: {}", endpoint.node_id());
    }
    Ok(connection)
}
