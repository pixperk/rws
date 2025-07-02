use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
pub enum EventMessage {
    Join { username: String },
    AssignedId { user_id: uuid::Uuid },
    Chat { sender : UserInfo,  content: String },
    CreateRoom { creator : UserInfo, room_name: String },
    JoinRoom { user : UserInfo, room_id: uuid::Uuid },
    LeaveRoom { user : UserInfo, room_id: uuid::Uuid },
    Ping,
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub id: uuid::Uuid,
    pub username: String,
}