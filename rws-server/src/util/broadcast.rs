use rws_common::EventMessage;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures_util::SinkExt;

use crate::client::Clients;

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

/// Send a message to a specific client
pub async fn send_to_client(
    clients: &Clients,
    client_id: uuid::Uuid,
    event: EventMessage,
) {
    let clients = clients.lock().await;
    let tx = clients.get(&client_id).map(|c| c.tx.clone());
    let payload = serde_json::to_string(&event).unwrap();
    let tx = tx.unwrap();
    let mut tx_lock = tx.lock().await;
    let _ = tx_lock.send(WsMessage::Text(payload)).await;
}