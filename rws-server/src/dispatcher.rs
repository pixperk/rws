use rws_common::Message;

use crate::{handler::handle_chat, Clients};

pub async fn dispatch(event : &str, message : &Message, sender_id : uuid::Uuid, clients : &Clients){
    match event{
        "chat" => handle_chat(message, sender_id, clients).await,
        // Add more event handlers as needed
        _ => {
            println!("Unknown event: {}", event);
        }
    }
}