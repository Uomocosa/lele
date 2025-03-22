use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]

pub enum MessageContent {
    AboutMe {username: String},
    RequestImg {image_name: String},
    // RequestImg {image_name: String},
}

