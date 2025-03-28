use crate::{iroh::get_server_addresses, string::random_string};
use anyhow::{Ok, Result, anyhow};
use iroh::{Endpoint, NodeAddr, NodeId, RelayUrl, SecretKey, protocol::Router};
use iroh_gossip::{
    net::{Gossip, GossipTopic},
    proto::TopicId,
};
// use tokio::task::JoinHandle;

use super::{IrohData, IrohInstance};

#[derive(Debug, Clone)]
pub struct UserData {
    name: String,
}

pub type User = IrohInstance<UserData>;
pub type GossipFuture = tokio::task::JoinHandle<anyhow::Result<GossipTopic>>;

impl User {
    pub async fn create(
        secret_key: SecretKey,
        topic_id: TopicId,
        relay_url: RelayUrl,
        name: &str,
    ) -> Result<Self> {
        // Maybe in the future I'll add a proper discovery mechanism.
        // I would also like to integrate the Dht discovery mechanism.
        // let discovery = ConcurrentDiscovery::from_services(vec![
        //     Box::new(PkarrPublisher::n0_dns(secret_key.clone())),
        //     Box::new(DnsDiscovery::n0_dns()),
        //     Box::new(dht),
        //     Box::new(LocalSwarmDiscovery::new(secret_key.public())?),
        // ]);
        // let endpoint = Endpoint::builder()
        //     .secret_key(secret_key)
        //     .discovery(Box::new(discovery))
        //     .bind()
        //     .await?;
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .discovery_local_network()
            .bind()
            .await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn()
            .await?;
        let iroh_data = IrohData {
            endpoint,
            gossip,
            router,
            topic_id,
            relay_url,
        };
        let data = UserData {
            name: name.to_string(),
        };
        Ok(User::Data {
            iroh_data,
            data,
            debug: false,
        })
    }

    pub async fn recreate(self) -> Result<Self> {
        match self {
            IrohInstance::Empty => Ok(self),
            IrohInstance::Data {
                iroh_data,
                data,
                debug,
            } => {
                let secret_key = iroh_data.endpoint.secret_key().clone();
                let topic_id = iroh_data.topic_id;
                let relay_url = iroh_data.relay_url;
                let name = &data.name;
                let mut new_user = User::create(secret_key, topic_id, relay_url, name).await?;
                new_user.set_debug(debug)?;
                Ok(new_user)
            }
        }
    }

    pub async fn random_with_topic(topic_id: TopicId) -> Result<Self> {
        let secret_key = SecretKey::generate(rand::rngs::OsRng);
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
        let iroh_data = IrohData {
            endpoint,
            gossip,
            router,
            topic_id,
            relay_url,
        };
        let name = "user_".to_string() + &random_string(7);
        let data = UserData { name };
        Ok(User::Data {
            iroh_data,
            data,
            debug: false,
        })
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
        let relay_url = match endpoint.node_addr().await?.relay_url {
            None => return Err(anyhow!("user::random::NoRelayUrlFound")),
            Some(relay_url) => relay_url,
        };
        let iroh_data = IrohData {
            endpoint,
            gossip,
            router,
            topic_id,
            relay_url,
        };
        let name = "user_".to_string() + &random_string(7);
        let data = UserData { name };
        Ok(User::Data {
            iroh_data,
            data,
            debug: false,
        })
    }
}

// Useful methods
impl User {
    pub async fn add_node_addresses(&self, node_addrs: &Vec<NodeAddr>) -> Result<()> {
        self.assert_correct_relay().await?;
        let debug = self.debug();
        for node_addr in node_addrs {
            if node_addr.relay_url.is_none() {
                if debug {
                    println!("> found empty node_addr");
                }
                continue;
            }
            if node_addr.relay_url != self.relay_url() {
                if debug {
                    println!(
                        "> NodeArr has a different RelayUrl than this User:\n>\t{:?}\n>\t{:?}",
                        node_addr.relay_url.clone().unwrap(),
                        self.relay_url()
                    );
                }
                continue;
            }
            self.add_node_addr(node_addr.clone())?;
        }
        Ok(())
    }

    pub async fn connect_to_servers(&self, server_addrs: Vec<NodeAddr>) -> Result<GossipFuture> {
        self.add_node_addresses(&server_addrs).await?;
        let iroh_data_clone = match self.iroh_data().clone() {
            None => return Err(anyhow!("user::connect_to_servers::NoIrohDataFound")),
            Some(iroh_data) => iroh_data,
        };
        let node_ids: Vec<NodeId> = server_addrs.iter().map(|addr| addr.node_id).collect();
        let (debug, name) = (self.debug(), self.name().unwrap());
        let user_handle: GossipFuture = tokio::spawn(async move {
            if debug {
                println!("> {name}: connecting to servers ...");
            }
            let user_gtopic = iroh_data_clone
                .gossip
                .subscribe_and_join(iroh_data_clone.topic_id, node_ids)
                .await?;
            // let user_gtopic = user_clone.subscribe_and_join(node_ids).await?;
            if debug {
                println!("> {name}: connected!");
            }
            anyhow::Ok(user_gtopic)
        });
        Ok(user_handle)
    }

    pub async fn users_online(&self, relay_vec: &[&str], seed: &[u8; 32]) -> Result<Vec<NodeId>> {
        let mut peer_ids: Vec<NodeId> = self.online_peers()?.keys().cloned().collect();
        let server_ids: Vec<u64> = (0..100).collect();
        let relay_vec: Vec<String> = relay_vec.iter().map(|s| s.to_string()).collect();
        let only_server_ids: Vec<NodeId> = get_server_addresses(&server_ids, &relay_vec, seed)?
            .iter()
            .map(|addr| addr.node_id)
            .collect();
        if self.debug() {
            println!("> 'all' peer_ids: {:?}", peer_ids);
        }

        peer_ids.retain(|peer| !only_server_ids.contains(peer));
        if self.debug() {
            println!("> 'non-server' peer_ids: {:?}", peer_ids);
        }
        if self.debug() {
            println!("> user.node_id(): {:?}", self.node_id());
        }
        Ok(peer_ids)
    }

    pub async fn is_any_other_user_online(
        &self,
        relay_vec: &[&str],
        seed: &[u8; 32],
    ) -> Result<bool> {
        let users_vec = self.users_online(relay_vec, seed).await?;
        Ok(!users_vec.is_empty())
    }
}

// setters and getters
impl User {
    pub fn name(&self) -> Option<String> {
        match self {
            IrohInstance::Empty => None,
            IrohInstance::Data { data, .. } => Some(data.name.clone()),
        }
    }

    pub fn set_name(&mut self, name: String) -> Result<&mut Self> {
        match self {
            IrohInstance::Empty => return Err(anyhow!("user::set_name::UserIsEmpty")),
            IrohInstance::Data { data, .. } => {
                data.name = name;
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use iroh::NodeId;
    use std::{str::FromStr, time::Duration};

    use super::*;
    use crate::{
        consts::{RELAY_VEC, SEED, TOPIC},
        iroh::{ConnectOptions, Connection, Server, get_server_addresses},
    };

    #[tokio::test]
    // run test by using: 'cargo test iroh::user::tests::subscribe_to_node_addreses -- --exact --nocapture'
    async fn connect_and_boradcast() -> Result<()> {
        tracing_subscriber::fmt::init();
        let options = ConnectOptions {
            debug: true,
            ..Default::default()
        };
        let topic_id = TopicId::from_str(TOPIC)?;
        let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
        let user = connection.user;
        let server_future = connection.server_future;
        let user_gtopic = connection.user_gossip_topic;

        let (sender, _receiver) = user_gtopic.split();
        sender.broadcast(Bytes::from("Hello!")).await?;

        tokio::time::sleep(Duration::from_millis(250)).await;
        println!("> press Ctrl+C to exit.");
        tokio::signal::ctrl_c().await?;
        println!("> online_peers:\n{:?}", user.online_peers()?.keys());
        println!("> closing server ...");
        server_future.close().await?;
        println!("> closing user ...");
        user.close().await?;
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test iroh::user::tests::subscribe_to_node_addreses -- --exact --nocapture'
    async fn subscribe_to_node_addreses() -> Result<()> {
        tracing_subscriber::fmt::init();
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

        tokio::time::sleep(Duration::from_millis(500)).await;

        // User side
        println!("> creating user ...");
        let mut user = User::random_with_topic(topic_id).await?;
        user.set_debug(true)?;
        let id_vec: Vec<u64> = (0..10).collect();
        let relay_vec: Vec<String> = RELAY_VEC.iter().map(|s| s.to_string()).collect();
        let server_addrs = get_server_addresses(&id_vec, &relay_vec, &SEED)?;
        // println!("> server_addrs:\n{:#?}", server_addrs);
        user.add_node_addresses(&server_addrs).await?;
        let node_ids: Vec<NodeId> = server_addrs.iter().map(|addr| addr.node_id).collect();
        let mut user_gtopic = user.subscribe_and_join(node_ids).await?;
        user_gtopic.joined().await?;
        println!("> user has joined a server!");
        println!("> you can now send messages via user_gtopic!");

        println!("> online_peers:\n{:?}", user.online_peers()?.keys());
        println!("> closing server ...");
        server.close().await?;
        println!("> closing user ...");
        user.close().await?;
        Ok(())
    }

    #[tokio::test]
    // run test by using: 'cargo test iroh::user::tests::complex_example -- --exact --nocapture'
    async fn complex_example() -> Result<()> {
        // This shows a really simple implemntation of inner mecchanics of 'connect()'
        tracing_subscriber::fmt::init();
        let topic_id = TopicId::from_str(TOPIC)?;
        let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;

        // Server side
        println!("> creating server ...");
        let id = 0;
        let server = Server::create(id, topic_id, relay_url, &SEED).await?;
        let mut server_gtopic = server.subscribe(vec![])?;
        tokio::spawn(async move { server_gtopic.joined().await });
        println!("> server is waiting for user to find it...");

        tokio::time::sleep(Duration::from_millis(500)).await;

        // User side
        println!("> creating user ...");
        let mut user = User::random_with_topic(topic_id).await?;
        user.set_debug(true)?;
        let id_vec: Vec<u64> = vec![id];
        let relay_vec: Vec<String> = RELAY_VEC.iter().map(|s| s.to_string()).collect();
        let server_addrs = get_server_addresses(&id_vec, &relay_vec, &SEED)?;
        let user_handle = user.connect_to_servers(server_addrs).await?;
        let user_gtopic = user_handle.await??;
        let (sender, _receiver) = user_gtopic.split();

        sender.broadcast(Bytes::from("Hello!")).await?;
        println!("> user.node_id(): {:?}", user.node_id());
        println!("> online_peers:\n{:?}", user.online_peers()?.keys());

        println!("> closing server ...");
        server.close().await?;
        println!("> closing user ...");
        user.close().await?;
        Ok(())
    }
}
