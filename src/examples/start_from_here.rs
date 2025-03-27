#![allow(unused_imports)]
#![allow(dead_code)]

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
use crate::{
    consts::{RELAY_VEC, SEED, TOPIC},
    iroh::{
        gossip::{Message, Sender, SignedMessage}, ConnectOptions, Connection, ServerFuture, User
    },
};
use n0_future::TryStreamExt;

const DEBUG: bool = false;

#[tokio::test]
// run test by using: 'cargo test examples::start_from_here::example -- --exact --nocapture'
async fn example() -> Result<()> {
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
    let sender_clone = sender.clone();
    tokio::spawn(async move { user_loop(sender_clone, receiver).await });

    /* Do somenthing with sender, like: */
    sender.broadcast(&Message::about_me(&user)?).await?;

    // Close everything
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

// This is important, it defines how your app respond to each received Message.
// To add new messages change the message.rs file.
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
                    /* Here you could use iroh-blobs to send the requested image if you have it */
                }
            }
        }
    }
    Ok(())
}
