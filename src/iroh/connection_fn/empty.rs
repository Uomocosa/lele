use std::net::Ipv4Addr;

use crate::iroh::Connection;
use crate::iroh::gossip::Command;

pub fn empty() -> Connection {
    Connection {
        command: Command::Open { topic: None },
        secret_key: None,
        no_relay: true,
        relay: None,
        port: 0,
        ipv4: Ipv4Addr::UNSPECIFIED,
        name: None,

        ticket: None,
        topic: None,
        peers: vec![],

        endpoint: None,
        gossip: None,
        router: None,

        // peers: Vec::<PublicKey>::new(),
        debug: false,
    }
}
