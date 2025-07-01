use futures_util::SinkExt;
use rws_common::EventMessage;
use crate::Clients;


use tokio_tungstenite::tungstenite::{protocol::Message as WsMessage};

pub async fn handle_join(username : String, sender_id : uuid::Uuid, clients: &Clients){
    let mut clients = clients.lock().await;

    if let Some(client) = clients.get_mut(&sender_id){
        client.username = Some(username.clone());
        println!("🟢 {} joined as {}", sender_id, username);
    }

    //broadcast the join event to all clients
     // let join_msg = EventMessage::SystemJoined { ... };
}

pub async fn handle_chat(content : String, sender_id : uuid::Uuid, clients : &Clients){
    let clients  = clients.lock().await;
    let sender = clients.get(&sender_id)
        .and_then(|client| client.username.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let payload = serde_json::to_string(
        &EventMessage::Chat { sender, content }
    ).unwrap();

    for (id, client) in clients.iter(){
        if *id != sender_id {
            let mut tx = client.tx.lock().await;
             let _ = tx.send(WsMessage::Text(payload.clone())).await;
        }
    }
}