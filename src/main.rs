use std::{io::{self, Write}, str::FromStr, time::{Duration, Instant}};

use anyhow::Result;
use iroh::RelayUrl;
use iroh_gossip::proto::TopicId;
use lele::{consts::{RELAY_VEC, SEED, TOPIC}, iroh::{get_server_addresses, Server, User}};

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();
    // let start = Instant::now();
    let topic_id = TopicId::from_str(TOPIC)?;
    let relay_url = RelayUrl::from_str(RELAY_VEC[0])?;
    let mut id = 0;

    

    // User side
    println!("> creating user ...");
    let mut user = User::random(topic_id).await?;
    user.set_debug(true)?;
    let id_vec: Vec<u64> = vec![id];
    let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
    // println!("> server_addrs:\n{:#?}", server_addrs);
    let mut user_gtopic = user.subscribe_to_node_addreses(server_addrs.clone()).await?;
    let user_handle = tokio::spawn(
        async move { 
            user_gtopic.joined().await?;
            println!("\n> user: connected!");
            anyhow::Ok(user_gtopic)
        }
    );
    let joined_timer = Instant::now();
    print!("> searching for servers "); io::stdout().flush()?;
    while joined_timer.elapsed() <= Duration::from_secs(2) {
        print!("."); io::stdout().flush()?;
        if user_handle.is_finished() {break;}
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    println!();
    match user_handle.is_finished() {
        true => {
            println!("> user has joined a server!");
            id += 1
        },
        false => {
            println!("> server Id({}) is not yet established ;;", id);
            user_handle.abort();
        },
    }



    // Server side
    println!("> creating server ...");
    let server = Server::create(id, topic_id, relay_url, &SEED).await?;
    let mut server_gtopic = server.subscribe(vec![])?;
    tokio::spawn(
        async move { 
            server_gtopic.joined().await?;
            println!("> server: connected!");
            anyhow::Ok(server_gtopic)
        }
    );
    println!("> server is waiting for user to find it ...");
    println!("> server_addr:\n{:#?}", server.node_addr().await?);

    
    println!("> waiting for user to find at least your own server ...");
    user.close().await?;
    println!("> re-creating user ...");
    let mut user = User::random(topic_id).await?;
    user.set_debug(true)?;
    let id_vec: Vec<u64> = vec![id];
    let server_addrs = get_server_addresses(&id_vec, RELAY_VEC, &SEED).await?;
    // println!("> server_addrs:\n{:#?}", server_addrs);
    let mut user_gtopic = user.subscribe_to_node_addreses(server_addrs.clone()).await?;
    let user_handle = tokio::spawn(
        async move { 
            user_gtopic.joined().await?;
            println!("\n> user: connected!");
            anyhow::Ok(user_gtopic)
        }
    );
    let joined_timer = Instant::now();
    print!("> searching for servers "); io::stdout().flush()?;
    while joined_timer.elapsed() <= Duration::from_secs(15) {
        print!("."); io::stdout().flush()?;
        if user_handle.is_finished() {break;}
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    println!();
    match user_handle.is_finished() {
        true => {
            println!("> user has joined a server!");
            // id += 1
        },
        false => {
            println!("> server Id({}) is not yet established ;;", id);
            user_handle.abort();
        },
    }
    

    // println!("> finished [{:?}]", start.elapsed());
    tokio::time::sleep(Duration::from_millis(250)).await;
    println!("> press Ctrl+C to exit.");  tokio::signal::ctrl_c().await?;
    println!("> closing server ..."); server.close().await?;
    println!("> closing user ..."); user.close().await?;
    Ok(())
}

#[cfg(test)]
// run test by using: '???'
mod tests {
    use iroh::SecretKey;
    use iroh_gossip::proto::TopicId;
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use super::*;

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
