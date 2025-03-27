use anyhow::{anyhow, Result};
use iroh::SecretKey;
use iroh_gossip::net::GossipSender;

use crate::iroh::{gossip::{Message, SignedMessage}, User};

#[derive(Debug, Clone)]
pub struct Sender {
    // user: &'a User,
    secret_key: SecretKey,
    gossip_sender: GossipSender,
}

impl Sender {
    pub fn create(user: &User, gossip_sender: GossipSender) -> Result<Self> {
        match user {
            User::Empty => return Err(anyhow!("todo::create::UserIsEmpty")),
            User::Data { .. } => {}
        }
        let secret_key = match user.secret_key()? {
            None => return Err(anyhow!("todo::broadcast::SecretKeyNotFound")),
            Some(secret_key) => secret_key,
        };
        Ok(Sender {
            secret_key,
            gossip_sender,
        })
    }

    pub async fn broadcast(&self, message: &Message) -> Result<()> {
        let encoded_message = SignedMessage::sign_and_encode(&self.secret_key, message)?;
        self.gossip_sender.broadcast(encoded_message).await?;
        Ok(())
    }
}