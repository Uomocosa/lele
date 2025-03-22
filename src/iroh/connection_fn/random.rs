use iroh::SecretKey;
use iroh_gossip::proto::TopicId;

use crate::iroh::Connection;
use crate::string::random_string;

pub fn random() -> Connection {
    let mut connection = Connection::_empty();
    connection.set_secret_key(SecretKey::generate(rand::rngs::OsRng));
    connection.set_name(&("random_".to_owned() + &random_string(5)));
    connection.set_topic(&TopicId::from_bytes(rand::random()).to_string());
    connection
}

