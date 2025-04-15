use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use std::{collections::{HashMap, HashSet}, sync::Arc};
use axum::http::StatusCode;

use crate::{room::RoomConfig, Presentation, RoomId, ServerState, WsServer};

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
    State(server): State<WsServer>,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, (StatusCode, Json<CreateRoomResponse>)> {
    let room_id = RoomId::new(&payload.organisation_id, &payload.room_name);
    
    let config = RoomConfig {
        name: payload.name,
        initial_slide: Some(0),
        slide_data: payload.slide_data,
        ..Default::default()
    };
    
    match server.create_room(room_id.clone(), &payload.room_type, config).await {
        Ok(_) => Ok(Json(CreateRoomResponse {
            room_id,
            success: true,
            message: "Room created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(CreateRoomResponse {
                room_id,
                success: false,
                message: e,
            }),
        )),
    }
}

pub async fn get_rooms(
    State(state): State<Arc<RwLock<ServerState>>>,
) -> Json<Vec<RoomId>> {
    let state_guard = state.read().await;
    let rooms: Vec<RoomId> = state_guard.rooms.keys().cloned().collect();
    Json(rooms)
}