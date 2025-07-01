use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
pub enum EventMessage {
    Join { username: String },
    AssignedId { user_id: uuid::Uuid },
    Chat { sender_id : uuid::Uuid, sender_name : String,  content: String },
    CreateRoom { room_name: String },
    JoinRoom { room_id: uuid::Uuid },
    LeaveRoom { room_id: uuid::Uuid },
    Ping,
}
