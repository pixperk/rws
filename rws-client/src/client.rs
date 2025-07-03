use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rws_common::{ChatScope, EventMessage, UserInfo};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use url::Url;
use uuid::Uuid;

use crate::app::UiEvent;

pub async fn connect_and_handle(
    username: String,
    server_url: String,
    ui_tx: mpsc::UnboundedSender<UiEvent>,
    mut ws_rx: mpsc::UnboundedReceiver<String>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(Url::parse(&server_url)?).await?;
    let (mut write, mut read) = ws_stream.split();

    let join = EventMessage::Join {
        username: username.clone(),
    };
    let raw = serde_json::to_string(&join)?;
    write.send(WsMessage::Text(raw)).await?;
    ui_tx.send(UiEvent::AddMessage {
        content: "🟢 Connected to server!".to_string(),
        is_system: true,
    })?;

    let self_id = Arc::new(Mutex::new(None));
    let pending_msgs = Arc::new(Mutex::new(HashMap::<Uuid, String>::new()));

    // Reader
    {
        let self_id = Arc::clone(&self_id);
        let ui_tx = ui_tx.clone();
        let pending_msgs = Arc::clone(&pending_msgs);

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(WsMessage::Text(text)) = msg {
                    if let Ok(event) = serde_json::from_str::<EventMessage>(&text) {
                        match &event {
                            EventMessage::AssignedId { user_id } => {
                                let mut id = self_id.lock().await;
                                *id = Some(*user_id);
                            }

                            EventMessage::Chat { id, sender, content, scope } => {
                                let my_id = self_id.lock().await;
                                if let Some(my_id) = *my_id {
                                    // Check if this is our own message coming back from server
                                    if sender.id == my_id {
                                        // This is our message being echoed back - treat as delivery confirmation
                                        let mut pending = pending_msgs.lock().await;
                                        if let Some(original_content) = pending.remove(id) {
                                            let delivered_msg = match scope {
                                                ChatScope::Global => format!("[GLOBAL]💬 You: {} ✅", original_content),
                                                ChatScope::Room { room } => format!("[{}]🏠 You: {} ✅", room.name, original_content),
                                            };
                                            let _ = ui_tx.send(UiEvent::UpdateMessage {
                                                id: *id,
                                                content: delivered_msg,
                                            });
                                        }
                                    } else {
                                        // This is someone else's message
                                        let formatted = format_message(event, &my_id);
                                        if !formatted.is_empty() {
                                            let _ = ui_tx.send(UiEvent::AddMessage {
                                                content: formatted,
                                                is_system: false,
                                            });
                                        }
                                    }
                                }
                            }

                            EventMessage::AckDelivered { id } => {
                                let mut pending = pending_msgs.lock().await;
                                if let Some(content) = pending.remove(id) {
                                    let delivered_content = format!("[GLOBAL]💬 You: {} ", content);
                                    let _ = ui_tx.send(UiEvent::UpdateMessage {
                                        id: *id,
                                        content: delivered_content,
                                    });
                                }
                            }

                            _ => {
                                let my_id = self_id.lock().await;
                                if let Some(my_id) = *my_id {
                                    let formatted = format_message(event, &my_id);
                                    if !formatted.is_empty() {
                                        let _ = ui_tx.send(UiEvent::AddMessage {
                                            content: formatted,
                                            is_system: false,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    // Writer
    while let Some(input) = ws_rx.recv().await {
        if input.trim().is_empty() {
            continue;
        }

        let my_id = *self_id.lock().await;
        if my_id.is_none() {
            continue;
        }
        let my_id = my_id.unwrap();

        let message = if input.starts_with("/create ") {
            let room_name = input["/create ".len()..].to_string();
            EventMessage::CreateRoom {
                creator: UserInfo {
                    id: my_id,
                    username: username.clone(),
                },
                room_name,
            }
        } else if input.starts_with("/join ") {
            let room_id_str = input["/join ".len()..].trim();
            match Uuid::parse_str(room_id_str) {
                Ok(room_id) => EventMessage::JoinRoom {
                    user: UserInfo {
                        id: my_id,
                        username: username.clone(),
                    },
                    room: rws_common::RoomInfo {
                        id: room_id,
                        name: "".to_string(),
                    },
                },
                Err(_) => {
                    ui_tx.send(UiEvent::AddMessage {
                        content: "❌ Invalid room ID format. Use: /join <room-uuid>".to_string(),
                        is_system: true,
                    })?;
                    continue;
                }
            }
        } else if input.starts_with("/leave") {
            EventMessage::LeaveRoom {
                user: UserInfo {
                    id: my_id,
                    username: username.clone(),
                },
                room: rws_common::RoomInfo {
                    id: Uuid::nil(),
                    name: "".to_string(),
                },
            }
        } else {
            let msg_id = Uuid::new_v4();

            // Insert into pending msgs
            {
                let mut pending = pending_msgs.lock().await;
                pending.insert(msg_id, input.clone());
            }

            ui_tx.send(UiEvent::AddMessageWithId {
                id: msg_id,
                content: format!("[GLOBAL]💬 You: {} ⏳", input),
                is_system: false,
            })?;

            EventMessage::Chat {
                id: msg_id,
                sender: UserInfo {
                    id: my_id,
                    username: username.clone(),
                },
                content: input,
                scope: ChatScope::Global,
            }
        };

        let raw = serde_json::to_string(&message)?;
        write.send(WsMessage::Text(raw)).await?;
    }

    Ok(())
}

fn format_message(event: EventMessage, self_id: &Uuid) -> String {
    use EventMessage::*;
    match event {
        Chat {
            id: _,
            sender: UserInfo { id, username },
            content,
            scope,
        } => match scope {
            ChatScope::Global => {
                if &id == self_id {
                    // Already handled via ack
                    "".into()
                } else {
                    format!("[GLOBAL]💬 {}: {}", username, content)
                }
            }
            ChatScope::Room { room } => {
                if &id == self_id {
                    format!("[{}]🏠 You: {}", room.name, content)
                } else {
                    format!("[{}]🏠 {}: {}", room.name, username, content)
                }
            }
        },
        Join { username } => format!("👋 {} joined", username),
        CreateRoom {
            creator: UserInfo { username, .. },
            room_name,
        } => format!("🏠 Room '{}' created by '{}'", room_name, username),
        JoinRoom {
            user: UserInfo { id, username },
            room,
        } => {
            if &id == self_id {
                format!("✅ You joined room {}", room.name)
            } else {
                format!("👥 {} joined room {}", username, room.name)
            }
        }
        LeaveRoom {
            user: UserInfo { id, username },
            room,
        } => {
            if &id == self_id {
                format!("🚪 You left room {}", room.name)
            } else {
                format!("👋 {} left room {}", username, room.name)
            }
        }
        Error { error } => format!("❌ Error: {:?}", error),
        _ => "".into(),
    }
}
