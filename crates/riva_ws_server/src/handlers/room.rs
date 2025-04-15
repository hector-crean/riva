use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::{RoomConfig, RoomId, ServerState, WsServer};



#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomRequest {
    organisation_id: String,
    room_name: String,
    room_type: String,
    // Additional fields for room configuration
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomResponse {
    room_id: RoomId,
    success: bool,
    message: String,
}

#[axum::debug_handler]
pub async fn create_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Json(payload): Json<CreateRoomRequest>,
) -> Json<CreateRoomResponse> {
    let room_id = RoomId::new(&payload.organisation_id, &payload.room_name);

    let config = RoomConfig::default();
    let state_guard = state.read().await;

    

    state_guard.create_room(room_id.clone(), &payload.room_type, config).await;

   Json(CreateRoomResponse {
    room_id,
    success: true,
    message: "Room created successfully".to_string(),
   })
}

pub async fn get_rooms(State(state): State<Arc<RwLock<ServerState>>>) -> Json<Vec<RoomId>> {
    let state_guard = state.read().await;
    let rooms: Vec<RoomId> = state_guard.rooms.keys().cloned().collect();
    Json(rooms)
}
