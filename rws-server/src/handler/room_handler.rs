use rws_common::EventMessage;
use std::collections::{HashMap, HashSet};

use crate::{
    client::Clients,
    room::{self, RoomManager},
    util::{
        broadcast::{broadcast_to_room, send_to_client},
        get_username_from_client,
    },
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

    pub async fn handle_join_room(
        &mut self,
        clients: &Clients,
        client_id: uuid::Uuid,
        room_id: uuid::Uuid,
    ) {
        if let Some(room) = self.rooms.get_mut(&room_id) {
            if room.members.contains(&client_id) {
                eprintln!("Client {} is already in room {}", client_id, room_id);

                let error_event = EventMessage::Error {
                    error: rws_common::ErrorCode::AlreadyInRoom {
                        message: format!("Client {} is already in room {}", client_id, room_id),
                    },
                };

                send_to_client(clients, client_id, error_event).await;
                return;
            }

            room.members.insert(client_id);
            self.user_rooms.insert(client_id, room_id);

            let join_event = EventMessage::JoinRoom {
                user: rws_common::UserInfo {
                    id: client_id,
                    username: get_username_from_client(clients, client_id)
                        .await
                        .unwrap_or_else(|| "Unknown".to_string()),
                },
                room: rws_common::RoomInfo {
                    id: room_id,
                    name: room.name.clone(),
                },
            };
            {};

            let room_name = room.name.clone();

            broadcast_to_room(&join_event, room_id, self, clients).await;

            println!("🟢 Client {} joined room {}", client_id, room_name);
        } else {
            eprintln!("Room with id {} not found", room_id);

            let error_event = EventMessage::Error {
                error: rws_common::ErrorCode::RoomNotFound {
                    message: format!("Room with id {} not found", room_id),
                },
            };

            send_to_client(clients, client_id, error_event).await;
        }
    }

   pub async fn handle_leave_room(
    &mut self,
    clients: &Clients,
    client_id: uuid::Uuid,
) {
    
    let room_id = match self.user_rooms.get(&client_id).cloned() {
        Some(id) => id,
        None => {
            eprintln!("Client {} is not in any room", client_id);
            return;
        }
    };

    let (room_name, all_members) = if let Some(room) = self.rooms.get_mut(&room_id) {
        let room_name = room.name.clone();
        // Get all members before removing the leaving user
        let all_members: Vec<uuid::Uuid> = room.members.iter().copied().collect();
        room.members.remove(&client_id);
        (room_name, all_members)
    } else {
        eprintln!("Room with id {} not found", room_id);
        return;
    };

    self.user_rooms.remove(&client_id);

    // Check if room is empty after removal
    let room_empty = if let Some(room) = self.rooms.get(&room_id) {
        room.members.is_empty()
    } else {
        false
    };

    if room_empty {
        self.rooms.remove(&room_id);
        println!("🟢 Room {} is now empty and has been removed", room_name);
    }

    let leave_event = EventMessage::LeaveRoom {
        user: rws_common::UserInfo {
            id: client_id,
            username: get_username_from_client(clients, client_id)
                .await
                .unwrap_or_else(|| "Unknown".to_string()),
        },
        room: rws_common::RoomInfo {
            id: room_id,
            name: room_name.clone(),
        },
    };

    // Send to all members (including the one who left)
    for member_id in all_members {
        send_to_client(clients, member_id, leave_event.clone()).await;
    }
    

    println!("🔴 Client {} left room {}", client_id, room_name);
}

}
