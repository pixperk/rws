use rws_common::EventMessage;

use crate::{handler::{self, handle_chat}, Clients};

pub async fn dispatch(message: EventMessage, sender_id: uuid::Uuid, clients: &Clients){
        match message {
            EventMessage::Join { username } => handler::handle_join(username, sender_id, clients).await,
            EventMessage::Chat { sender, content } => handle_chat(content, sender_id, clients).await,
            EventMessage::Ping => {
                // Handle ping if needed, currently no action
                println!("Received ping from client {}", sender_id);
            }
        }
    }
