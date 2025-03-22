use anyhow::{anyhow, Result};
use iroh::{protocol::Router, Endpoint, NodeAddr, NodeId, RelayUrl, SecretKey};
use iroh_gossip::{net::{Gossip, GossipTopic}, proto::TopicId};
use crate::string::random_string;

use super::{IrohData, IrohInstance};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UserData {
    pub name: String
}

pub type User = IrohInstance<UserData>;



impl User {
    pub async fn create(secret_key: SecretKey, topic_id: TopicId, relay_url: RelayUrl, name: &str) -> Result<Self> {
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_url };
        let data = UserData { name: name.to_string() };
        Ok(User::Data { iroh_data, data, debug: false })
    }

    pub async fn random(topic_id: TopicId) -> Result<Self> {
        let secret_key = SecretKey::generate(rand::rngs::OsRng);
        // let topic_id = TopicId::from_bytes(rand::random());
        let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let relay_url = match endpoint.node_addr().await?.relay_url {
            None => return Err(anyhow!("user::random::NoRelayUrlFound")),
            Some(relay_url) => relay_url,
        };
        let iroh_data = IrohData { endpoint, gossip, router, topic_id, relay_url };
        let name = "user_".to_string() + &random_string(7);
        let data = UserData { name };
        Ok(User::Data { iroh_data, data, debug: false })
    }
}

// Useful methods
impl User {
    pub async fn subscribe_to_node_addreses(&self, node_addrs: Vec<NodeAddr>) -> Result<GossipTopic> {
        self.assert_correct_relay().await?;
        let debug = self.debug();
        for node_addr in &node_addrs {
            if node_addr.relay_url.is_none() { 
                if debug { println!("> found empty node_addr"); }
                continue; 
            }
            if node_addr.relay_url != self.relay_url() { 
                if debug { println!(
                        "> NodeArr has a different RelayUrl than this User:\n>\t{:?}\n>\t{:?}",
                        node_addr.relay_url.clone().unwrap(), self.relay_url()
                    ); 
                }
                continue; 
            }
            self.add_node_addr(node_addr.clone())?;
        }
        let node_ids: Vec<NodeId> = node_addrs.iter().map(|addr| addr.node_id).collect();
        self.subscribe(node_ids)
    }
}

#[cfg(test)]
mod tests{
    use std::str::FromStr;
    use crate::{consts::{RELAY_VEC, SEED, TOPIC}, iroh::{get_server_addresses, Server}};
    use super::*;

    #[tokio::test]
    // run test by using: 'cargo test iroh::user::tests::subscribe_to_node_addreses -- --exact --nocapture'
    async fn subscribe_to_node_addreses() -> Result<()> {
        // tracing_subscriber::fmt::init();
        let topic_id = TopicId::from_str(TOPIC)?;
        let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;

        // Server side
        println!("> creating server ...");
        let id = 0;
        let server = Server::create(id, topic_id, relay_url, &SEED).await?;
        let mut server_gtopic = server.subscribe(vec![])?;
        tokio::spawn(async move { server_gtopic.joined().await });
        println!("> server is waiting for user to find it...");
        // println!("> server_addr:\n{:#?}", server.node_addr().await?);
        
        // User side
        println!("> creating user ...");
        let mut user = User::random(topic_id).await?;
        user.set_debug(true)?;
        let id_vec: Vec<u64> = (0..10).collect();
        let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
        // println!("> server_addrs:\n{:#?}", server_addrs);
        let mut user_gtopic = user.subscribe_to_node_addreses(server_addrs).await?;
        user_gtopic.joined().await?;
        println!("> user has joined a server!");
        println!("> you can now send messages via user_gtopic!");
        
        Ok(())
    }
}
