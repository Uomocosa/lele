use iroh::{protocol::Router, Endpoint, RelayUrl};
use iroh_gossip::{net::Gossip, proto::TopicId};


#[derive(Debug, Clone)]
pub struct IrohData {
    pub endpoint: Endpoint,
    pub gossip: Gossip,
    pub router: Router,
    
    pub topic_id: TopicId,
    pub relay_url: RelayUrl,
}