use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::SocketIo;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use ts_rs::TS;

use crate::{
    events::{ClientEvent, Message, ServerEvent},
    room::{presentation::Presentation, room_id::RoomId, RoomLike},
    Room, ServerState,
};

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct CreateRoomRequest {
    organisation_id: String,
    room_type: String,
    name: String,
    // Additional fields for room configuration
    slide_data: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct CreateRoomResponse {
    room_id: RoomId,
    success: bool,
    message: String,
}

pub async fn create_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Json(payload): Json<CreateRoomRequest>,
) -> Json<CreateRoomResponse> {
    let room_id = RoomId::new(&payload.organisation_id, &payload.name);
    let mut state_guard = state.write().await;
    
    // Check if room already exists
    if state_guard.rooms.contains_key(&room_id) {
        return Json(CreateRoomResponse {
            room_id,
            success: false,
            message: "Room already exists".to_string(),
        });
    }
    
    // Create room based on type
    let room = match payload.room_type.as_str() {
        Presentation::ROOM_TYPE => {
            let room_name = payload.name;
            let organisation_id = payload.organisation_id;
            let slide_data = payload.slide_data.unwrap_or_default();
            Room::Presentation(Presentation::new(
                room_name,
                organisation_id,
                0, // Start at first slide
                slide_data,
            ))
        },
        // Add other room types here as needed
        _ => {
            return Json(CreateRoomResponse {
                room_id,
                success: false,
                message: format!("Unsupported room type: {}", payload.room_type),
            });
        }
    };
    
    // Insert the new room
    state_guard.rooms.insert(room_id.clone(), room);
    
    Json(CreateRoomResponse {
        room_id,
        success: true,
        message: "Room created successfully".to_string(),
    })
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct GetRoomsResponse {
    rooms: Vec<(RoomId, Room)>,
}

pub async fn get_rooms(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> Json<GetRoomsResponse> {
    let state_guard = state.read().await;
    let rooms: Vec<(RoomId, Room)> = state_guard.rooms.iter()
        .map(|(id, room)| (id.clone(), room.clone()))
        .collect();
    Json(GetRoomsResponse { rooms })
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct RoomResponse {
    room_id: RoomId,
    room: Option<Room>,
    success: bool,
    message: String,
}

// Implement the get_room handler
pub async fn get_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Path(room_id_str): Path<String>,
) -> Json<RoomResponse> {
    let state_guard = state.read().await;
    
    // Parse the room_id from the path parameter
    let room_id = match RoomId::try_from(room_id_str.clone()) {
        Ok(id) => id,
        Err(_) => {
            return Json(RoomResponse {
                room_id: RoomId::new("invalid", "invalid"),
                room: None,
                success: false,
                message: "Invalid room ID format".to_string(),
            });
        }
    };
    
    // Look up the room in the state
    match state_guard.rooms.get(&room_id) {
        Some(room) => Json(RoomResponse {
            room_id: room_id.clone(),
            room: Some(room.clone()),
            success: true,
            message: "Room found".to_string(),
        }),
        None => Json(RoomResponse {
            room_id,
            room: None,
            success: false,
            message: "Room not found".to_string(),
        }),
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct UpdateRoomRequest {
    room: Room,
}

// Implement the update_room handler
pub async fn update_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpdateRoomRequest>,
) -> Json<RoomResponse> {
    let mut state_guard = state.write().await;
    
    // Parse the room_id from the path parameter
    let room_id = match RoomId::try_from(room_id_str.clone()) {
        Ok(id) => id,
        Err(_) => {
            return Json(RoomResponse {
                room_id: RoomId::new("invalid", "invalid"),
                room: None,
                success: false,
                message: "Invalid room ID format".to_string(),
            });
        }
    };
    
    // Check if the room exists
    if !state_guard.rooms.contains_key(&room_id) {
        return Json(RoomResponse {
            room_id,
            room: None,
            success: false,
            message: "Room not found".to_string(),
        });
    }
    
    // Update the room
    state_guard.rooms.insert(room_id.clone(), payload.room);
    
    Json(RoomResponse {
        room_id: room_id.clone(),
        room: state_guard.rooms.get(&room_id).cloned(),
        success: true,
        message: "Room updated successfully".to_string(),
    })
}

// Implement the delete_room handler
pub async fn delete_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Path(room_id_str): Path<String>,
) -> Json<RoomResponse> {
    let mut state_guard = state.write().await;
    
    // Parse the room_id from the path parameter
    let room_id = match RoomId::try_from(room_id_str.clone()) {
        Ok(id) => id,
        Err(_) => {
            return Json(RoomResponse {
                room_id: RoomId::new("invalid", "invalid"),
                room: None,
                success: false,
                message: "Invalid room ID format".to_string(),
            });
        }
    };
    
    // Remove the room and return the result
    match state_guard.rooms.remove(&room_id) {
        Some(room) => Json(RoomResponse {
            room_id: room_id.clone(),
            room: Some(room),
            success: true,
            message: "Room deleted successfully".to_string(),
        }),
        None => Json(RoomResponse {
            room_id,
            room: None,
            success: false,
            message: "Room not found".to_string(),
        }),
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct UpsertRoomRequest {
    organisation_id: String,
    room_type: String,
    name: String,
    slide_data: Option<Vec<Value>>,
}

// Implement the upsert_room handler (create if not exists, update if exists)
pub async fn upsert_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpsertRoomRequest>,
) -> Json<RoomResponse> {
    let mut state_guard = state.write().await;
    
    // Parse the room_id from the path parameter
    let room_id = match RoomId::try_from(room_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Json(RoomResponse {
                room_id: RoomId::new("invalid", "invalid"),
                room: None,
                success: false,
                message: "Invalid room ID format".to_string(),
            });
        }
    };
    
    // Create room based on type
    let room = match payload.room_type.as_str() {
        Presentation::ROOM_TYPE => {
            let room_name = payload.name;
            let organisation_id = payload.organisation_id;
            let slide_data = payload.slide_data.unwrap_or_default();
            Room::Presentation(Presentation::new(
                room_name,
                organisation_id,
                0, // Start at first slide
                slide_data,
            ))
        },
        // Add other room types here as needed
        _ => {
            return Json(RoomResponse {
                room_id,
                room: None,
                success: false,
                message: format!("Unsupported room type: {}", payload.room_type),
            });
        }
    };
    
    // Insert or update the room
    let exists = state_guard.rooms.contains_key(&room_id);
    state_guard.rooms.insert(room_id.clone(), room);
    
    let message = if exists {
        "Room updated successfully".to_string()
    } else {
        "Room created successfully".to_string()
    };
    
    Json(RoomResponse {
        room_id: room_id.clone(),
        room: state_guard.rooms.get(&room_id).cloned(),
        success: true,
        message,
    })
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BroadcastEventRequest {
    event: ServerEvent,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BroadcastEventResponse {
    success: bool,
    message: String,
}

// Implement the broadcast_event handler
#[axum::debug_handler]
pub async fn broadcast_event(
    State(state): State<Arc<RwLock<ServerState>>>,
    Path(room_id_str): Path<String>,
    io: axum::extract::Extension<SocketIo>,
    Json(payload): Json<BroadcastEventRequest>,
) -> Json<BroadcastEventResponse> {
    // Parse the room_id from the path parameter
    let room_id = match RoomId::try_from(room_id_str.clone()) {
        Ok(id) => id,
        Err(_) => {
            return Json(BroadcastEventResponse {
                success: false,
                message: "Invalid room ID format".to_string(),
            });
        }
    };
    
    // Check if the room exists
    let state_guard = state.read().await;
    if !state_guard.rooms.contains_key(&room_id) {
        return Json(BroadcastEventResponse {
            success: false,
            message: "Room not found".to_string(),
        });
    }
    
    // Create a server message
    let message = Message {
        room_id: room_id.clone(),
        payload: payload.event,
        datetime: Utc::now(),
        sender_id: None,
        request_id: None,
        broadcast: Some(true),
    };
    
    // Broadcast the event to all clients in the room
    match io.within(room_id_str.clone()).emit("message", &message).await {
        Ok(_) => {
            info!(
                room_id = %room_id_str,
                "Event broadcasted to room"
            );
            Json(BroadcastEventResponse {
                success: true,
                message: "Event broadcasted successfully".to_string(),
            })
        },
        Err(err) => {
            error!(
                room_id = %room_id_str,
                error = %err,
                "Failed to broadcast event"
            );
            Json(BroadcastEventResponse {
                success: false,
                message: format!("Failed to broadcast event: {}", err),
            })
        }
    }
}