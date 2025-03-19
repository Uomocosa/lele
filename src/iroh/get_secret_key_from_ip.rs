use std::net::Ipv4Addr;

use iroh::SecretKey;
use rand::{SeedableRng, rngs::StdRng};

pub fn get_secret_key_from_ip(ip: &Ipv4Addr, seed: &[u8; 28]) -> SecretKey {
    let mut key_seed = [0u8; 32]; // Create new array with 32 elements
    key_seed[0..4].copy_from_slice(&ip.octets()); // 
    key_seed[4..32].copy_from_slice(seed); // Copy old values
    let rng = StdRng::from_seed(key_seed);
    SecretKey::generate(rng)
}
