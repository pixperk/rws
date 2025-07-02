use std::collections::{HashMap, HashSet};
use rws_common::EventMessage;

use crate::{
    client::Clients,
    room::{self, RoomManager},
    util::{broadcast::send_to_client, get_username_from_client},
};

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
            user_rooms: HashMap::new(),
        }
    }

    pub async fn handle_create_room(
        &mut self,
        clients: &Clients,
        client_id: uuid::Uuid,
        room_name: String,
    ) {
        let room_id = uuid::Uuid::new_v4();

        if self.rooms.contains_key(&room_id) {
            eprintln!("Room with id {} already exists", room_id);

            let error_event = EventMessage::Error {
                error: rws_common::ErrorCode::RoomAlreadyExists {
                    message: format!("Room with id {} already exists", room_id),
                },
            };

            send_to_client(clients, client_id, error_event).await;

            return;
        }

        if self.user_rooms.contains_key(&client_id) {
            eprintln!("Client {} is already in a room", client_id);

            let error_event = EventMessage::Error {
                error: rws_common::ErrorCode::AlreadyInRoom {
                    message: format!("Client {} is already in a room", client_id),
                },
            };

            send_to_client(clients, client_id, error_event).await;

            return;
        }

        let created_room = room::Room {
            id: room_id,
            name: room_name.clone(),
            owner_id: client_id,
            members: {
                let mut s = HashSet::new();
                s.insert(client_id);
                s
            },
        };

        self.rooms.insert(room_id, created_room);
        self.user_rooms.insert(client_id, room_id);

        let user = get_username_from_client(clients, client_id);

        if let Some(username) = user.await {
            println!("🟢 {} created room {}", username, room_name);
        } else {
            println!("🟢 Client with id {} created room {}", client_id, room_name);
        }

        //Debug hashmaps
        println!("DEBUG : Current rooms: {:?}", self.rooms);
        println!("DEBUG  : Current user_rooms: {:?}", self.user_rooms);

        let create_room_event = EventMessage::CreateRoom {
            creator: rws_common::UserInfo {
                id: client_id,
                username: get_username_from_client(clients, client_id)
                    .await
                    .unwrap_or_else(|| "Unknown".to_string()),
            },
            room_name: room_name.clone(),
        };

        send_to_client(clients, client_id, create_room_event).await;
    }
}


