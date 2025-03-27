#![allow(unused_imports)]
#![allow(dead_code)]

use anyhow::Result;
use rand::Rng;

#[test]
// run test by using: 'cargo test examples::get_random_seed::run -- --exact --nocapture'
fn run() -> Result<()> {
    let mut rng = rand::thread_rng();
    let random_seed: [u8; 32] = std::array::from_fn(|_| rng.r#gen());
    println!("const SEED: [u8; 32] = {:?};", random_seed);
    Ok(())
}