use crate::{room::SharedRoomManager, util::{broadcast::{broadcast_to_room, get_client_by_id, send, send_to_client_instance}, get_username_from_client}, Clients};
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

pub async fn handle_chat(content: String, sender_id: uuid::Uuid, clients: &Clients, room_manager : &SharedRoomManager) {
    println!(
        "DEBUG: Received chat message: '{}' from {}",
        content, sender_id
    ); // Debug log

    let sender = get_username_from_client(clients, sender_id).await.unwrap_or_else(|| "Unknown".to_string());

    // Broadcast the chat message to all clients (including sender)
    let chat_msg = EventMessage::Chat {
        sender: UserInfo {
            id: sender_id,
            username: sender.clone(),
        },
        content,
    };

    let room_id = {
        let rm = room_manager.lock().await;
        rm.user_rooms.get(&sender_id).cloned()
    };

    match room_id{
        Some(room_id) => {
            // If the user is in a room, broadcast to that room
            println!("DEBUG: Broadcasting to room {}: {:?}", room_id, chat_msg); // Debug log
            let room_manager = room_manager.lock().await;
            broadcast_to_room(&chat_msg, room_id, &room_manager, clients).await;
        },
        None => {
            // If the user is not in a room, broadcast to all clients
            println!("DEBUG: Broadcasting to all clients: {:?}", chat_msg); // Debug log
            send(&chat_msg, clients).await;
        }
    }
}
