use rws_common::Message;
use crate::Clients;
use uuid::Uuid;

pub async fn handle_chat(message: &Message, sender_id: Uuid, _clients: &Clients) {
    println!("💬 [chat] from {} => {:?}", sender_id, message);
    // TODO: Broadcast logic
}
