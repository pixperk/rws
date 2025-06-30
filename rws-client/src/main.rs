use futures_util::{StreamExt, SinkExt};
use rws_common::Message;
use tokio_tungstenite::connect_async;
use url::Url; // Add StreamExt

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async(Url::parse("ws://localhost:3000").unwrap()).await.unwrap();
    println!("Connected to the server");

    // Split the stream into read and write halves
    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to receive messages
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(tungstenite::Message::Text(text)) = msg {
                println!("Server: {text}");
            }
        }
    });

    // Send your message
    let msg = Message {
        event: "chat".to_string(),
        data: serde_json::json!({ "user": "yashaswi", "msg": "yo server" }),
    };

    let raw = serde_json::to_string(&msg).unwrap();
    write.send(tungstenite::Message::Text(raw)).await.unwrap();

    // Optional: wait a bit before exit so receive task doesn't instantly die
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}
