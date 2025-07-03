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

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
            user_rooms: HashMap::new(),
        }
    }

    pub fn get_room(&self, room_id: &uuid::Uuid) -> Option<Room> {
        self.rooms.get(room_id).cloned()
    }

    pub fn get_user_room(&self, user_id: &uuid::Uuid) -> Option<uuid::Uuid> {
        self.user_rooms.get(user_id).cloned()
    }
}