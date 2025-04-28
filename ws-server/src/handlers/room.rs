use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::SocketIo;
use std::sync::{Arc, RwLock};
use tracing::{error, info};
use ts_rs::TS;

use crate::{
    AppState, Application,
    message::{Message, ServerMessage},
    message_broker::MessageBroker,
    presentation::Presentation,
    room::{RoomError, RoomLike, room_id::RoomId, room_manager::RoomManager},
};

impl IntoResponse for RoomError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RoomError::ClientNotFound(client_id) => (
                StatusCode::NOT_FOUND,
                format!("Client '{}' not found", client_id),
            ),
            RoomError::StorageError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage operation failed: {}", err),
            ),
            RoomError::PresenceError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Presence operation failed: {}", err),
            ),
            RoomError::TransactionError(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Transaction failed: {}", msg),
            ),
            RoomError::SerializationError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Serialization error: {}", err),
            ),
            RoomError::NetworkError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Network/Broadcast error: {}", msg),
            ),
            RoomError::PersistenceError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Persistence error: {}", msg),
            ),
            RoomError::RoomNotFound(room_id) => (
                StatusCode::NOT_FOUND,
                format!("Room not found: {}", room_id),
            ),
            RoomError::MessageBrokerError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Message broker error: {}", err),
            ),
            RoomError::RoomAlreadyExists(room_id) => (
                StatusCode::BAD_REQUEST,
                format!("Room already exists: {}", room_id),
            ),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomRequest<R: RoomLike> {
    pub project_id: Option<String>,
    pub organisation_id: Option<String>,
    pub room_type: String,
    pub room_name: String,
    pub room: R,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomResponse {
    pub room_id: RoomId,
    pub success: bool,
    pub message: String,
}

pub async fn create_room<S>(
    State(state): State<S>,
    Json(payload): Json<CreateRoomRequest<S::Room>>,
) -> Result<Json<CreateRoomResponse>, RoomError>
where
    S: AppState,
{
    let room_id = RoomId::new();
    let new_room = payload.room;

    let room_manager = state.room_manager();

    room_manager.add_room(room_id.clone(), new_room).await?;

    Ok(Json(CreateRoomResponse {
        room_id,
        success: true,
        message: "Room created successfully".to_string(),
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomResponse<R: RoomLike> {
    pub room: Option<R>,
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRoomRequest<R: RoomLike> {
    pub room: R,
}

pub async fn get_room<S>(
    State(state): State<S>,
    Path(room_id_str): Path<String>,
) -> Result<Json<RoomResponse<S::Room>>, RoomError>
where
    S: AppState,
{
    let room_id =
        RoomId::try_from(room_id_str).map_err(|_| RoomError::RoomNotFound(RoomId::new()))?;

    let room_manager = state.room_manager();
    let room = room_manager.get_room(&room_id).await?;

    Ok(Json(RoomResponse {
        room: Some(room),
        success: true,
        message: "Room found".to_string(),
    }))
}

pub async fn update_room<S>(
    State(state): State<S>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpdateRoomRequest<S::Room>>,
) -> Result<Json<RoomResponse<S::Room>>, RoomError>
where
    S: AppState,
{
    let room_id =
        RoomId::try_from(room_id_str).map_err(|_| RoomError::RoomNotFound(RoomId::new()))?;

    let room_manager = state.room_manager();

    // First get the current room to ensure it exists
    let _ = room_manager.get_room(&room_id).await?;

    // Update the room with new data
    room_manager.update_room(&room_id, payload.room).await?;

    // Get the updated room snapshot
    let updated_room = room_manager.get_room(&room_id).await?;

    Ok(Json(RoomResponse {
        room: Some(updated_room),
        success: true,
        message: "Room updated successfully".to_string(),
    }))
}

pub async fn delete_room<S>(
    State(state): State<S>,
    Path(room_id_str): Path<String>,
) -> Result<Json<RoomResponse<S::Room>>, RoomError>
where
    S: AppState,
{
    let room_id =
        RoomId::try_from(room_id_str).map_err(|_| RoomError::RoomNotFound(RoomId::new()))?;

    let room_manager = state.room_manager();

    // Get the room snapshot before deleting
    let room = room_manager.get_room(&room_id).await?;

    // Delete the room
    room_manager.remove_room(&room_id).await?;

    Ok(Json(RoomResponse {
        room: Some(room),
        success: true,
        message: "Room deleted successfully".to_string(),
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRoomsResponse<R: RoomLike> {
    pub rooms: Vec<R>,
    pub success: bool,
    pub message: String,
}

pub async fn get_rooms<S>(
    State(state): State<S>,
) -> Result<Json<GetRoomsResponse<S::Room>>, RoomError>
where
    S: AppState,
{
    let room_manager = state.room_manager();
    let rooms = room_manager.get_rooms().await?;

    Ok(Json(GetRoomsResponse {
        rooms,
        success: true,
        message: "Rooms fetched successfully".to_string(),
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpsertRoomResponse<R: RoomLike> {
    pub success: bool,
    pub message: String,
    pub room: Option<R>,
}

pub async fn upsert_room<S>(
    State(state): State<S>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpdateRoomRequest<S::Room>>,
) -> Result<Json<UpsertRoomResponse<S::Room>>, RoomError>
where
    S: AppState,
{
    let room_id =
        RoomId::try_from(room_id_str).map_err(|_| RoomError::RoomNotFound(RoomId::new()))?;

    let room_manager = state.room_manager();

    room_manager
        .update_room_with(&room_id, |room| {
            *room = payload.room;
            Ok(())
        })
        .await?;

    Ok(Json(UpsertRoomResponse {
        success: true,
        message: "Room upserted successfully".to_string(),
        room: None,
    }))
}
