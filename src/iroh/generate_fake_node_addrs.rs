use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::IpAddr;

use iroh::SecretKey;
use iroh::NodeAddr;
use rand::Rng;


pub fn generate_fake_node_addrs(n: usize) -> Vec<NodeAddr> {
    let mut out_vec: Vec<NodeAddr> = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        out_vec.push(NodeAddr::from_parts(
            SecretKey::generate(rand::rngs::OsRng).public(),
            None,
            vec![SocketAddr::new(
                IpAddr::V4(
                    Ipv4Addr::new(
                        rng.gen_range(1..=254), // Avoid 0 and 255 for simplicity
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                        rng.gen_range(0..=255),
                    )
                ),
                rng.gen_range(1024..=65535) // port
                )
            ]
        ))
    }
    out_vec
}