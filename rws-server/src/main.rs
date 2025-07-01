use std::{collections::HashMap, sync::Arc};

use rws_common::EventMessage;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_tungstenite::accept_async;
use futures_util::stream::{StreamExt};

use crate::{client::{Client, Clients}, dispatcher::dispatch};

mod dispatcher;
mod handler;
mod client;


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Starting RWS server on ws://localhost:3000...");

    while let Ok((stream, _)) = listener.accept().await{
        let clients = Arc::clone(&clients);
        tokio::spawn(async move{
            let ws_stream = accept_async(stream).await.unwrap(); //upgrade to WebSocket stream
            let id = uuid::Uuid::new_v4();


            println!("New client connected: {}", id);
            let (write, mut read) = ws_stream.split();
            let tx = Arc::new(Mutex::new(write));
            let client = Client{
                id,
                username: None,
                tx: tx.clone(),
            };

            clients.lock().await.insert(id, client.clone());

            // Handle incoming messages
            while let Some(Ok(msg)) = read.next().await {
                if msg.is_text() {
                    if let Ok(msg_obj) = serde_json::from_str::<EventMessage>(&msg.to_string()) {
                       dispatch(msg_obj, id,  &clients).await;
                    }
                }
            }

                  
             clients.lock().await.remove(&id);
            println!("Client {} disconnected", id);
           
             
        });
    }
}