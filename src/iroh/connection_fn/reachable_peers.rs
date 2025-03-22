use std::time::Duration;

use anyhow::Result;
use iroh::{endpoint::ConnectionError, NodeAddr};

use crate::{iroh::Connection, vector::chunk_vector};

// This does not work even on the same PC.
pub async fn _reachable_peers(connection: Connection) -> Result<Vec<NodeAddr>> {
    let chunk_len = 255 * 51;
    let adrrs_chunks = chunk_vector(&connection.peers, chunk_len);
    let mut succesfull_connections: Vec<NodeAddr> = Vec::new();
    let mut resulting_connections = Vec::new();
    for chunk in &adrrs_chunks {
        println!("> checking {} node addresses", chunk.len());
        for node_addr in chunk.clone() {
            let endpoint = connection.endpoint.clone().unwrap();
            resulting_connections.push(tokio::spawn(async move {
                endpoint
                    .connect(node_addr, iroh_gossip::net::GOSSIP_ALPN)
                    .await
            }));
        }
        println!("> waiting for one connection to finish ...");
        let res = loop {
            while !resulting_connections.iter().any(|res| res.is_finished()) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            let index = resulting_connections
                .iter()
                .position(|x| x.is_finished())
                .unwrap();
            // find(|x| x.is_finished());
            println!("> index: {:?}", index);
            let handle = resulting_connections.remove(index);
            match handle.await? {
                Ok(_) => break Ok(chunk[index].clone()),
                // Err(e) if e.is::<ConnectionError>()  => break Err(e),
                Err(e) if Some(&ConnectionError::TimedOut) == e.downcast_ref() => break Err(e),
                Err(_) => continue,
            }
        };
        println!("> res.is_ok(): {}", res.is_ok());
        if res.is_ok() {
            succesfull_connections.push(res?);
            // break;
        }
    }
    Ok(succesfull_connections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use iroh::SecretKey;
    use tokio::time::Instant;
    use crate::{consts::{SEED, SERVER_PORTS, TOPIC}, iroh::{get_all_192_168_server_addresses, gossip::create_known_server, Connection}, string::random_string};

    #[tokio::test]
    // run test by using: 'cargo test iroh::connection_fn::reachable_peers::tests::one -- --exact --nocapture'
    pub async fn one() -> Result<()> {
        // tracing_subscriber::fmt::init();
        let start = Instant::now();
        let mut server = create_known_server(SERVER_PORTS, TOPIC, &SEED).await?;
        if server.is_empty() { 
            println!("> creating private_server ...");
            server.set_name("private_server");
            server.set_topic(TOPIC);
            server.set_random_secret_key();
            server.bind_endpoint().await?;
            server.spawn_gossip().await?;
            server.spawn_router().await?;
        }
        println!("> server.public_key(): {:?}", server.public_key());

        println!("> creating user connection ...");
        let mut user = Connection::_empty();
        user.peers = get_all_192_168_server_addresses(SERVER_PORTS, &SEED)?;
        user.set_name(&("user_".to_owned() + &random_string(5)));
        user.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
        user.set_topic(TOPIC);
        user.bind_endpoint().await?;

        // tokio::spawn(_reachable_peers(user.clone())); // works!
        println!("> searching for reachable_peers ...");
        let reachable_peers = _reachable_peers(user.clone()).await?;
        println!("> reachable_peers:\n{:?}", reachable_peers);

        println!("> finished [{:?}]", start.elapsed());
        tokio::time::sleep(Duration::from_millis(250)).await;
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        println!("> closing user ..."); user.close().await?;
        println!("> closing server ..."); server.close().await?;
        Ok(())
    }
}