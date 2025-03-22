use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, NodeAddr, NodeId, PublicKey, RelayUrl, SecretKey};
use iroh_gossip::{net::{Gossip, GossipTopic}, proto::TopicId};

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

impl<T> IrohInstance<T> {
    pub async fn assert_correct_relay(&self) -> Result<()> {
        let endpoint_relay =  self.endpoint_relay().await?;
        let relay = self.relay_url();
        match endpoint_relay == relay {
            true => Ok(()),
            false => Err(anyhow!("instance::assert_correct_relay::EndpointHasDifferentRelay")),
        }
    }
}

// usefull functions from items in IrohData
impl<T> IrohInstance<T> {
    pub fn secret_key(&self) -> Result<Option<SecretKey>> {
        match self {
            IrohInstance::Empty => Ok(None),
            IrohInstance::Data { iroh_data, .. } => {
                let secret_key = iroh_data.endpoint.secret_key().clone();
                Ok(Some(secret_key))
            }
        }
    }

    pub fn public_key(&self) -> Result<Option<PublicKey>> {
        let secret_key = self.secret_key()?;
        match secret_key {
            None => Ok(None),
            Some(secret_key) => Ok(Some(secret_key.public())),
        }
    }

    pub async fn node_addr(&self) -> Result<Option<NodeAddr>> {
        match self {
            IrohInstance::Empty => Ok(None),
            IrohInstance::Data { iroh_data, .. } => {
                let node_addr = iroh_data.endpoint.node_addr().await?;
                Ok(Some(node_addr))
            }
        }
    }

    pub async fn endpoint_relay(&self) -> Result<Option<RelayUrl>> {
        // Get the actual relay at which the endpoint is refering to //TODO: transform in docs
        let node_addr = self.node_addr().await?;
        match node_addr {
            None => Ok(None),
            Some(node_addr) => Ok(node_addr.relay_url)
        }
    }

    pub fn add_node_addr(&self, node_addr: NodeAddr) -> Result<()> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::add_node_addr::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.endpoint.add_node_addr(node_addr)?;
            },
        }
        Ok(())
    }

    pub fn subscribe(&self, node_ids: Vec<NodeId>) -> Result<GossipTopic> {
        match self {
            IrohInstance::Empty => Err(anyhow!("instance::subscribe::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                Ok(iroh_data.gossip.subscribe(iroh_data.topic_id, node_ids)?)
            },
        }
    }

    pub async fn subscribe_and_join(&self, node_ids: Vec<NodeId>) -> Result<GossipTopic> {
        match self {
            IrohInstance::Empty => Err(anyhow!("instance::subscribe_and_join::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                Ok(iroh_data.gossip.subscribe_and_join(iroh_data.topic_id, node_ids).await?)
            },
        }
    }
}



// standard setters and getters
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

    pub fn relay_url(&self) -> Option<RelayUrl> {
        // Get the RelayUrl currently in the IrohData struct //TODO: transform in docs
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { iroh_data , ..} => Some(iroh_data.relay_url.clone())
        }
    }

    pub fn debug(&self) -> bool {
        match self {
            IrohInstance::Empty => false,
            IrohInstance::Data { debug , ..} => *debug
        }
    }

    pub fn set_endpoint(&mut self, endpoint: Endpoint) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_endpoint::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.endpoint = endpoint;
            }
        }
        Ok(self)
    }

    pub fn set_gossip(&mut self, gossip: Gossip) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_gossip::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.gossip = gossip;
            }
        }
        Ok(self)
    }

    pub fn set_router(&mut self, router: Router) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_router::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.router = router;
            }
        }
        Ok(self)
    }

    pub fn set_topic_id(&mut self, topic_id: TopicId) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_topic_id::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.topic_id = topic_id;
            }
        }
        Ok(self)
    }

    pub fn set_relay_url(&mut self, relay_url: RelayUrl) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_relay_url::UserIsEmpty")),
            IrohInstance::Data { iroh_data , ..} => {
                iroh_data.relay_url = relay_url;
            }
        }
        Ok(self)
    }

    pub fn set_debug(&mut self, debug: bool) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("instance::set_debug::UserIsEmpty")),
            IrohInstance::Data { debug: debug_ , ..} => {
                *debug_ = debug;
            }
        }
        Ok(self)
    }
}