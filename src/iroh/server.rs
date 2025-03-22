use anyhow::Result;
use iroh::{protocol::Router, Endpoint, RelayUrl};
use iroh_gossip::{net::Gossip, proto::TopicId};
use super::{generate_server_secret_key, IrohData, IrohInstance};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ServerData { id: u64 }

pub type Server = IrohInstance<ServerData>;



// create and close methods
impl Server {
    pub async fn create(id: u64, topic_id: TopicId, relay_url: RelayUrl, seed: &[u8; 32]) -> Result<Self> {
        let secret_key = generate_server_secret_key(id, seed);
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_url };
        let data = ServerData { id };
        Ok(Server::Data { iroh_data, data, debug: false })
    }
}


#[cfg(test)]
mod tests{
    use std::str::FromStr;
    use crate::{consts::{RELAY_VEC, SEED, TOPIC}, iroh::Server};
    use super::*;

    #[tokio::test]
    // run test by using: 'cargo test iroh::server::tests::same_args_same_server -- --exact --nocapture'
    async fn same_args_same_server() -> Result<()> {
        let topic_id = TopicId::from_str(TOPIC)?;
        let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;
        let id_1 = 0;
        let id_2 = 1;
        let server1 = Server::create(id_1, topic_id, relay_url.clone(), &SEED).await?;
        let server1_copy = Server::create(id_1, topic_id, relay_url.clone(), &SEED).await?;
        let server2 = Server::create(id_2, topic_id, relay_url.clone(), &SEED).await?;
        println!("server1.public_key()?: {:?}", server1.public_key()?);
        println!("server2.public_key()?: {:?}", server2.public_key()?);

        assert_eq!(server1.public_key()?, server1_copy.public_key()?);
        assert_eq!(server1.endpoint_relay().await?, server1_copy.endpoint_relay().await?);
        assert_ne!(server1.public_key()?, server2.public_key()?);
        Ok(())
    }
}
