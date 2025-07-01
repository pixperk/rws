use std::collections::HashSet;

use crate::{client::Clients, room, util::get_username_from_client};

pub async fn handle_create_room(
    clients: &Clients,
    client_id : uuid::Uuid,
    room_name : String,
    room_manager: &room::SharedRoomManager

){
    let room_id = uuid::Uuid::new_v4();
    let mut rm = room_manager.lock().await;

    if rm.rooms.contains_key(&room_id) {
        eprintln!("Room with id {} already exists", room_id);
        return;
    }

    let created_room = room::Room{
        id : room_id,
        name : room_name.clone(),
        owner_id : client_id,
        members : {
            let mut s = HashSet::new();
            s.insert(client_id);
            s
        }
    };

    rm.rooms.insert(room_id, created_room);

    let user = get_username_from_client(clients, client_id);

    if let Some(username) = user.await {
        println!("🟢 {} created room {}", username, room_name);
    } else {
        println!("🟢 Client with id {} created room {}", client_id, room_name);
    }

}