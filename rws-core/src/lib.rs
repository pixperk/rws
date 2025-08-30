use std::{collections::HashMap,sync::Arc};
use tokio::{net::{TcpListener, ToSocketAddrs}, sync::Mutex};
use futures_util::StreamExt;
use tokio_tungstenite::accept_async;
use rws_common::EventMessage;

use crate::{client::{Client, Clients},  room::RoomManager};


mod server;
mod client;
mod room;

pub struct Server {
    addr: String,
    clients: Clients,
    room_manager: Arc<Mutex<RoomManager>>,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs + ToString) -> anyhow::Result<Self> {
        // test binding for early errors
        TcpListener::bind(&addr).await?;

        Ok(Self {
            addr: addr.to_string(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            room_manager: Arc::new(Mutex::new(RoomManager::new())),
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Starting RWS server on ws://{}...", self.addr);

        while let Ok((stream, _)) = listener.accept().await {
            let clients = Arc::clone(&self.clients);
            let room_manager = Arc::clone(&self.room_manager);

            tokio::spawn(async move {
                let ws_stream = accept_async(stream).await.unwrap(); // WebSocket upgrade
                let id = uuid::Uuid::new_v4();

                println!("New client connected: {}", id);
                let (write, mut read) = ws_stream.split();
                let tx = Arc::new(Mutex::new(write));
                let client = Client {
                    id,
                    username: None,
                    tx: tx.clone(),
                };

                clients.lock().await.insert(id, client.clone());

                while let Some(Ok(msg)) = read.next().await {
                    if msg.is_text() {
                        if let Ok(msg_obj) = serde_json::from_str::<EventMessage>(&msg.to_string()) {
                            /* dispatch(msg_obj, id, &clients, &room_manager).await; */
                        }
                    }
                }

                clients.lock().await.remove(&id);
                println!("Client {} disconnected", id);
            });
        }

        Ok(())
    }
}
