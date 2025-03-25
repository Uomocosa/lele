use iroh::SecretKey;
use rand::{Rng, SeedableRng, rngs::StdRng};

pub fn generate_server_secret_key(id: u64, seed: &[u8; 32]) -> SecretKey {
    let mut rng = StdRng::from_seed(*seed);
    let mut new_seed = [0_u8; 32];
    let first_part: [u8; 24] = rng.r#gen();
    let second_part: [u8; 8] = id.to_le_bytes();
    new_seed[..24].clone_from_slice(&first_part);
    new_seed[24..].clone_from_slice(&second_part);
    rng = StdRng::from_seed(new_seed);
    SecretKey::generate(rng)
}
