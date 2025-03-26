use std::{
    collections::HashMap, str::FromStr, time::{Duration, Instant}
};

use anyhow::{Result, anyhow};
use iroh::{PublicKey, SecretKey};
use iroh_gossip::{net::{Event, GossipEvent, GossipReceiver, GossipSender}, proto::TopicId};
use lele::{consts::{RELAY_VEC, SEED, TOPIC}, iroh::{
    gossip::{Message, SignedMessage}, ConnectOptions, Connection, User
}};
use n0_future::TryStreamExt;

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

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();

    let options = ConnectOptions { debug: true, ..Default::default()};
    let topic_id = TopicId::from_str(TOPIC)?;
    let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
    let user = connection.user;
    let server = connection.server;
    let user_gtopic = connection.user_gossip_topic;
    
    let (gossip_sender, receiver) = user_gtopic.split();
    let sender = Sender::create(&user, gossip_sender)?;

    sender.broadcast(&Message::about_me(&user)?).await?;
    tokio::spawn(async move { user_loop(sender.clone(), receiver).await });

    println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    println!("> online_peers:\n{:?}", user.online_peers()?.keys());
    println!("> closing server ...");
    server.close().await?;
    println!("> closing user ...");
    user.close().await?;
    Ok(())
}

pub async fn user_loop(sender: Sender, mut receiver: GossipReceiver) -> Result<()> {
    let mut usernames: HashMap<_, _> = HashMap::new();
    let name = |from: &PublicKey, hmap: &HashMap<_, _>| {
        hmap.get(from).map_or_else(|| from.fmt_short(), String::to_string)
    };
    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            let (from, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::AboutMe { username } => {
                    usernames.insert(from, username.clone());
                    println!("> {} is now known as {}", from.fmt_short(), name(&from, &usernames));
                }
                Message::SimpleText { text } => {
                    println!("> {}: {}", name(&from, &usernames), text);
                }
                Message::RequestImg { image_name } => {
                    println!("> {} rquested image: {}", name(&from, &usernames), image_name);
                }
                Message::SendNodeId => {}
                Message::RequestNodeId => {
                    sender.broadcast(&Message::SendNodeId).await?;
                    println!("> {} requested node id, sending Message::SendNodeId", name(&from, &usernames));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
// run test by using: '???'
mod tests {
    use super::*;
    use iroh::SecretKey;
    use iroh_gossip::proto::TopicId;
    use rand::{Rng, SeedableRng, rngs::StdRng};

    #[test]
    // run test by using: 'cargo test get_random_seed -- --exact --nocapture'
    fn get_random_seed() -> Result<()> {
        let mut rng = rand::thread_rng();
        let random_seed: [u8; 32] = std::array::from_fn(|_| rng.r#gen());
        println!("const SEED: [u8; 32] = {:?};", random_seed);
        Ok(())
    }

    #[test]
    // run test by using: 'cargo test get_random_topic -- --exact --nocapture'
    fn get_random_topic() -> Result<()> {
        let topic = TopicId::from_bytes(rand::random());
        println!("const TOPIC: &str = {:?};", topic.to_string());
        Ok(())
    }

    #[test]
    fn secret_key_from_seed() -> Result<()> {
        let mut seed = [0u8; 32]; // Create new array with 32 elements
        let mut rng = rand::thread_rng();
        let random_seed: [u8; 28] = std::array::from_fn(|_| rng.r#gen());
        seed[0..4].copy_from_slice(&[192, 168, 0, 0]); // 
        seed[4..32].copy_from_slice(&random_seed); // Copy old values
        let rng = StdRng::from_seed(seed);
        let s1 = SecretKey::generate(rng.clone());
        let s2 = SecretKey::generate(rng.clone());
        assert_eq!(s1.public(), s2.public());
        Ok(())
    }
}
