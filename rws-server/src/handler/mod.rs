use crate::Clients;
use futures_util::SinkExt;
use rws_common::{EventMessage, UserInfo};

use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

pub mod room_handler;

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

pub async fn handle_join(username: String, sender_id: uuid::Uuid, clients: &Clients) {
    {
        let mut clients_guard = clients.lock().await;
        if let Some(client) = clients_guard.get_mut(&sender_id) {
            client.username = Some(username.clone());
            println!("🟢 {} joined as {}", sender_id, username);

            //Assign an ID to the client
            let id_msg = EventMessage::AssignedId { user_id: sender_id };
            let tx = client.tx.clone();
            let payload = serde_json::to_string(&id_msg).unwrap();
            let mut tx_lock = tx.lock().await;
            let _ = tx_lock.send(WsMessage::Text(payload)).await;
        }
    } // Release the lock here

    // Broadcast the join event to all clients
    let join_msg = EventMessage::Join { username };
    send(&join_msg, clients).await;
}

pub async fn handle_chat(content: String, sender_id: uuid::Uuid, clients: &Clients) {
    println!(
        "DEBUG: Received chat message: '{}' from {}",
        content, sender_id
    ); // Debug log

    let sender = {
        let clients_guard = clients.lock().await;
        clients_guard
            .get(&sender_id)
            .and_then(|client| client.username.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }; // Release the lock here

    // Broadcast the chat message to all clients (including sender)
    let chat_msg = EventMessage::Chat {
        sender: UserInfo {
            id: sender_id,
            username: sender.clone(),
        },
        content,
    };
    println!("DEBUG: Broadcasting message: {:?}", chat_msg); // Debug log
    send(&chat_msg, clients).await;
}
