use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rws_common::EventMessage;
use tokio::sync::{mpsc, Mutex};
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
    let join = EventMessage::Join {
        username: username.clone(),
    };
    let raw = serde_json::to_string(&join)?;
    write.send(WsMessage::Text(raw)).await?;

    ui_tx.send("🟢 Connected to server!".to_string())?;

    // Handle incoming messages
    let ui_tx_clone = ui_tx.clone();
    let self_id = Arc::new(Mutex::new(None));
    let self_id_clone = Arc::clone(&self_id);


    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(event) = serde_json::from_str::<EventMessage>(&text) {
                    match event {
                        EventMessage::AssignedId { user_id } => {
                            let mut id = self_id_clone.lock().await;
                            *id = Some(user_id);
                        }
                        _ => {
                           let id = self_id_clone.lock().await;
                        if let Some(my_id) = *id {
                            let formatted = format_message(event,  &my_id);
                            if !formatted.is_empty() {
                                let _ = ui_tx_clone.send(formatted);
                            }
                        }
                        }
                        
                    };
                }
            }
        }
    });

    // Handle outgoing messages
    while let Some(input) = ws_rx.recv().await {
        if input.is_empty() {
            continue;
        }

        let message = if input.starts_with("/create ") {
            let room_name = input.strip_prefix("/create ").unwrap().to_string();
            // Lock and extract the user id
            let id_guard = self_id.lock().await;
            if let Some(my_id) = *id_guard {
                EventMessage::CreateRoom { 
                    creator: rws_common::UserInfo {
                        id: my_id,
                        username: username.clone(),
                    },
                    room_name
                }
            } else {
                // If we don't have our id yet, skip sending the message
                continue;
            }
        } else if input.starts_with("/join ") {
            let room_id_str = input.strip_prefix("/join ").unwrap().trim();
            match uuid::Uuid::parse_str(room_id_str) {
                Ok(room_id) => {
                    let id_guard = self_id.lock().await;
                    if let Some(my_id) = *id_guard {
                        EventMessage::JoinRoom {
                            user: rws_common::UserInfo {
                                id: my_id,
                                username: username.clone(),
                            },
                            room: rws_common::RoomInfo {
                                id: room_id,
                                name: "".to_string(), // Name can be empty for join requests
                            },
                        }
                    } else {
                        // If we don't have our id yet, skip sending the message
                        continue;
                    }
                }
                Err(_) => {
                    ui_tx.send("❌ Invalid room ID format. Use: /join <room-uuid>".to_string())?;
                    continue;
                }
            }
        } else if input.starts_with("/leave ") {
            let room_id_str = input.strip_prefix("/leave ").unwrap().to_string();
            match uuid::Uuid::parse_str(&room_id_str) {
                Ok(room_id) => {
                    let id_guard = self_id.lock().await;
                    if let Some(my_id) = *id_guard {
                        EventMessage::LeaveRoom {
                            user: rws_common::UserInfo {
                                id: my_id,
                                username: username.clone(),
                            },
                            room: rws_common::RoomInfo {
                                id: room_id,
                                name: "".to_string(), // Name can be empty for leave requests
                            },
                        }
                    } else {
                        // If we don't have our id yet, skip sending the message
                        continue;
                    }
                }
                Err(_) => {
                    ui_tx.send("❌ Invalid room ID format. Use: /leave <room-uuid>".to_string())?;
                    continue;
                }
            }
        } else {
            // Lock and extract the user id
            let id_guard = self_id.lock().await;
            if let Some(my_id) = *id_guard {
                EventMessage::Chat {
                    sender: rws_common::UserInfo {
                        id: my_id,
                        username: username.clone(),
                    },
                    content: input,
                    scope : rws_common::ChatScope::Global // Default to global chat
                }
            } else {
                // If we don't have our id yet, skip sending the message
                continue;
            }
        };

        let raw = serde_json::to_string(&message)?;
        if write.send(WsMessage::Text(raw)).await.is_err() {
            break;
        }
    }

    Ok(())
}

fn format_message(event: EventMessage, self_id: &uuid::Uuid) -> String {
    match event {
        EventMessage::Chat {
            sender: rws_common::UserInfo { id: sender_id, username: sender_name },
            content,
            scope 
        } => {
            match scope {
                rws_common::ChatScope::Global => {
                    if self_id == &sender_id {
                        format!("[GLOBAL]💬 You: {}", content)
                    } else {
                        format!("[GLOBAL]💬 {}: {}", sender_name, content)
                    }
                }
                rws_common::ChatScope::Room { room } => {
                    if self_id == &sender_id {
                        format!("[{}]🏠 You: {}", room.name, content)
                    } else {
                        format!("[{}]🏠 {}: {}",room.name, sender_name, content)
                    }
                }
            }
        }
        EventMessage::Join { username } => {
           
                format!("👋 {} joined", username)
            
        }
        EventMessage::CreateRoom { 
            creator: rws_common::UserInfo { id: _, username },
            room_name,
        } => {
            format!("🏠 Room '{}' created! by '{}'", room_name, username)
        }
        EventMessage::JoinRoom {
            user: rws_common::UserInfo { id: user_id, username },
            room : rws_common::RoomInfo { id: _, name },
        } => {
            if self_id == &user_id {
                format!("✅ You joined room {}", name)
            } else {
                format!("👥 {} joined room {}", username, name)
            }
        }
        EventMessage::LeaveRoom {
            user: rws_common::UserInfo { id: user_id, username },
            room : rws_common::RoomInfo { id: _, name },
        } => {
            if self_id == &user_id {
                format!("🚪 You left room {}", name)
            } else {
                format!("👋 {} left room {}", username, name)
            }
        }
        EventMessage::Error { error } => match error {
            rws_common::ErrorCode::RoomNotFound { message } => format!("❌ Room not found: {}", message),
            rws_common::ErrorCode::RoomAlreadyExists { message } => format!("❌ Room already exists: {}", message),
            rws_common::ErrorCode::AlreadyInRoom { message } => format!("❌ Already in a room: {}", message),
            rws_common::ErrorCode::InvalidRoomId { message } => format!("❌ Invalid room ID: {}", message),
            rws_common::ErrorCode::PermissionDenied { message } => format!("❌ Permission denied: {}", message),
        },
        _ => String::new(),
    }
}
