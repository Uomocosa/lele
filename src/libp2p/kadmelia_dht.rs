// NOTE:
// THIS IS AI GENERATED (from gemini 2.5 flash)
// I DID NOT TEST IT.
// I DID NOT READ IT. (#TODO)

// use futures::StreamExt;
// use libp2p::{
//     core::transport::upgrade::Version,
//     identity,
//     kad::{
//         store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent, QueryId, QueryResult,
//     },
//     swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
//     tcp::TcpTransport,
//     Transport, PeerId, Multiaddr,
// };
// use std::{error::Error, time::Duration};
// use tokio::io::{stdin, AsyncBufReadExt};

// // We create a custom network behaviour that combines Kademlia and Ping.
// // Kademlia is for peer discovery, and Ping is just to keep connections alive
// // and verify connectivity.
// #[derive(NetworkBehaviour)]
// #[behaviour(to_swarm = "MyBehaviourEvent")]
// struct MyBehaviour {
//     kademlia: Kademlia<MemoryStore>,
//     ping: libp2p::ping::Behaviour,
// }

// // Define the events that our custom behaviour can emit.
// enum MyBehaviourEvent {
//     Kademlia(KademliaEvent),
//     Ping(libp2p::ping::Event),
// }

// // Implement `From` conversions to allow the `Swarm` to handle events
// // from the sub-behaviours and emit our custom `MyBehaviourEvent`.
// impl From<KademliaEvent> for MyBehaviourEvent {
//     fn from(event: KademliaEvent) -> Self {
//         MyBehaviourEvent::Kademlia(event)
//     }
// }

// impl From<libp2p::ping::Event> for MyBehaviourEvent {
//     fn from(event: libp2p::ping::Event) -> Self {
//         MyBehaviourEvent::Ping(event)
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Generate a random Ed25519 keypair for the local peer.
//     let local_key = identity::Keypair::generate_ed25519();
//     let local_peer_id = local_key.pub().to_peer_id();
//     println!("Local peer ID: {:?}", local_peer_id);

//     // Create a TCP transport.
//     let transport = TcpTransport::new(local_key.clone()).boxed();

//     // Configure the Kademlia DHT.
//     let mut kademlia_config = KademliaConfig::default();
//     // Use Kademlia in server mode, allowing it to participate in routing and storage.
//     kademlia_config.set_mode(Some(libp2p::kad::Mode::Server));
//     // Set a query timeout.
//     kademlia_config.set_query_timeout(Duration::from_secs(60));

//     // Create a Kademlia behaviour with a memory store for DHT data.
//     let kademlia = Kademlia::new(local_peer_id, MemoryStore::new(local_peer_id), kademlia_config);

//     // Create a Ping behaviour.
//     let ping = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());

//     // Combine the behaviours into our custom `MyBehaviour`.
//     let behaviour = MyBehaviour { kademlia, ping };

//     // Build the Swarm.
//     let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
//         .executor(Box::new(|fut| {
//             tokio::spawn(fut);
//         }))
//         .build();

//     // Add some known bootstrap nodes. In a real application, these would be
//     // well-known, stable nodes in the network.
//     // Replace these with actual multiaddresses and peer IDs of bootstrap nodes.
//     let bootstrap_nodes = vec![
//         // Example: "/ip4/127.0.0.1/tcp/XXXX/p2p/Qm...".parse()?,
//         // Example: "/ip6/::1/tcp/YYYY/p2p/Qm...".parse()?,
//     ];

//     for addr in bootstrap_nodes {
//         if let Ok(peer_id) = addr.clone().iter().find_map(|proto| match proto {
//             libp2p::multiaddr::Protocol::P2p(hash) => PeerId::from_multihash(hash).ok(),
//             _ => None,
//         }) {
//             swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
//         } else {
//              eprintln!("Warning: Could not parse PeerId from bootstrap node address: {}", addr);
//         }
//     }

//     // Bootstrap the Kademlia DHT. This connects to the bootstrap nodes
//     // and starts populating the routing table.
//     if !bootstrap_nodes.is_empty() {
//         if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
//             eprintln!("Bootstrap failed: {:?}", e);
//         } else {
//             println!("Bootstrapping Kademlia...");
//         }
//     } else {
//         println!("No bootstrap nodes provided. Starting without bootstrapping.");
//     }


//     // Listen on a random TCP port.
//     swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

//     // Read input from stdin to simulate user commands.
//     let mut stdin = stdin().lines();

//     println!("Enter commands: 'find <peer_id>', 'provide <key>', 'get <key>'");

//     // Event loop
//     loop {
//         tokio::select! {
//             // Handle user input from stdin.
//             line = stdin.next() => {
//                 let line = match line {
//                     Some(Ok(line)) => line,
//                     _ => break, // Exit on error or EOF
//                 };
//                 let parts: Vec<&str> = line.split_whitespace().collect();
//                 if parts.len() > 0 {
//                     match parts[0] {
//                         "find" => {
//                             if parts.len() == 2 {
//                                 match parts[1].parse::<PeerId>() {
//                                     Ok(peer_id) => {
//                                         println!("Finding closest peers to {:?}", peer_id);
//                                         swarm.behaviour_mut().kademlia.get_closest_peers(peer_id);
//                                     }
//                                     Err(e) => eprintln!("Invalid PeerId: {}", e),
//                                 }
//                             } else {
//                                 println!("Usage: find <peer_id>");
//                             }
//                         }
//                         "provide" => {
//                              if parts.len() == 2 {
//                                  let key = parts[1].as_bytes().to_vec();
//                                  println!("Providing key: {:?}", parts[1]);
//                                  swarm.behaviour_mut().kademlia.start_providing(key.into()).unwrap_or_else(|e| {
//                                      eprintln!("Failed to start providing: {:?}", e);
//                                  });
//                              } else {
//                                  println!("Usage: provide <key>");
//                              }
//                         }
//                         "get" => {
//                             if parts.len() == 2 {
//                                 let key = parts[1].as_bytes().to_vec();
//                                 println!("Getting providers for key: {:?}", parts[1]);
//                                 swarm.behaviour_mut().kademlia.get_providers(key.into());
//                             } else {
//                                 println!("Usage: get <key>");
//                             }
//                         }
//                         _ => println!("Unknown command. Use 'find', 'provide', or 'get'"),
//                     }
//                 }
//             }
//             // Handle Swarm events.
//             event = swarm.select_next_some() => {
//                 match event {
//                     SwarmEvent::NewListenAddr { address, .. } => {
//                         println!("Listening on {:?}", address);
//                     }
//                     SwarmEvent::Behaviour(event) => {
//                         match event {
//                             MyBehaviourEvent::Kademlia(kademlia_event) => {
//                                 match kademlia_event {
//                                     KademliaEvent::OutboundQueryProgressed { id, result, stats, step } => {
//                                         match result {
//                                             QueryResult::GetClosestPeers(result) => {
//                                                 println!("Query ID {:?} GetClosestPeers: Peers found: {:?}", id, result.peers);
//                                             }
//                                             QueryResult::StartProviding(result) => {
//                                                 println!("Query ID {:?} StartProviding: Key: {:?}", id, result.key);
//                                             }
//                                             QueryResult::GetProviders(result) => {
//                                                 println!("Query ID {:?} GetProviders: Key: {:?}, Providers: {:?}", id, result.key, result.providers);
//                                             }
//                                             QueryResult::RepublishProvider(result) => {
//                                                 println!("Query ID {:?} RepublishProvider: Key: {:?}", id, result.key);
//                                             }
//                                             QueryResult::PutRecord(result) => {
//                                                 println!("Query ID {:?} PutRecord: Key: {:?}", id, result.key);
//                                             }
//                                             QueryResult::GetRecord(result) => {
//                                                 println!("Query ID {:?} GetRecord: Record: {:?}", id, result.record);
//                                             }
//                                             QueryResult::RepublishRecord(result) => {
//                                                 println!("Query ID {:?} RepublishRecord: Key: {:?}", id, result.key);
//                                             }
//                                         }
//                                     }
//                                     KademliaEvent::InboundRequest { request } => {
//                                         // println!("Inbound Kademlia request: {:?}", request);
//                                     }
//                                     KademliaEvent::RoutingUpdated { peer, is_new_peer, addresses, old_peer } => {
//                                         // println!("Routing updated: Peer {:?} (new: {}), Addresses: {:?}", peer, is_new_peer, addresses);
//                                     }
//                                     KademliaEvent::UnroutablePeer { peer } => {
//                                         // println!("Unroutable peer: {:?}", peer);
//                                     }
//                                     KademliaEvent::RoutablePeer { peer, address } => {
//                                         // println!("Routable peer: {:?} at {:?}", peer, address);
//                                     }
//                                     KademliaEvent::PendingRoutablePeer { peer, address } => {
//                                         // println!("Pending routable peer: {:?} at {:?}", peer, address);
//                                     }
//                                     KademliaEvent::ModeChanged { new_mode } => {
//                                         println!("Kademlia mode changed to {:?}", new_mode);
//                                     }
//                                 }
//                             }
//                             MyBehaviourEvent::Ping(ping_event) => {
//                                 // println!("Ping event: {:?}", ping_event);
//                             }
//                         }
//                     }
//                     // Handle other swarm events like connection established, closed, etc.
//                     _ => {}
//                 }
//             }
//         }
//     }

//     Ok(())
// }
