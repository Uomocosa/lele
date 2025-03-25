use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::iroh::User;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]

pub enum Message {
    AboutMe { username: String },
    SimpleText { text: String },
    RequestImg { image_name: String },
    // RequestImg {image_name: String},
}

#[rustfmt::skip] // Not the best, but it works
// #[rustfmt::fn_single_line] // This should be set to EACH function in this block ;; AND it doesn't even work
impl Message {
    pub fn about_me(user: &User) -> Result<Message> {
        match user {
            User::Empty => return Err(anyhow!("message::about_me::UserIsEmpty")),
            User::Data {..} => {},
        }
        Ok(Message::AboutMe { username: user.name().unwrap() })
    }

    pub fn text(msg: &str) -> Message {
        Message::SimpleText{ text: msg.to_string() }
    }

    pub fn req_img(image_name: &str) -> Message {
        Message::RequestImg{ image_name: image_name.to_string() }
    }
    
}
