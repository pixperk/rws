use crate::{
    room::SharedRoomManager, util::{
        broadcast::{broadcast_to_room, send, send_to_client, send_to_client_instance},
        get_username_from_client,
    }, Clients
};
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

pub async fn handle_chat(
    id: uuid::Uuid,
    content: String,
    sender_id: uuid::Uuid,
    clients: &Clients,
    room_manager: &SharedRoomManager,
) {
    println!(
        "DEBUG: Received chat message: '{}' from {}",
        content, sender_id
    ); // Debug log

    let sender = get_username_from_client(clients, sender_id)
        .await
        .unwrap_or_else(|| "Unknown".to_string());

    let room_id = {
        let rm = room_manager.lock().await;
        rm.user_rooms.get(&sender_id).cloned()
    };

    let ack_delivered = EventMessage::AckDelivered { id };

    match room_id {
        Some(room_id) => {
            // If the user is in a room, broadcast to that room

            let room_manager = room_manager.lock().await;

            let chat_msg = EventMessage::Chat {
                id,
                sender: UserInfo {
                    id: sender_id,
                    username: sender.clone(),
                },
                content,
                scope: rws_common::ChatScope::Room {
                    room: rws_common::RoomInfo {
                        id: room_id,
                        name: room_manager
                            .rooms
                            .get(&room_id)
                            .map_or("Unknown".to_string(), |r| r.name.clone()),
                    },
                },
            };

            println!("DEBUG: Broadcasting to room {}: {:?}", room_id, chat_msg); // Debug log

            broadcast_to_room(&chat_msg, room_id, &room_manager, clients).await;
           
        }
        None => {
            // If the user is not in a room, broadcast to all clients
            let chat_msg = EventMessage::Chat {
                id,
                sender: UserInfo {
                    id: sender_id,
                    username: sender.clone(),
                },
                content,
                scope: rws_common::ChatScope::Global,
            };
            println!("DEBUG: Broadcasting to all clients: {:?}", chat_msg); // Debug log
            send(&chat_msg, clients).await;
        }

         
    }
    send_to_client(clients, sender_id, ack_delivered).await;
}
