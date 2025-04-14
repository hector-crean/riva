
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use std::{collections::{HashMap, HashSet}, sync::Arc};

use crate::{Presentation, Room, RoomId, ServerState};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomRequest {
    organisation_id: String,
    room_name: String,
    room_type: String,
    // Additional fields for room configuration
    name: Option<String>,
    slide_data: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomResponse {
    room_id: RoomId,
    success: bool,
    message: String,
}


pub async fn create_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Json(payload): Json<CreateRoomRequest>,
) -> Json<CreateRoomResponse> {
    let room_id = RoomId::new(&payload.organisation_id, &payload.room_name);
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
        "presentation" => {
            let name = payload.name.unwrap_or_else(|| "Untitled Presentation".to_string());
            let slide_data = payload.slide_data.unwrap_or_default();
            Room::Presentation(Presentation::new(
                name,
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



pub async fn get_rooms(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> Json<Vec<RoomId>> {
    let state_guard = state.read().await;
    let rooms: Vec<RoomId> = state_guard.rooms.keys().cloned().collect();
    Json(rooms)
}