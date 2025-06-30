use std::{collections::HashMap, sync::Arc};

use rws_common::Message;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_tungstenite::accept_async;
use futures_util::stream::{StreamExt};



type Clients = Arc<Mutex<HashMap<uuid::Uuid, tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Starting RWS server on ws://localhost:3000...");

    while let Ok((stream, _)) = listener.accept().await{
        let clients = Arc::clone(&clients);
        tokio::spawn(async move{
            let ws_stream = accept_async(stream).await.unwrap();
            let id = uuid::Uuid::new_v4();

             //clients.lock().await.insert(id, ws_stream);

            println!("New client connected: {}", id);
            let (mut write, mut read) = ws_stream.split();

            // Handle incoming messages
            while let Some(Ok(msg)) = read.next().await {
                if msg.is_text() {
                    if let Ok(msg_obj) = serde_json::from_str::<Message>(&msg.to_string()) {
                        println!("Received from {}: {:?}", id, msg_obj);
                    }
                }
            }

                  
            
            println!("Client {} disconnected", id);
            clients.lock().await.remove(&id);
             
        });
    }
}