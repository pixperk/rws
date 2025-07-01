use std::{collections::{HashMap, HashSet}, sync::Arc};

use tokio::sync::Mutex;


#[derive(Debug, Clone)]
pub struct Room{
    pub id: uuid::Uuid,
    pub name: String,
    pub owner_id : uuid::Uuid,
    pub memebers : HashSet<uuid::Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct RoomManager{
    pub rooms : HashMap<uuid::Uuid, Room>,
}

pub type SharedRoomManafer = Arc<Mutex<RoomManager>>;