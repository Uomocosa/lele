#![allow(unused_imports)]
#![allow(dead_code)]

use anyhow::Result;
use iroh::SecretKey;
use rand::{rngs::StdRng, Rng, SeedableRng};


#[test]
// run test by using: 'cargo test examples::secret_key_from_seed::run -- --exact --nocapture'
fn run() -> Result<()> {
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