use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, NodeAddr, RelayUrl};
use iroh_gossip::{net::Gossip, proto::TopicId};


#[derive(Debug, Clone)]
pub struct IrohData{
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    topic_id: TopicId,
    relay_vec: Vec<RelayUrl>,
}

#[derive(Debug, Clone)]
pub enum Instance{
    Empty,
    Data { iroh_data: IrohData, debug: bool },
}

// create and close methods
pub trait IrohInstance {
    fn empty() -> Instance {
        Instance::Empty
    }

    async fn close(instance: Instance) -> Result<()> {
        match instance {
            Instance::Empty => {},
            Instance::Data { iroh_data, ..} => {
                iroh_data.endpoint.close().await;
                iroh_data.router.shutdown().await?;
            }
        }
        Ok(())
    }
}


// Useful methods
impl Instance {
    pub async fn search_for_instances(&self, instance_node_addrs: Vec<NodeAddr>) -> Result<()> {
        for node_addr in &instance_node_addrs {
            if node_addr.relay_url.is_none() {
                return Err(anyhow!("search_for_instances::NoRelayUrlFound"));
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_search_for_instances() {
    
}

// Standard getter and setters
impl Instance {
    pub fn data(&self) -> Option<IrohData> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.clone()),
        }
    }

    pub fn endpoint(&self) -> Option<Endpoint> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.endpoint.clone())
        }
    }

    pub fn gossip(&self) -> Option<Gossip> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.gossip.clone())
        }
    }

    pub fn router(&self) -> Option<Router> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.router.clone())
        }
    }

    pub fn topic_id(&self) -> Option<TopicId> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.topic_id)
        }
    }

    pub fn relay_vec(&self) -> Option<Vec<RelayUrl>> {
        match self {
            Instance::Empty => None,
            Instance::Data { iroh_data , ..} => Some(iroh_data.relay_vec.clone())
        }
    }

    pub fn set_endpoint(self, endpoint: Endpoint) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_endpoint::InstanceIsEmpty")),
            Instance::Data { iroh_data , debug} => {
                let mut new_data = iroh_data;
                new_data.endpoint = endpoint;
                Ok(Instance::Data{iroh_data: new_data, debug})
            }
        }
    }

    pub fn set_gossip(self, gossip: Gossip) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_gossip::InstanceIsEmpty")),
            Instance::Data { iroh_data , debug} => {
                let mut new_data = iroh_data;
                new_data.gossip = gossip;
                Ok(Instance::Data{iroh_data: new_data, debug})
            }
        }
    }

    pub fn set_router(self, router: Router) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_router::InstanceIsEmpty")),
            Instance::Data { iroh_data , debug} => {
                let mut new_data = iroh_data;
                new_data.router = router;
                Ok(Instance::Data{iroh_data: new_data, debug})
            }
        }
    }

    pub fn set_topic_id(self, topic_id: TopicId) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_topic_id::InstanceIsEmpty")),
            Instance::Data { iroh_data , debug} => {
                let mut new_data = iroh_data;
                new_data.topic_id = topic_id;
                Ok(Instance::Data{iroh_data: new_data, debug})
            }
        }
    }

    pub fn set_relay_vec(self, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_relay_vec::InstanceIsEmpty")),
            Instance::Data { iroh_data , debug} => {
                let mut new_data = iroh_data;
                new_data.relay_vec = relay_vec;
                Ok(Instance::Data{iroh_data: new_data, debug})
            }
        }
    }

    pub fn set_debug(self, debug: bool) -> Result<Self> {
        match self {
            Instance::Empty => Err(anyhow!("set_debug::InstanceIsEmpty")),
            Instance::Data { iroh_data , ..} => {
                Ok(Instance::Data{iroh_data, debug})
            }
        }
    }
}