use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{client::Clients, room::RoomManager};

pub struct Server {
    addr: String,
    clients: Clients,
    room_manager: Arc<Mutex<RoomManager>>,
}

