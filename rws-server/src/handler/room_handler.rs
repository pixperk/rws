use std::collections::HashSet;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::{protocol::Message as WsMessage};

use rws_common::EventMessage;

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

    let create_room_event = EventMessage::CreateRoom { room_name: room_name.clone() };

    let user = get_username_from_client(clients, client_id);

   let clients = clients.lock().await;
    let tx = clients.get(&client_id).map(|c| c.tx.clone());
    let payload = serde_json::to_string(&create_room_event).unwrap();
    let tx = tx.unwrap();
    let mut tx_lock = tx.lock().await;
    let _ = tx_lock.send(WsMessage::Text(payload)).await;
    

    if let Some(username) = user.await {
        println!("🟢 {} created room {}", username, room_name);
    } else {
        println!("🟢 Client with id {} created room {}", client_id, room_name);
    }

}