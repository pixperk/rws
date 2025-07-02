use crate::{util::broadcast::{send, send_to_client_instance}, Clients};
use rws_common::{EventMessage, UserInfo};



pub mod room_handler;



pub async fn handle_join(username: String, sender_id: uuid::Uuid, clients: &Clients) {
    {
        let mut clients_guard = clients.lock().await;
        if let Some(client) = clients_guard.get_mut(&sender_id) {
            client.username = Some(username.clone());
            println!("🟢 {} joined as {}", sender_id, username);

            //Assign an ID to the client
            let id_msg = EventMessage::AssignedId { user_id: sender_id };
            send_to_client_instance(client, id_msg).await;
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
