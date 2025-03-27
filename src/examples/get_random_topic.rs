#![allow(unused_imports)]
#![allow(dead_code)]

use anyhow::Result;
use iroh_gossip::proto::TopicId;

#[test]
// run test by using: 'cargo test examples::get_random_topic::run -- --exact --nocapture'
fn run() -> Result<()> {
    let topic = TopicId::from_bytes(rand::random());
    println!("\npub const TOPIC: &str = {:?};\n", topic.to_string());
    Ok(())
}