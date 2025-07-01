use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rws_common::EventMessage;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use url::Url;

pub async fn connect_and_handle(
    username: String,
    server_url: String,
    ui_tx: mpsc::UnboundedSender<String>,
    mut ws_rx: mpsc::UnboundedReceiver<String>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(Url::parse(&server_url)?).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send join message
    let join = EventMessage::Join { username: username.clone() };
    let raw = serde_json::to_string(&join)?;
    write.send(WsMessage::Text(raw)).await?;

    ui_tx.send("🟢 Connected to server!".to_string())?;

    // Handle incoming messages
    let ui_tx_clone = ui_tx.clone();
    let username_clone = username.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(event) = serde_json::from_str::<EventMessage>(&text) {
                    let formatted = format_message(event, &username_clone);
                    if !formatted.is_empty() {
                        let _ = ui_tx_clone.send(formatted);
                    }
                }
            }
        }
    });

    // Handle outgoing messages
    while let Some(input) = ws_rx.recv().await {
        if input.is_empty() { continue; }

        let message = if input.starts_with("/create ") {
            let room_name = input.strip_prefix("/create ").unwrap().to_string();
            EventMessage::CreateRoom { room_name }
        } else {
            EventMessage::Chat {
                sender: username.clone(),
                content: input,
            }
        };

        let raw = serde_json::to_string(&message)?;
        if write.send(WsMessage::Text(raw)).await.is_err() {
            break;
        }
    }

    Ok(())
}

fn format_message(event: EventMessage, current_user: &str) -> String {
    match event {
        EventMessage::Chat { sender, content } => {
            if sender == current_user {
                String::new() // Don't show our own messages
            } else {
                format!("💬 {}: {}", sender, content)
            }
        }
        EventMessage::Join { username } => {
            if username == current_user {
                "✅ You joined the chat".to_string()
            } else {
                format!("👋 {} joined", username)
            }
        }
        EventMessage::CreateRoom { room_name } => {
            format!("🏠 Room '{}' created!", room_name)
        }
        _ => String::new(),
    }
}
