// to get a new const SEED run 'cargo test get_random_seed -- --nocapture'
// pub const SEED: [u8; 28] = [
//     7, 65, 249, 159, 206, 249, 104, 196, 5, 86, 85, 174, 126, 11, 239, 214, 51, 41, 4, 137, 88, 75,
//     191, 62, 174, 249, 220, 235,
// ];
pub const SEED: [u8; 32] = [
    7, 65, 249, 159, 206, 249, 104, 196, 5, 86, 85, 174, 126, 11, 239, 214, 51, 41, 4, 137, 88, 75,
    191, 62, 174, 249, 220, 235, 27, 102, 97, 65
];
// to get a new const TOPIC run 'cargo test get_random_topic -- --nocapture'
pub const TOPIC: &str = "1c0dfcb99ba03e3c799a9c7053d04cd5ceb7cef623b9141e6777103ac045fcfb";
// server ports should u16 values between 49152 and 65535
pub const SERVER_PORTS: &[u16] = &[52027];
