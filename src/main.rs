use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use iroh::SecretKey;
use iroh_gossip::{
    net::{Event, GossipEvent, GossipReceiver, GossipSender, GossipTopic},
    proto::TopicId,
};
use lele::{
    consts::{RELAY_VEC, SEED, TOPIC},
    iroh::{
        gossip::{Message, SignedMessage}, ConnectOptions, Connection, ServerFuture, User
    },
};
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

const DEBUG: bool = false;

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();
    let start = Instant::now();

    let options = ConnectOptions {
        debug: DEBUG,
        ..Default::default()
    };
    let topic_id = TopicId::from_str(TOPIC)?;
    let connection = Connection::create_with_opts(topic_id, RELAY_VEC, &SEED, options).await?;
    let user: User = connection.user;
    let server_future: ServerFuture = connection.server_future;
    let user_gtopic: GossipTopic = connection.user_gossip_topic;

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
    server_future.close().await?;
    println!("> closing user ...");
    user.close().await?;
    Ok(())
}

pub async fn user_loop(sender: Sender, mut receiver: GossipReceiver) -> Result<()> {
    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            let (from, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::AboutMe { username } => {
                    let msg = format!("hello {}!", &username);
                    sender.broadcast(&Message::text(&msg)).await?;
                }
                Message::SimpleText { text } => {
                    println!("> {}: {}", from.fmt_short(), text);
                }
                Message::RequestImg { image_name } => {
                    println!(
                        "> {} rquested image: {}",
                        from.fmt_short(),
                        image_name
                    );
                }
            }
        }
    }
    Ok(())
}
