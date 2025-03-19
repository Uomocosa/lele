use crate::iroh::gossip::{Message, SignedMessage};
use ::anyhow::Result;
use futures_lite::StreamExt;
use iroh_gossip::net::{Event, GossipEvent, GossipReceiver};
use std::collections::HashMap;

pub async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    // init a peerid -> name hashmap
    let mut names = HashMap::new();
    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            let (from, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::AboutMe { name } => {
                    names.insert(from, name.clone());
                    println!("> {} is now known as {}", from.fmt_short(), name);
                }
                Message::Message { text } => {
                    let name = names
                        .get(&from)
                        .map_or_else(|| from.fmt_short(), String::to_string);
                    println!("{}: {}", name, text);
                }
            }
        }
    }
    Ok(())
}
