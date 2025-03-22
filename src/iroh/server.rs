use anyhow::Result;
use iroh::{protocol::Router, Endpoint, RelayUrl, SecretKey};
use iroh_gossip::{net::Gossip, proto::TopicId};
use rand::{rngs::StdRng, SeedableRng};
use super::{IrohData, IrohInstance};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ServerData {}

pub type Server = IrohInstance<ServerData>;



// create and close methods
impl Server {
    pub async fn create(topic_id: TopicId, relay_vec: Vec<RelayUrl>, seed: &[u8; 32]) -> Result<Self> {
        let rng = StdRng::from_seed(*seed);
        let secret_key = SecretKey::generate(rng);
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_vec };
        let data = ServerData {};
        Ok(Server::Data { iroh_data, data, debug: false })
    }
}