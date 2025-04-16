
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use ts_rs::TS;
use std::{collections::{HashMap, HashSet}, sync::Arc};
use crate::RoomLike;

use crate::{Presentation, Room, RoomId, ServerState};

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
    let rooms: Vec<(RoomId, Room)> = state_guard.rooms.iter().map(|(id, room)| (id.clone(), room.clone())).collect();
    Json(GetRoomsResponse { rooms })
}