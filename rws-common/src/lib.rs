use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
pub enum EventMessage {
    Join {username : String},
    Chat {
        sender: String,
        content : String
    },
    Ping,
}