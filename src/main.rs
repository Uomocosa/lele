use std::{
    error::Error, time::{Duration, Instant},
};
use bytes::BufMut;
use futures::StreamExt;
use libp2p::{
    identify, identity, kad::{self, store::MemoryStore}, noise, swarm::NetworkBehaviour, tcp, yamux, StreamProtocol
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

    #[derive(NetworkBehaviour)]
    pub struct Behaviour {
        kademlia: kad::Behaviour<MemoryStore>,
        identify: identify::Behaviour,
    }

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = local_key.public().to_peer_id();
    println!("{peer_id:?}");

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            let mut kad_cfg = kad::Config::new(IPFS_PROTO_NAME);
            kad_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            let id_cfg = identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public()
            );
            Ok(Behaviour {
                kademlia: kad::Behaviour::with_config(key.public().to_peer_id(), store, kad_cfg),
                identify: identify::Behaviour::new(id_cfg),
            })
        })?
        .build();
    // let server_mode = Some(libp2p::kad::Mode::Server);
    // swarm.behaviour_mut().set_mode(server_mode); // this should be automatic
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    for peer in &BOOTNODES {
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
    }
    swarm.behaviour_mut().kademlia.bootstrap()?;

    println!(">>> await for bootstrapping ...");
    loop{
        let event = swarm.select_next_some().await;
        if let libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed{result, ..})) = event { 
            if let libp2p::kad::QueryResult::Bootstrap(Ok(bootstrap_ok)) = result {
                if bootstrap_ok.num_remaining == 0 { break; }              
            }
        }
    }

    // let my_key = [118, 60, 141, 37, 201, 183, 174, 98, 151, 39, 67, 220, 24, 13, 88, 175, 231, 222, 14, 226, 192, 204, 52, 38, 36, 68, 192, 30, 10, 146, 171, 73];
    let my_key: Vec<u8> = vec![1, 2, 3, 4, 5];
    // let my_key= "/standardofcomuniation/lele/testing/libp2p";
    let record_key = libp2p::kad::RecordKey::new(&my_key);
    let rand_byte = lele::rnd::get_random_bytes::<1>()[0];
    println!("rand_byte: {rand_byte:?}");
    if rand_byte % 2 == 0 {
        println!(">>> start_providing ...");
        swarm.behaviour_mut()
            .kademlia
            .start_providing(record_key.clone())
            .expect("Failed to start providing key");

        println!(">>> Putting PK record into the DHT ...");

        // let mut pk_record_key = my_key;
        let mut pk_record_key = vec![];
        pk_record_key.put_slice("/pk/".as_bytes());
        pk_record_key.put_slice(swarm.local_peer_id().to_bytes().as_slice());

        let mut pk_record =
            kad::Record::new(pk_record_key, local_key.public().encode_protobuf());
        pk_record.publisher = Some(*swarm.local_peer_id());
        pk_record.expires = Some(Instant::now() + Duration::from_secs(60));

        swarm
            .behaviour_mut()
            .kademlia
            .put_record(pk_record, kad::Quorum::N(std::num::NonZeroUsize::new(3).unwrap()))?;
    }
    else {
        println!(">>> get_providers ...");
        swarm.behaviour_mut()
            .kademlia
            .get_providers(record_key.clone());
    }

    println!(">>> starting main loop ...");
    let time = Instant::now();
    loop {
        let x: Vec<_> = swarm.connected_peers().collect();
        println!("{:#?}", x);
        let t = time.elapsed();
        let event = swarm.select_next_some().await;
        use libp2p::swarm::SwarmEvent as Event;
        use libp2p::kad::QueryResult::*;
        match event {
            Event::ConnectionEstablished{peer_id, ..} => {
                println!("[{t:#?}] Event::ConnectionEstablished: {peer_id:?}")
            }
            Event::NewListenAddr{address, ..} => {
                println!("[{t:#?}] Event::NewListenAddr: {address:?}")
            },
            Event::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed{ result, ..})) => {
                match result {
                    Bootstrap(_bootstrap_ok) => {
                        // println!("[{t:#?}] Bootstrap: {bootstrap_ok:?}");
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
            },
            _ => {}
        }
    }
}