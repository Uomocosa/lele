use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, NodeAddr, RelayUrl, SecretKey};
use iroh_gossip::{net::Gossip, proto::TopicId};


#[derive(Debug, Clone)]
pub struct UserData{
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    topic_id: TopicId,
    relay_vec: Vec<RelayUrl>,
}

#[derive(Debug, Clone)]
pub enum User{
    Empty,
    Data { user_data: UserData, debug: bool },
}

// create and close methods
impl User {
    pub fn empty() -> Self {
        User::Empty
    }

    pub async fn create(secret_key: SecretKey, topic_id: TopicId, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let user_data = UserData { endpoint, gossip, router, topic_id, relay_vec };
        Ok(User::Data { user_data, debug: false })
    }

    pub async fn random() -> Result<Self> {
        let secret_key = SecretKey::generate(rand::rngs::OsRng);
        let topic_id = TopicId::from_bytes(rand::random());
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let relay_vec = match endpoint.node_addr().await?.relay_url {
            None => vec![],
            Some(relay_url) => vec![relay_url]
        };
        let user_data = UserData { endpoint, gossip, router, topic_id, relay_vec };
        Ok(User::Data { user_data, debug: false })
    }

    pub async fn close(self) -> Result<()> {
        match self {
            User::Empty => {},
            User::Data { user_data, ..} => {
                user_data.endpoint.close().await;
                user_data.router.shutdown().await?;
            }
        }
        Ok(())
    }
}


// Useful methods
impl User {
    pub async fn search_for_users(&self, user_node_addrs: Vec<NodeAddr>) -> Result<()> {
        for node_addr in &user_node_addrs {
            if node_addr.relay_url.is_none() {
                return Err(anyhow!("search_for_users::NoRelayUrlFound"));
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_search_for_users() {
    
}

// Standard getter and setters
impl User {
    pub fn data(&self) -> Option<UserData> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.clone()),
        }
    }

    pub fn endpoint(&self) -> Option<Endpoint> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.endpoint.clone())
        }
    }

    pub fn gossip(&self) -> Option<Gossip> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.gossip.clone())
        }
    }

    pub fn router(&self) -> Option<Router> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.router.clone())
        }
    }

    pub fn topic_id(&self) -> Option<TopicId> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.topic_id)
        }
    }

    pub fn relay_vec(&self) -> Option<Vec<RelayUrl>> {
        match self {
            User::Empty => None,
            User::Data { user_data , ..} => Some(user_data.relay_vec.clone())
        }
    }

    pub fn set_endpoint(self, endpoint: Endpoint) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_endpoint::UserIsEmpty")),
            User::Data { user_data , debug} => {
                let mut new_data = user_data;
                new_data.endpoint = endpoint;
                Ok(User::Data{user_data: new_data, debug})
            }
        }
    }

    pub fn set_gossip(self, gossip: Gossip) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_gossip::UserIsEmpty")),
            User::Data { user_data , debug} => {
                let mut new_data = user_data;
                new_data.gossip = gossip;
                Ok(User::Data{user_data: new_data, debug})
            }
        }
    }

    pub fn set_router(self, router: Router) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_router::UserIsEmpty")),
            User::Data { user_data , debug} => {
                let mut new_data = user_data;
                new_data.router = router;
                Ok(User::Data{user_data: new_data, debug})
            }
        }
    }

    pub fn set_topic_id(self, topic_id: TopicId) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_topic_id::UserIsEmpty")),
            User::Data { user_data , debug} => {
                let mut new_data = user_data;
                new_data.topic_id = topic_id;
                Ok(User::Data{user_data: new_data, debug})
            }
        }
    }

    pub fn set_relay_vec(self, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_relay_vec::UserIsEmpty")),
            User::Data { user_data , debug} => {
                let mut new_data = user_data;
                new_data.relay_vec = relay_vec;
                Ok(User::Data{user_data: new_data, debug})
            }
        }
    }

    pub fn set_debug(self, debug: bool) -> Result<Self> {
        match self {
            User::Empty => Err(anyhow!("set_debug::UserIsEmpty")),
            User::Data { user_data , ..} => {
                Ok(User::Data{user_data, debug})
            }
        }
    }
}