use std::error::Error;

use futures::StreamExt;
use libp2p::{
    kad::{self, store::MemoryStore}, mdns, noise, swarm::{NetworkBehaviour}, tcp, yamux
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    // cargo test libp2p::dht::tests::print_rand_bytes -- --exact --nocapture
    fn print_rand_bytes() {
        let rand_bytes= crate::rnd::get_random_bytes::<32>();
        println!("rand_bytes: {rand_bytes:?}");
    }

    #[tokio::test]
    // cargo test libp2p::dht::tests::one -- --exact --nocapture
    async fn one() -> Result<(), Box<dyn Error>> {
        const KEY: [u8; 32] = [118, 60, 141, 37, 201, 183, 174, 98, 151, 39, 67, 220, 24, 13, 88, 175, 231, 222, 14, 226, 192, 204, 52, 38, 36, 68, 192, 30, 10, 146, 171, 73];
        let record_key = libp2p::kad::RecordKey::new(&KEY);
        dht(record_key).await?;
        Ok(())
    }
}

pub async fn dht(record_key: kad::RecordKey) -> Result<(), Box<dyn Error>> {
    #[derive(NetworkBehaviour)]
    pub struct Behaviour {
        kademlia: kad::Behaviour<MemoryStore>,
        mdns: mdns::tokio::Behaviour,
    }

    let mut swarm: libp2p::Swarm<Behaviour> = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(Behaviour {
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .build();

    // let server_mode = Some(libp2p::kad::Mode::Server);
    // swarm.behaviour_mut().kademlia.set_mode(server_mode); // this should be automatic
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let _put_query = swarm.behaviour_mut().kademlia
        .start_providing(record_key.clone())
        .expect("Failed to start providing key");

    let _get_query = swarm.behaviour_mut().kademlia
        .get_providers(record_key.clone());

    for i in 0..10 {
        let event = swarm.select_next_some().await;
        println!("event ({i}): {event:#?}");
    }
    
    Ok(())
}
