use futures_util::{SinkExt, StreamExt};
use rws_common::EventMessage;
use std::{env};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    time::Duration,
};
use tokio_tungstenite::connect_async;
use tungstenite::Message as WsMessage;
use url::Url;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let username = args.get(1).cloned().unwrap_or_else(|| "anonymous".into());

    let (ws_stream, _) = connect_async(Url::parse("ws://localhost:3000").unwrap())
        .await
        .expect("Failed to connect");
    println!("✅ Connected to server as `{}`", username);

    let (mut write, mut read) = ws_stream.split();

    // Clone username for the spawned task
    let username_for_task = username.clone();

    // Task to receive messages
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    match serde_json::from_str::<EventMessage>(&text) {
                        Ok(EventMessage::Chat { sender, content }) => {
                            println!("📨 {}: {}", sender, content);
                        }
                        Ok(EventMessage::Ping) => {
                            println!("🏓 Server pinged");
                        }
                        Ok(EventMessage::Join { username }) => {
                            println!("👤 {} joined", username);
                        }
                        Ok(EventMessage::CreateRoom { room_name}) => {
                            println!("🆕 Room created: {} by {}", room_name, username_for_task.clone());
                        }
                        Ok(EventMessage::LeaveRoom { room_id }) => {
                            println!("🚪 Left room: {}", room_id);
                        }
                        Ok(EventMessage::JoinRoom { room_id }) => {
                            println!("👥 Joined room: {}", room_id);
                        }
                        Err(_) => {
                            println!("❓ Unknown message: {}", text);
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("❌ Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Send the initial Join event
    let join = EventMessage::Join {
        username: username.clone(),
    };
    let raw = serde_json::to_string(&join).unwrap();
    write.send(WsMessage::Text(raw)).await.unwrap();

    println!("💬 Type messages to chat (Ctrl+C to exit)");

    // Read input from stdin
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let chat = EventMessage::Chat {
            sender: username.clone(),
            content: line,
        };
        let raw = serde_json::to_string(&chat).unwrap();
        if let Err(e) = write.send(WsMessage::Text(raw)).await {
            eprintln!("❌ Send error: {}", e);
            break;
        }
    }

    println!("👋 Disconnected.");
    tokio::time::sleep(Duration::from_secs(1)).await;
}
