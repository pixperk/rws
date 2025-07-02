use rws_common::EventMessage;

use crate::{client::Clients, handler, room::SharedRoomManager};

pub async fn dispatch(message: EventMessage, sender_id: uuid::Uuid, clients: &Clients, room_manager: &SharedRoomManager) {
        match message {
            EventMessage::Join { username } => handler::handle_join(username, sender_id, clients).await,
            EventMessage::Chat { id, sender,  content , scope : _} => handler::handle_chat(id, content, sender.id, clients, room_manager).await,
            EventMessage::Ping => {
               
                println!("Received ping from client {}", sender_id);
            }
            EventMessage::CreateRoom { creator, room_name } => room_manager.lock().await.handle_create_room(clients, creator.id, room_name).await,
            EventMessage::JoinRoom { user, room } => {
                room_manager.lock().await.handle_join_room(clients, user.id, room.id).await;
            }
            EventMessage::LeaveRoom { user, room : _ } => {
                room_manager.lock().await.handle_leave_room(clients, user.id).await;
            }
            _ => {
                eprintln!("❓ Unknown message: {:?}", message);
            }
        }
    }
