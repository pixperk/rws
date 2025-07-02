use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
pub enum EventMessage {
    Join { username: String },
    AssignedId { user_id: uuid::Uuid },
    Chat { sender: UserInfo, content: String, scope: ChatScope },
    CreateRoom { creator: UserInfo, room_name: String },
    JoinRoom { user: UserInfo, room: RoomInfo },
    LeaveRoom { user: UserInfo, room: RoomInfo },
    Error { error: ErrorCode },
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub id: uuid::Uuid,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub id : uuid::Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "error_type", content = "details")]
pub enum ErrorCode {
    RoomNotFound { message: String },
    RoomAlreadyExists { message: String },
    AlreadyInRoom { message: String },
    InvalidRoomId { message: String },
    PermissionDenied { message: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "scope", content = "details")]
pub enum ChatScope {
    Global,
    Room{room : RoomInfo},
}