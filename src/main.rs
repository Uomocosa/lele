use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    
    
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
