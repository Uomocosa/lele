use std::str::FromStr;

use iroh::{NodeAddr, PublicKey};
use iroh_gossip::proto::TopicId;

use super::Ticket;

pub fn create_ticket(topic: &str, peers: Vec<PublicKey>) -> Ticket {
    let topic = TopicId::from_str(topic)
        .unwrap_or_else(|_| panic!(
            "Cannot create topic form str: '{}'\nThe str should be somenthing like: 'aa17cebe62afa7cc2315e46c943f47a414acfe4db6155d55dbb50d2b257eb12c'", 
            topic
        )
    );
    let peers: Vec<NodeAddr> = peers.iter().cloned().map(NodeAddr::from).collect();
    Ticket { topic, peers }
}
