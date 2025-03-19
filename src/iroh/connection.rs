use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
};

use anyhow::{Result, anyhow};

use crate::iroh::connection_fn as cfn;
use crate::iroh::gossip::Command;
use iroh::{Endpoint, NodeAddr, PublicKey, RelayUrl, SecretKey, protocol::Router};
use iroh_gossip::{
    net::{Gossip, GossipReceiver, GossipSender},
    proto::TopicId,
};

use super::gossip::Ticket;

// use super::gossip::{Message, SignedMessage, Ticket};

#[derive(Debug, Clone)]
pub struct Connection {
    pub command: Command,
    pub secret_key: Option<SecretKey>,
    pub no_relay: bool,
    pub relay: Option<RelayUrl>,
    pub ipv4: Ipv4Addr,
    pub port: u16,
    pub name: Option<String>,

    pub endpoint: Option<Endpoint>,
    pub gossip: Option<Gossip>,
    pub router: Option<Router>,
    // pub sender: Option<GossipSender>,
    // pub receiver: Option<GossipReceiver>,
    pub ticket: Option<Ticket>,
    pub topic: Option<TopicId>,
    pub peers: Vec<NodeAddr>,

    // pub peers: Vec<PublicKey>,
    pub debug: bool,
}

#[rustfmt::skip] // Not the best, but it works
// #[rustfmt::fn_single_line] // This should be set to EACH function in this block ;; AND it doesn't even work
impl Connection {
    pub fn _empty() -> Self { cfn::empty() }



    pub async fn _open(&mut self) -> Result<&mut Self> { cfn::open(self).await }
    pub async fn _join(&mut self, ticket: String) -> Result<&mut Self> { cfn::join(self, ticket).await }

    pub async fn _send(&mut self, sender: GossipSender, text: &str) -> Result<&mut Self> { cfn::send(self, sender, text).await }

    pub async fn _establish_known_connection(self, ticket: Ticket) -> Result<()> { cfn::establish_known_connection(self, ticket).await }

    pub fn _recive() {}

    pub async fn subscribe(self) -> Result<(GossipSender, GossipReceiver)> { cfn::subscribe(self).await }
    pub async fn subscribe_and_join(self) -> Result<(GossipSender, GossipReceiver)> { cfn::subscribe_and_join(self).await }

    pub async fn connect_to_peers(self) -> Result<(GossipSender, GossipReceiver)> { cfn::connect_to_peers(self).await }
    pub async fn bind_endpoint(&mut self) -> Result<&mut Self> { cfn::bind_endpoint(self).await }

    pub async fn create_ticket(&mut self) -> Result<Ticket> { cfn::create_ticket(self).await }
    pub async fn spawn_gossip(&mut self) -> Result<&mut Self> { cfn::spawn_gossip(self).await }
    pub async fn spawn_router(&mut self) -> Result<&mut Self> { cfn::spawn_router(self).await }

    pub fn secret_key(&self) -> Result<&SecretKey> {
        if self.secret_key.is_none() { 
            return Err(anyhow!("Connection::secret_key::SecretKeyNotFound")); 
        }
        Ok(self.secret_key.as_ref().unwrap())
    }

    pub fn public_key(&self) -> Result<PublicKey> {
        if self.secret_key.is_none() { 
            return Err(anyhow!("Connection::public_key::SecretKeyNotFound")); 
        }
        Ok(self.secret_key.as_ref().unwrap().public())
    }

    pub async fn handle_iroh_connection(&mut self, iroh_connection: iroh::endpoint::Connection) -> Result<&mut Self> { 
        cfn::handle_iroh_connection(self, iroh_connection).await 
    }
    pub async fn handle_iroh_connections(self, iroh_connections: Vec<iroh::endpoint::Connection>) -> Result<(GossipSender, GossipReceiver)> { 
        cfn::handle_iroh_connections(self, iroh_connections).await 
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn set_secret_key(&mut self, secret_key: SecretKey) -> &mut Self {
        self.secret_key = Some(secret_key);
        self
    }

    pub fn set_topic(&mut self, topic: &str) -> &mut Self {
        let topic_id = TopicId::from_str(topic)
            .unwrap_or_else(|_| panic!(
                "Cannot create topic form str: '{}'\nThe str should be somenthing like: 'aa17cebe62afa7cc2315e46c943f47a414acfe4db6155d55dbb50d2b257eb12c'", 
                topic
            ));
        self.topic = Some(topic_id);
        self
    }

    pub fn set_ip(&mut self, ip: Ipv4Addr) -> &mut Self {
        self.ipv4 = ip;
        self
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn set_socket_addr(&mut self, socket_addr: SocketAddrV4) -> &mut Self {
        self.ipv4 = *socket_addr.ip();
        self.port = socket_addr.port();
        self
    }

    pub async fn node_addr(&self) -> Result<NodeAddr> {
        if self.endpoint.is_none() { return Err(anyhow!("Connection::node_addr::EndpointNotFound")); }
        let endpoint = self.endpoint.clone().unwrap();
        endpoint.node_addr().await
    }

    pub fn socket_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ipv4, self.port)
    }

    pub async fn socket_addresses(&self) -> Result<Vec<SocketAddrV4>> {
        if self.endpoint.is_none() { return Ok(vec![self.socket_addr()]); }
        let endpoint = self.endpoint.clone().unwrap();
        let node_addr = endpoint.node_addr().await?;
        let socket_addresses: Vec<&SocketAddr> =  node_addr.direct_addresses().collect();
        let mut out_vec: Vec<SocketAddrV4> = Vec::new();
        for &addr in socket_addresses {
            match addr.ip() {
                IpAddr::V4(ipv4) => out_vec.push(SocketAddrV4::new(ipv4, addr.port())),
                IpAddr::V6(_) => continue,
            }
        };
        Ok(out_vec)
    }

    pub async fn ip(&self) -> Ipv4Addr {
        self.ipv4
    }

    pub async fn ips(&self) -> Result<Vec<Ipv4Addr>> {
        let addresses = self.socket_addresses().await?;
        let ips: Vec<Ipv4Addr> = addresses.iter().map(|a| *a.ip()).collect();
        Ok(ips)
    }

    pub async fn port(&self) -> u16 {
        self.port
    }

    pub async fn ports(&self) -> Result<Vec<u16>> {
        let addresses = self.socket_addresses().await?;
        let ports: Vec<u16> = addresses.iter().map(|a| a.port()).collect();
        Ok(ports)
    }

    pub fn is_empty(&self) -> bool {
        if self.secret_key.is_some() { return false; }
        if self.relay.is_some() { return false; }
        if self.name.is_some() { return false; }
        if self.ticket.is_some() { return false; }
        if self.topic.is_some() { return false; }
        if !self.peers.is_empty() { return false; }
        if self.endpoint.is_some() { return false; }
        if self.endpoint.is_some() { return false; }
        if self.gossip.is_some() { return false; }
        if self.router.is_some() { return false; }
        if self.port != 0 { return false; }
        if self.ipv4 != Ipv4Addr::UNSPECIFIED { return false; }
        true
    }

    pub async fn close(self) -> Result<()> {
        self.endpoint.unwrap().close().await;
        self.router.unwrap().shutdown().await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "prints on screen"]
    fn test_1() {
        let connection = Connection::_empty();
        println!("{:#?}", connection);
    }
}
