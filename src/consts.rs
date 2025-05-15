// to get a new const SEED run: 'cargo test examples::get_random_seed::run -- --exact --nocapture'
pub const SEED: [u8; 32] = [
    11, 224, 148, 63, 181, 73, 114, 239, 172, 192, 51, 253, 89, 16, 164, 68, 123, 61, 209, 176,
    230, 163, 182, 249, 6, 224, 240, 79, 160, 193, 213, 204,
];

// to get a new const TOPIC run: 'cargo test examples::get_random_topic::run -- --exact --nocapture'
pub const TOPIC: &str = "01b434160bd09c7a753af2b6b53dfc64f7324f46b012c9f4610f593d47e5e488";

pub const RELAY_VEC: &[&str] = &["https://euw1-1.relay.iroh.network./"];
