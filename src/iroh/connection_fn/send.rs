use crate::iroh::{
    Connection,
    gossip::{Message, SignedMessage},
};
use anyhow::{Result, anyhow};
use iroh_gossip::net::GossipSender;

pub async fn send<'a>(
    connection: &'a mut Connection,
    sender: GossipSender,
    text: &str,
) -> Result<&'a mut Connection> {
    if connection.endpoint.is_none() {
        return Err(anyhow!("EndpointNotFound"));
    }
    let endpoint = connection.endpoint.clone().unwrap();
    let message = Message::Message {
        text: text.to_string(),
    };
    let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
    sender.broadcast(encoded_message).await?;
    Ok(connection)
}
