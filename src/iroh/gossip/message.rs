use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]

pub enum Message {
    AboutMe { username: String },
    SimpleText { text: String },
    RequestImg { image_name: String },
    // RequestImg {image_name: String},
}
