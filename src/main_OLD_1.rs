use std::{
    error::Error, time::{Duration, Instant},
};
use bytes::BufMut;
use futures::StreamExt;
use libp2p::{
    identity, kad, noise,
    swarm::StreamProtocol,
    tcp, yamux,
};
use tracing_subscriber::EnvFilter;

const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();
    println!("{:?}", local_key.public().to_peer_id());

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            // Create a Kademlia behaviour.
            let mut cfg = kad::Config::new(IPFS_PROTO_NAME);
            cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            kad::Behaviour::with_config(key.public().to_peer_id(), store, cfg)
        })?
        .build();
    // let server_mode = Some(libp2p::kad::Mode::Server);
    // swarm.behaviour_mut().set_mode(server_mode); // this should be automatic

    for peer in &BOOTNODES {
        swarm
            .behaviour_mut()
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
    }
    swarm.behaviour_mut().bootstrap()?;
    
    // let peer_id = libp2p::PeerId::random();
    //     println!(">>> Searching for the closest peers to {peer_id} ...");
    //     swarm.behaviour_mut().get_closest_peers(peer_id);

    println!(">>> putting PK record into the DHT ...");
    let mut pk_record_key = vec![];
    pk_record_key.put_slice("/pk/".as_bytes());
    pk_record_key.put_slice(swarm.local_peer_id().to_bytes().as_slice());
    let mut pk_record =
        kad::Record::new(pk_record_key, local_key.public().encode_protobuf());
    pk_record.publisher = Some(*swarm.local_peer_id());
    pk_record.expires = Some(Instant::now() + Duration::from_secs(60));
    swarm
        .behaviour_mut()
        .put_record(pk_record, kad::Quorum::N(std::num::NonZeroUsize::new(3).unwrap()))?;

    const KEY: [u8; 32] = [118, 60, 141, 37, 201, 183, 174, 98, 151, 39, 67, 220, 24, 13, 88, 175, 231, 222, 14, 226, 192, 204, 52, 38, 36, 68, 192, 30, 10, 146, 171, 73];
    let record_key = libp2p::kad::RecordKey::new(&KEY);
    swarm.behaviour_mut()
        .start_providing(record_key.clone())
        .expect("Failed to start providing key");
    swarm.behaviour_mut()
        .get_providers(record_key.clone());

    println!(">>> starting main loop ...");
    let time = Instant::now();
    loop {
        let t = time.elapsed();
        let event = swarm.select_next_some().await;
        if let libp2p::swarm::SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed{ result, ..}) = event {
            use libp2p::kad::QueryResult::*;
            match result {
                Bootstrap(bootstrap_ok) => {
                    println!("[{t:#?}] Bootstrap: {bootstrap_ok:?}");
                }
                GetClosestPeers(get_closest_peers_ok) => {
                    println!("[{t:#?}] GetClosestPeers: {get_closest_peers_ok:?}");
                },
                GetProviders(get_providers_ok) => {
                    println!("[{t:#?}] GetProviders: {get_providers_ok:?}");
                },
                StartProviding(add_provider_ok) => {
                    println!("[{t:#?}] StartProviding: {add_provider_ok:?}");
                },
                RepublishProvider(add_provider_ok) => {
                    println!("[{t:#?}] RepublishProvider: {add_provider_ok:?}");
                },
                GetRecord(get_record_ok) => {
                    println!("[{t:#?}] GetRecord: {get_record_ok:?}");
                },
                PutRecord(put_record_ok) => {
                    println!("[{t:#?}] PutRecord: {put_record_ok:?}");
                },
                RepublishRecord(put_record_ok) => {
                    println!("[{t:#?}] RepublishRecord: {put_record_ok:?}");
                },
            }
            // println!("OutboundQueryProgressed: {result:?}");
        }
    }
}
