use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::SocketIo;
use tracing::{error, info};
use ts_rs::TS;

use crate::{
    Application, Room,
    message::{Message, ServerMessage},
    presentation::Presentation,
    room::{RoomLike, room_id::RoomId},
};

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct CreateRoomRequest {
    project_id: Option<String>,
    organisation_id: Option<String>,
    room_type: String,
    room_name: String,
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
    State(Application { rooms, .. }): State<Application>,
    Json(payload): Json<CreateRoomRequest>,
) -> Json<CreateRoomResponse> {
    let room_id = RoomId::new();
    let mut rooms_state_guard = rooms.write().await;

    // Check if room already exists
    if rooms_state_guard.contains_key(&room_id) {
        return Json(CreateRoomResponse {
            room_id,
            success: false,
            message: "Room already exists".to_string(),
        });
    }

    let CreateRoomRequest {
        project_id,
        organisation_id,
        room_type,
        room_name,
        slide_data,
    } = payload;

    // Create room based on type
    let room = match room_type.as_str() {
        "presentation" => {
            let slide_data = slide_data.unwrap_or_default();
            Room::Presentation(Presentation::new(
                room_name,
                organisation_id,
                project_id,
                0, // Start at first slide
                slide_data,
            ))
        }
        // Add other room types here as needed
        _ => {
            return Json(CreateRoomResponse {
                room_id,
                success: false,
                message: format!("Unsupported room type: {room_type}"),
            });
        }
    };

    // Insert the new room
    rooms_state_guard.insert(room_id.clone(), room);

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
    State(Application { rooms, .. }): State<Application>,
) -> Json<GetRoomsResponse> {
    let state_guard = rooms.read().await;
    let rooms: Vec<(RoomId, Room)> = state_guard
        .iter()
        .map(|(id, room)| (id.clone(), room.clone()))
        .collect();
    Json(GetRoomsResponse { rooms })
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct GetRoomResponse {
    room: Option<Room>,
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct RoomError {
    success: bool,
    message: String,
    room_id: Option<RoomId>,
    status_code: u16,
}

impl RoomError {
    fn invalid_room_id(room_id_str: String) -> Self {
        Self {
            success: false,
            message: format!("Invalid room ID format: {room_id_str}"),
            room_id: None,
            status_code: 400,
        }
    }

    fn room_not_found(room_id: RoomId) -> Self {
        Self {
            success: false,
            message: "Room not found".to_string(),
            room_id: Some(room_id),
            status_code: 404,
        }
    }

    fn unsupported_room_type(room_type: String) -> Self {
        Self {
            success: false,
            message: format!("Unsupported room type: {room_type}"),
            room_id: None,
            status_code: 400,
        }
    }
}

impl axum::response::IntoResponse for RoomError {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::from_u16(self.status_code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}

// Update get_room handler to use the new error type
pub async fn get_room(
    State(Application { rooms, .. }): State<Application>,
    Path(room_id_str): Path<String>,
) -> Result<Json<GetRoomResponse>, RoomError> {
    let state_guard = rooms.read().await;

    // Parse the room_id from the path parameter
    let room_id = RoomId::try_from(room_id_str.clone())
        .map_err(|_| RoomError::invalid_room_id(room_id_str))?;

    // Look up the room in the state
    match state_guard.get(&room_id) {
        Some(room) => Ok(Json(GetRoomResponse {
            room: Some(room.clone()),
            success: true,
            message: "Room found".to_string(),
        })),
        None => Err(RoomError::room_not_found(room_id)),
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct UpdateRoomRequest {
    room: Room,
}

// Update update_room handler
pub async fn update_room(
    State(Application { rooms, .. }): State<Application>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpdateRoomRequest>,
) -> Result<Json<GetRoomResponse>, RoomError> {
    let mut state_guard = rooms.write().await;

    // Parse the room_id from the path parameter
    let room_id = RoomId::try_from(room_id_str.clone())
        .map_err(|_| RoomError::invalid_room_id(room_id_str))?;

    // Check if the room exists
    if !state_guard.contains_key(&room_id) {
        return Err(RoomError::room_not_found(room_id));
    }

    // Update the room
    state_guard.insert(room_id.clone(), payload.room);

    Ok(Json(GetRoomResponse {
        room: state_guard.get(&room_id).cloned(),
        success: true,
        message: "Room updated successfully".to_string(),
    }))
}

// Update delete_room handler
pub async fn delete_room(
    State(Application { rooms, .. }): State<Application>,
    Path(room_id_str): Path<String>,
) -> Result<Json<GetRoomResponse>, RoomError> {
    let mut state_guard = rooms.write().await;

    // Parse the room_id from the path parameter
    let room_id = RoomId::try_from(room_id_str.clone())
        .map_err(|_| RoomError::invalid_room_id(room_id_str))?;

    // Remove the room and return the result
    match state_guard.remove(&room_id) {
        Some(room) => Ok(Json(GetRoomResponse {
            room: Some(room),
            success: true,
            message: "Room deleted successfully".to_string(),
        })),
        None => Err(RoomError::room_not_found(room_id)),
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

// Update upsert_room handler
pub async fn upsert_room(
    State(Application { rooms, .. }): State<Application>,
    Path(room_id_str): Path<String>,
    Json(payload): Json<UpsertRoomRequest>,
) -> Result<Json<GetRoomResponse>, RoomError> {
    let mut state_guard = rooms.write().await;

    // Parse the room_id from the path parameter
    let room_id = RoomId::try_from(room_id_str.clone())
        .map_err(|_| RoomError::invalid_room_id(room_id_str))?;

    // Create room based on type
    let room = match payload.room_type.as_str() {
        "presentation" => {
            let room_name = payload.name;
            let organisation_id = Some(payload.organisation_id);
            let slide_data = payload.slide_data.unwrap_or_default();
            Room::Presentation(Presentation::new(
                room_name,
                organisation_id,
                None, // No project_id
                0,    // Start at first slide
                slide_data,
            ))
        }
        // Add other room types here as needed
        _ => return Err(RoomError::unsupported_room_type(payload.room_type)),
    };

    // Insert or update the room
    let exists = state_guard.contains_key(&room_id);
    state_guard.insert(room_id.clone(), room);

    let message = if exists {
        "Room updated successfully".to_string()
    } else {
        "Room created successfully".to_string()
    };

    Ok(Json(GetRoomResponse {
        room: state_guard.get(&room_id).cloned(),
        success: true,
        message,
    }))
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BroadcastEventRequest {
    event: ServerMessage,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BroadcastEventResponse {
    success: bool,
    message: String,
}

// Update broadcast_event handler
pub async fn broadcast_event(
    State(Application { rooms, .. }): State<Application>,
    Path(room_id_str): Path<String>,
    io: axum::extract::Extension<SocketIo>,
    Json(payload): Json<BroadcastEventRequest>,
) -> Result<Json<BroadcastEventResponse>, RoomError> {
    // Parse the room_id from the path parameter
    let room_id = RoomId::try_from(room_id_str.clone())
        .map_err(|_| RoomError::invalid_room_id(room_id_str.clone()))?;

    // Check if the room exists
    let state_guard = rooms.read().await;
    if !state_guard.contains_key(&room_id) {
        return Err(RoomError::room_not_found(room_id));
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
    match io.within(room_id_str).emit("message", &message).await {
        Ok(()) => {
            info!(
                room_id = %room_id.as_str(),
                "Event broadcasted to room"
            );
            Ok(Json(BroadcastEventResponse {
                success: true,
                message: "Event broadcasted successfully".to_string(),
            }))
        }
        Err(err) => {
            error!(
                room_id = %room_id.as_str(),
                error = %err,
                "Failed to broadcast event"
            );

            let error = RoomError {
                success: false,
                message: format!("Failed to broadcast event: {err}"),
                room_id: Some(room_id),
                status_code: 500,
            };

            Err(error)
        }
    }
}
