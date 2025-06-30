use futures_util::SinkExt;
use rws_common::Message;
use crate::Clients;
use uuid::Uuid;

use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

pub async fn handle_chat(message: &Message, sender_id: Uuid, clients: &Clients) {
    let json = serde_json::to_string(message).unwrap();

    let clients = clients.lock().await;

    for (id, client) in clients.iter(){
        if *id != sender_id{
            let mut tx = client.tx.lock().await;
            if let Err(e) = tx.send(WsMessage::Text(json.clone())).await{
                eprintln!("Failed to send to {} : {:?}", id, e);
            }

        }
    }
    println!("📢 Broadcasted chat from {} to {} clients", sender_id, clients.len() - 1);
}
