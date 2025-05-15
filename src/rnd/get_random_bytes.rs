use rand::Rng;

pub fn get_random_bytes<const N: usize>() -> [u8; N] {
    let mut rng = rand::thread_rng();
    let random_seed: [u8; N] = std::array::from_fn(|_| rng.r#gen());
    random_seed
}

