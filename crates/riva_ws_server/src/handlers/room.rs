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
use thiserror::Error;

use crate::room::presentation::PresentationRoom;
use crate::{RoomConfig, RoomId, RoomLike, RoomMetadata, ServerState, TypedRoom, WsServer};

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Room already exists: {0}")]
    RoomAlreadyExists(RoomId),
    
    #[error("Invalid room type: {0}")]
    InvalidRoomType(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for RoomError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            RoomError::RoomAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            RoomError::InvalidRoomType(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            RoomError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomRequest<T: TypedRoom> {
    organisation_id: String,
    room_name: String,
    room: T
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomResponse {
    room_id: RoomId,
    success: bool,
    message: String,
}

pub async fn create_room(
    State(state): State<Arc<RwLock<ServerState>>>,
    Json(payload): Json<CreateRoomRequest<PresentationRoom>>,
) -> Result<Json<CreateRoomResponse>, RoomError> {
    let room_id = RoomId::new(&payload.organisation_id, &payload.room_name);
    
    let mut state_guard = state.write().await;
    
    if state_guard.rooms.contains_key(&room_id) {
        return Err(RoomError::RoomAlreadyExists(room_id));
    }
    
    state_guard.rooms.insert(room_id.clone(), Box::new(payload.room));
    
    Ok(Json(CreateRoomResponse {
        room_id,
        success: true,
        message: "Room created successfully".to_string(),
    }))
}

pub async fn get_rooms(
    State(state): State<Arc<RwLock<ServerState>>>
) -> Result<Json<Vec<(RoomId, RoomMetadata)>>, RoomError> {
    let state_guard = state.read().await;
    let rooms: Vec<(RoomId, RoomMetadata)> = state_guard.list_rooms().await;
    Ok(Json(rooms))
}
