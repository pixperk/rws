use rws_common::EventMessage;

use crate::{client::Clients, handler};

pub async fn dispatch(message: EventMessage, sender_id: uuid::Uuid, clients: &Clients){
        match message {
            EventMessage::Join { username } => handler::handle_join(username, sender_id, clients).await,
            EventMessage::Chat { sender, content } => handler::handle_chat(content, sender_id, clients).await,
            EventMessage::Ping => {
               
                println!("Received ping from client {}", sender_id);
            }
            _ => {
                eprintln!("❓ Unknown message: {:?}", message);
            }
        }
    }
