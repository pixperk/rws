use rws_common::EventMessage;

use crate::{client::Clients, handler, room::SharedRoomManager};

pub async fn dispatch(message: EventMessage, sender_id: uuid::Uuid, clients: &Clients, room_manager: &SharedRoomManager) {
        match message {
            EventMessage::Join { username } => handler::handle_join(username, sender_id, clients).await,
            EventMessage::Chat { sender,  content } => handler::handle_chat(content, sender_id, clients, room_manager).await,
            EventMessage::Ping => {
               
                println!("Received ping from client {}", sender_id);
            }
            EventMessage::CreateRoom { creator, room_name } => room_manager.lock().await.handle_create_room(clients, sender_id, room_name).await,
            _ => {
                eprintln!("❓ Unknown message: {:?}", message);
            }
        }
    }
