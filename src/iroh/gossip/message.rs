use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    AboutMe { name: String },
    Message { text: String },
}
