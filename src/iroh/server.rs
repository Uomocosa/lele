use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, RelayUrl, SecretKey};
use iroh_gossip::{net::Gossip, proto::TopicId};
use rand::{rngs::StdRng, SeedableRng};


#[derive(Debug, Clone)]
pub struct ServerData{
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    topic_id: TopicId,
    relay_vec: Vec<RelayUrl>,
}

#[derive(Debug, Clone)]
pub enum Server{
    Empty,
    Data { server_data: ServerData, debug: bool },
}

// create and close methods
impl Server {
    pub fn empty() -> Self {
        Server::Empty
    }

    pub async fn create(topic_id: TopicId, relay_vec: Vec<RelayUrl>, seed: &[u8; 32]) -> Result<Self> {
        let rng = StdRng::from_seed(*seed);
        let secret_key = SecretKey::generate(rng);
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let server_data = ServerData { endpoint, gossip, router, topic_id, relay_vec };
        Ok(Server::Data { server_data, debug: false})
    }

    pub async fn close(self) -> Result<()> {
        match self {
            Server::Empty => {},
            Server::Data { server_data, ..} => {
                server_data.endpoint.close().await;
                server_data.router.shutdown().await?;
            }
        }
        Ok(())
    }
}


// Standard getter and setters
impl Server {
    pub fn data(&self) -> Option<ServerData> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.clone()),
        }
    }

    pub fn endpoint(&self) -> Option<Endpoint> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.endpoint.clone())
        }
    }

    pub fn gossip(&self) -> Option<Gossip> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.gossip.clone())
        }
    }

    pub fn router(&self) -> Option<Router> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.router.clone())
        }
    }

    pub fn topic_id(&self) -> Option<TopicId> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.topic_id)
        }
    }

    pub fn relay_vec(&self) -> Option<Vec<RelayUrl>> {
        match self {
            Server::Empty => None,
            Server::Data { server_data , ..} => Some(server_data.relay_vec.clone())
        }
    }

    pub fn set_endpoint(self, endpoint: Endpoint) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_endpoint::ServerIsEmpty")),
            Server::Data { server_data , debug} => {
                let mut new_data = server_data;
                new_data.endpoint = endpoint;
                Ok(Server::Data{server_data: new_data, debug})
            }
        }
    }

    pub fn set_gossip(self, gossip: Gossip) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_gossip::ServerIsEmpty")),
            Server::Data { server_data , debug} => {
                let mut new_data = server_data;
                new_data.gossip = gossip;
                Ok(Server::Data{server_data: new_data, debug})
            }
        }
    }

    pub fn set_router(self, router: Router) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_router::ServerIsEmpty")),
            Server::Data { server_data , debug} => {
                let mut new_data = server_data;
                new_data.router = router;
                Ok(Server::Data{server_data: new_data, debug})
            }
        }
    }

    pub fn set_topic_id(self, topic_id: TopicId) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_topic_id::ServerIsEmpty")),
            Server::Data { server_data , debug} => {
                let mut new_data = server_data;
                new_data.topic_id = topic_id;
                Ok(Server::Data{server_data: new_data, debug})
            }
        }
    }

    pub fn set_relay_vec(self, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_relay_vec::ServerIsEmpty")),
            Server::Data { server_data , debug} => {
                let mut new_data = server_data;
                new_data.relay_vec = relay_vec;
                Ok(Server::Data{server_data: new_data, debug})
            }
        }
    }

    pub fn set_debug(self, debug: bool) -> Result<Self> {
        match self {
            Server::Empty => Err(anyhow!("set_debug::UserIsEmpty")),
            Server::Data { server_data , ..} => {
                Ok(Server::Data{server_data, debug})
            }
        }
    }
}