use iroh::SecretKey;
use iroh_gossip::{
    net::{GossipReceiver, GossipSender},
    proto::TopicId,
};

#[derive(Debug)]
pub struct ChatRoom {
    pub secret_key: SecretKey,
    pub topic: TopicId,
    pub sender: GossipSender,
    pub receiver: GossipReceiver,
}

#[rustfmt::skip] // Not the best, but it works
// #[rustfmt::fn_single_line] // This should be set to EACH function in this block ;; AND it doesn't even work
impl ChatRoom {
    // pub fn subscribe(connection:Connection) -> Self { crfn::subscribe(connection) }
    // pub fn subscribe_and_join(connection:Connection) -> Self { crfn::subscribe_and_join(connection) }
}
