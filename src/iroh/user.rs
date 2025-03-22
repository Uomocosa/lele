use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, NodeAddr, RelayUrl, SecretKey};
use iroh_gossip::{net::Gossip, proto::TopicId};
use super::{IrohData, IrohInstance};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UserData {
    pub name: String
}

pub type User = IrohInstance<UserData>;



impl User {
    pub async fn create(secret_key: SecretKey, topic_id: TopicId, relay_vec: Vec<RelayUrl>) -> Result<Self> {
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_vec };
        let data = UserData { name: "???".to_string() };
        Ok(User::Data { iroh_data, data, debug: false })
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
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_vec };
        let data = UserData { name: "???".to_string() };
        Ok(User::Data { iroh_data, data, debug: false })
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
