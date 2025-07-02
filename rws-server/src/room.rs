use std::{collections::{HashMap, HashSet}, sync::Arc};

use tokio::sync::Mutex;


#[derive(Debug, Clone)]
pub struct Room{
    pub id: uuid::Uuid,
    pub name: String,
    pub owner_id : uuid::Uuid,
    pub members : HashSet<uuid::Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct RoomManager{
    pub rooms : HashMap<uuid::Uuid, Room>,
    pub user_rooms : HashMap<uuid::Uuid, uuid::Uuid>, //user -> room
}

pub type SharedRoomManager = Arc<Mutex<RoomManager>>;