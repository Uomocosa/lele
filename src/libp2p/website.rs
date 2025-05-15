// NOTE:
// THIS IS AI GENERATED (from claude 3.7)
// I DID NOT TEST IT.
// I DID NOT READ IT. (#TODO)


use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    core::multiaddr::Protocol,
    floodsub::{self, Floodsub, FloodsubEvent},
    identity, mdns, noise, ping, swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};
use warp::Filter;

// Define the P2P network behavior
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "AppEvent")]
struct AppBehaviour {
    floodsub: Floodsub,
    mdns: mdns::async_io::Behaviour,
    ping: ping::Behaviour,
}

// Events from the network behavior
#[derive(Debug)]
enum AppEvent {
    Floodsub(FloodsubEvent),
    Mdns(mdns::Event),
    Ping(ping::Event),
}

impl From<FloodsubEvent> for AppEvent {
    fn from(event: FloodsubEvent) -> Self {
        AppEvent::Floodsub(event)
    }
}

impl From<mdns::Event> for AppEvent {
    fn from(event: mdns::Event) -> Self {
        AppEvent::Mdns(event)
    }
}

impl From<ping::Event> for AppEvent {
    fn from(event: ping::Event) -> Self {
        AppEvent::Ping(event)
    }
}

// P2P message types
#[derive(Debug, Serialize, Deserialize, Clone)]
enum Message {
    RequestWebsite,
    WebsiteContent { html: String },
    PeerAnnouncement { ip: String, port: u16 },
}

// Application state
struct AppState {
    peers: HashSet<PeerId>,
    known_nodes: HashSet<String>,
}

static STATE: Lazy<Arc<Mutex<AppState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(AppState {
        peers: HashSet::new(),
        known_nodes: HashSet::new(),
    }))
});

// Website HTML content including PWA features
fn get_html_content() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>P2P Web Server</title>
    <link rel="manifest" href="/manifest.json">
    <meta name="theme-color" content="\#4CAF50">
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background-color: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #4CAF50;
            text-align: center;
        }
        .card {
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 15px;
            margin-bottom: 15px;
        }
        button {
            background-color: #4CAF50;
            color: white;
            border: none;
            padding: 10px 15px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
        }
        button:hover {
            background-color: #45a049;
        }
        #peers-list {
            list-style-type: none;
            padding: 0;
        }
        #peers-list li {
            padding: 8px;
            border-bottom: 1px solid #eee;
        }
        #install-container {
            display: none;
            margin: 20px 0;
            text-align: center;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>P2P Web Server</h1>
        
        <div class="card">
            <h2>About</h2>
            <p>This is a peer-to-peer web server powered by Rust and libp2p. Anyone accessing this website is automatically part of the network.</p>
            <p>You can install this as a Progressive Web App (PWA) to contribute to the P2P network by hosting this website from your device!</p>
        </div>

        <div id="install-container" class="card">
            <h2>Install as PWA</h2>
            <p>Install this app on your device to become a P2P server!</p>
            <button id="install-button">Install</button>
        </div>
        
        <div class="card">
            <h2>Network Status</h2>
            <p>Connected peers in the P2P network:</p>
            <ul id="peers-list">
                <li>Loading peers...</li>
            </ul>
            <button id="refresh-peers">Refresh</button>
        </div>
    </div>

    <script>
        // Register service worker for PWA
        if ('serviceWorker' in navigator) {
            window.addEventListener('load', () => {
                navigator.serviceWorker.register('/sw.js')
                    .then(reg => console.log('Service Worker registered'))
                    .catch(err => console.error('Service Worker registration failed', err));
            });
        }

        // Handle PWA installation
        let deferredPrompt;
        const installContainer = document.getElementById('install-container');
        const installButton = document.getElementById('install-button');

        window.addEventListener('beforeinstallprompt', (e) => {
            e.preventDefault();
            deferredPrompt = e;
            installContainer.style.display = 'block';
        });

        installButton.addEventListener('click', () => {
            installContainer.style.display = 'none';
            deferredPrompt.prompt();
            deferredPrompt.userChoice.then((choiceResult) => {
                if (choiceResult.outcome === 'accepted') {
                    console.log('User accepted the installation');
                }
                deferredPrompt = null;
            });
        });

        // Refresh peer list
        function refreshPeers() {
            fetch('/api/peers')
                .then(response => response.json())
                .then(peers => {
                    const peersList = document.getElementById('peers-list');
                    peersList.innerHTML = '';
                    
                    if (peers.length === 0) {
                        peersList.innerHTML = '<li>No peers connected</li>';
                    } else {
                        peers.forEach(peer => {
                            const li = document.createElement('li');
                            li.textContent = peer;
                            peersList.appendChild(li);
                        });
                    }
                })
                .catch(error => {
                    console.error('Error fetching peers:', error);
                    document.getElementById('peers-list').innerHTML = 
                        '<li>Error loading peers</li>';
                });
        }

        // Initial load and periodic refresh
        refreshPeers();
        document.getElementById('refresh-peers').addEventListener('click', refreshPeers);
        setInterval(refreshPeers, 10000);
    </script>
</body>
</html>"#.to_string()
}

// Service worker script for PWA functionality
fn get_service_worker() -> String {
    r#"const CACHE_NAME = 'p2p-web-server-v1';
const urlsToCache = [
  '/',
  '/manifest.json'
];

// Install service worker and cache assets
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('Opened cache');
        return cache.addAll(urlsToCache);
      })
  );
  self.skipWaiting();
});

// Serve cached content when offline
self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request)
      .then(response => {
        if (response) {
          return response;
        }
        return fetch(event.request);
      }
    )
  );
});

// Clean up old caches
self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName !== CACHE_NAME) {
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
  self.clients.claim();
});

// PWA lifecycle events
self.addEventListener('appinstalled', (event) => {
  console.log('PWA was installed');
});

// Auto-start with system
self.addEventListener('periodicsync', (event) => {
  if (event.tag === 'start-p2p-server') {
    console.log('Starting P2P server automatically');
    self.clients.openWindow('/');
  }
});

// Register for auto-start
navigator.serviceWorker.ready.then((registration) => {
  if ('periodicSync' in registration) {
    registration.periodicSync.register('start-p2p-server', {
      minInterval: 24 * 60 * 60 * 1000, // Once per day
    });
  }
});"#.to_string()
}

// PWA manifest
fn get_manifest() -> String {
    r#"{
  "name": "P2P Web Server",
  "short_name": "P2P Server",
  "start_url": "/",
  "display": "standalone",
  "background_color": "\#ffffff",
  "theme_color": "\#4CAF50",
  "icons": [
    {
      "src": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 512 512'%3E%3Cpath fill='%234CAF50' d='M256 8C119 8 8 119 8 256s111 248 248 248 248-111 248-248S393 8 256 8zm0 448c-110.5 0-200-89.5-200-200S145.5 56 256 56s200 89.5 200 200-89.5 200-200 200z'/%3E%3Cpath fill='%234CAF50' d='M256 128c-13.3 0-24 10.7-24 24v116c0 13.3 10.7 24 24 24s24-10.7 24-24V152c0-13.3-10.7-24-24-24zm0 192c-13.3 0-24 10.7-24 24s10.7 24 24 24 24-10.7 24-24-10.7-24-24-24z'/%3E%3C/svg%3E",
      "sizes": "512x512",
      "type": "image/svg+xml",
      "purpose": "any maskable"
    }
  ]
}"#.to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Start P2P network
    let (peer_id, mut swarm) = setup_p2p_network().await?;
    info!("Local peer ID: {peer_id}");

    // Find our local IP address
    let local_ip = local_ip_address::local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let web_port = 3000;
    
    // Start the web server
    tokio::spawn(start_web_server(local_ip, web_port));
    
    // Start P2P event loop
    tokio::spawn(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(AppEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _) in list {
                        info!("mDNS discovered peer: {peer_id}");
                        let mut state = STATE.lock().unwrap();
                        state.peers.insert(peer_id);
                        swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id);
                    }
                }
                SwarmEvent::Behaviour(AppEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _) in list {
                        info!("mDNS peer expired: {peer_id}");
                        let mut state = STATE.lock().unwrap();
                        state.peers.remove(&peer_id);
                        swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(AppEvent::Floodsub(FloodsubEvent::Message(message))) => {
                    if let Ok(msg) = serde_json::from_slice::<Message>(&message.data) {
                        match msg {
                            Message::RequestWebsite => {
                                info!("Received website request");
                                let response = Message::WebsiteContent {
                                    html: get_html_content(),
                                };
                                let topic = floodsub::Topic::new("website");
                                if let Ok(json) = serde_json::to_string(&response) {
                                    swarm.behaviour_mut().floodsub.publish(topic, json.as_bytes());
                                }
                            }
                            Message::WebsiteContent { html: _ } => {
                                info!("Received website content from peer");
                            }
                            Message::PeerAnnouncement { ip, port } => {
                                info!("Peer announced at {}:{}", ip, port);
                                let mut state = STATE.lock().unwrap();
                                let addr_str = format!("{}:{}", ip, port);
                                if !state.known_nodes.contains(&addr_str) {
                                    state.known_nodes.insert(addr_str.clone());
                                    // Try to dial the new peer
                                    if let Ok(addr) = format!("/ip4/{}/tcp/{}", ip, port).parse::<Multiaddr>() {
                                        info!("Dialing newly announced peer at {}", addr);
                                        let _ = swarm.dial(addr);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // Open the website in the default browser
    info!("Opening web server URL in browser");
    let url = format!("http://{}:{}", local_ip, web_port);
    if let Err(e) = webbrowser::open(&url) {
        error!("Failed to open browser: {}", e);
        println!("Please open this URL manually: {}", url);
    }

    // Keep the main thread running
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn setup_p2p_network() -> Result<(PeerId, Swarm<AppBehaviour>)> {
    // Create a random keypair for our identity
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    // Set up the transport with noise for encryption and yamux for multiplexing
    let transport = tcp::async_io::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::NoiseAuthenticated::xx(&id_keys).unwrap())
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    // Create a floodsub topic
    let topic = floodsub::Topic::new("website");

    // Set up the swarm with our transport and behavior
    let mut behaviour = AppBehaviour {
        floodsub: Floodsub::new(peer_id),
        mdns: mdns::async_io::Behaviour::new(mdns::Config::default())?,
        ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(15))),
    };

    // Subscribe to the topic
    behaviour.floodsub.subscribe(topic);

    // Build the swarm
    let swarm = libp2p::SwarmBuilder::with_async_std_executor(
        transport,
        behaviour,
        peer_id,
    )
    .build();

    Ok((peer_id, swarm))
}

async fn start_web_server(ip: IpAddr, port: u16) {
    // Routes
    let index_route = warp::path::end().map(|| {
        warp::reply::html(get_html_content())
    });

    let sw_route = warp::path("sw.js").map(|| {
        warp::reply::with_header(
            get_service_worker(),
            "content-type",
            "application/javascript",
        )
    });

    let manifest_route = warp::path("manifest.json").map(|| {
        warp::reply::with_header(
            get_manifest(),
            "content-type",
            "application/json",
        )
    });

    let peers_route = warp::path!("api" / "peers").map(|| {
        let state = STATE.lock().unwrap();
        let peers: Vec<String> = state.peers.iter().map(|p| p.to_string()).collect();
        warp::reply::json(&peers)
    });

    // Combine all routes
    let routes = index_route
        .or(sw_route)
        .or(manifest_route)
        .or(peers_route);

    // Start the server
    let addr = SocketAddr::new(ip, port);
    info!("Starting web server on {}", addr);
    warp::serve(routes).run(addr).await;
}