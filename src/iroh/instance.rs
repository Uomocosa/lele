use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, RelayUrl};
use iroh_gossip::{net::Gossip, proto::TopicId};

use super::IrohData;


#[derive(Debug, Clone)]
pub enum IrohInstance<T> {
    Empty,
    Data { iroh_data: IrohData, data: T, debug: bool },
}

impl<T> IrohInstance<T> {
    pub fn empty() -> Self {
        IrohInstance::Empty
    }

    pub async fn close(self) -> Result<()> {
        match self {
            IrohInstance::Empty => {},
            IrohInstance::Data { iroh_data, ..} => {
                iroh_data.endpoint.close().await;
                iroh_data.router.shutdown().await?;
            }
        }
        Ok(())
    }
}


// etters and getters
impl<T> IrohInstance<T> {
    pub fn data(&self) -> Option<IrohData> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.clone()),
        }
    }

    pub fn endpoint(&self) -> Option<Endpoint> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.endpoint.clone())
        }
    }

    pub fn gossip(&self) -> Option<Gossip> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.gossip.clone())
        }
    }

    pub fn router(&self) -> Option<Router> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.router.clone())
        }
    }

    pub fn topic_id(&self) -> Option<TopicId> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.topic_id)
        }
    }

    pub fn relay_vec(&self) -> Option<Vec<RelayUrl>> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.relay_vec.clone())
        }
    }

    pub fn set_endpoint(self, endpoint: Endpoint) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_endpoint::UserIsEmpty")),
            IrohInstance::Data { iroh_data , data, debug} => {
                let mut new_data = iroh_data;
                new_data.endpoint = endpoint;
                Ok(IrohInstance::Data{iroh_data: new_data, data, debug})
            }
        }
    }

    pub fn set_gossip(self, gossip: Gossip) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_gossip::UserIsEmpty")),
            IrohInstance::Data { iroh_data , data, debug} => {
                let mut new_data = iroh_data;
                new_data.gossip = gossip;
                Ok(IrohInstance::Data{iroh_data: new_data, data, debug})
            }
        }
    }

    pub fn set_router(self, router: Router) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_router::UserIsEmpty")),
            IrohInstance::Data { iroh_data, data , debug} => {
                let mut new_data = iroh_data;
                new_data.router = router;
                Ok(IrohInstance::Data{iroh_data: new_data, data, debug})
            }
        }
    }

    pub fn set_topic_id(self, topic_id: TopicId) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_topic_id::UserIsEmpty")),
            IrohInstance::Data { iroh_data, data , debug} => {
                let mut new_data = iroh_data;
                new_data.topic_id = topic_id;
                Ok(IrohInstance::Data{iroh_data: new_data, data, debug})
            }
        }
    }

    pub fn set_relay_vec(self, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_relay_vec::UserIsEmpty")),
            IrohInstance::Data { iroh_data, data , debug} => {
                let mut new_data = iroh_data;
                new_data.relay_vec = relay_vec;
                Ok(IrohInstance::Data{iroh_data: new_data, data, debug})
            }
        }
    }

    pub fn set_debug(self, debug: bool) -> Result<Self> {
        match self {
            IrohInstance::Empty => Err(anyhow!("set_debug::UserIsEmpty")),
            IrohInstance::Data { iroh_data, data , ..} => {
                Ok(IrohInstance::Data{iroh_data, data, debug})
            }
        }
    }
}