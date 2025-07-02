use rws_common::EventMessage;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures_util::SinkExt;

use crate::{client::Clients, room::{Room, RoomManager, SharedRoomManager}};

/// Broadcast a message to all connected clients
pub async fn send(message: &EventMessage, clients: &Clients) {
    let clients = clients.lock().await;
    let payload = serde_json::to_string(message).unwrap();

    for (_, client) in clients.iter() {
        let mut tx = client.tx.lock().await;
        let _ = tx.send(WsMessage::Text(payload.clone())).await;
    }
}

/// Broadcast a message to all connected clients except the sender
pub async fn broadcast(message: &EventMessage, sender_id: uuid::Uuid, clients: &Clients) {
    let clients = clients.lock().await;
    let payload = serde_json::to_string(message).unwrap();

    for (id, client) in clients.iter() {
        if *id != sender_id {
            let mut tx = client.tx.lock().await;
            let _ = tx.send(WsMessage::Text(payload.clone())).await;
        }
    }
}

pub async fn broadcast_to_room(
    message: &EventMessage,
    room_id: uuid::Uuid,
    rm: &RoomManager,
    clients: &Clients,
){
    let room = rm.rooms.get(&room_id);
    let payload = serde_json::to_string(message).unwrap();

    if let Some(room) = room {
        let c = clients.lock().await;
        for id in &room.members {
            if let Some(client) = c.get(id) {
                let mut tx = client.tx.lock().await;
                let _ = tx.send(WsMessage::Text(payload.clone())).await;
            }
        }
    }
}

/// Get a client by ID
pub async fn get_client_by_id(
    clients: &Clients,
    client_id: uuid::Uuid,
) -> Option<crate::client::Client> {
    let clients = clients.lock().await;
    clients.get(&client_id).cloned()
}

/// Send a message to a specific client instance
pub async fn send_to_client_instance(
    client: &crate::client::Client,
    event: EventMessage,
) {
    let payload = serde_json::to_string(&event).unwrap();
    let mut tx_lock = client.tx.lock().await;
    let _ = tx_lock.send(WsMessage::Text(payload)).await;
}

/// Send a message to a specific client by ID
pub async fn send_to_client(
    clients: &Clients,
    client_id: uuid::Uuid,
    event: EventMessage,
) {
    if let Some(client) = get_client_by_id(clients, client_id).await {
        send_to_client_instance(&client, event).await;
    }
}