// to get a new const SEED run: 'cargo test examples::get_random_seed::run -- --exact --nocapture'
pub const SEED: [u8; 32] = [153, 19, 193, 113, 32, 237, 106, 239, 148, 24, 55, 180, 87, 146, 114, 196, 84, 129, 201, 44, 137, 54, 40, 205, 94, 219, 121, 108, 11, 217, 195, 139];  

// to get a new const TOPIC run: 'cargo test examples::get_random_topic::run -- --exact --nocapture'
pub const TOPIC: &str = "c83e12f6fc1e76e142e13e6020a661877c359a3cd45d0c2256716e3829dceb6e";

pub const RELAY_VEC: &[&str] = &["https://euw1-1.relay.iroh.network./"];
