use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;


use ts_rs::TS;

use crate::RoomId;



#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"JoinRoom\"")]
pub struct JoinRoom;

impl JoinRoom {
    pub const EVENT_NAME: &'static str = "JoinRoom";
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"LeaveRoom\"")]
pub struct LeaveRoom;

impl LeaveRoom {
    pub const EVENT_NAME: &'static str = "LeaveRoom";
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"Drawing\"")]
pub struct Drawing;

impl Drawing {
    pub const EVENT_NAME: &'static str = "Drawing";
}

// Define payload types
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct JoinRoomPayload {
    pub room_id: RoomId,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct LeaveRoomPayload {
    room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct DrawingPayload {
   shape: Vec<u32>
}



// Add new  types and payloads for slide management
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"SlideChange\"")]
pub struct SlideChange;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"SlideData\"")]
pub struct SlideData;



#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct SlideChangePayload {
    room: String,
    slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct SlideDataPayload {
    room: String,
    slide_index: usize,
    data: Value,
}




// Client-to-server message structure
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ClientMessage<E, T>
where
    E: TS,
    T: TS,
{
    #[serde(rename = "type")]
    pub _type: E,
    pub payload: T,
    // Client-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,  // Optional client identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>, // For correlating requests with responses
}

// Server-to-client message structure
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ServerMessage<E, T>
where
    E: TS,
    T: TS,
{
    #[serde(rename = "type")]
    pub _type: E,
    pub payload: T,
    // Server-specific fields
    pub datetime: DateTime<Utc>,                // Server timestamp for the 
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,     // Who triggered this  (for broadcasts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,    // Correlation ID matching client request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<bool>,       // Indicates if this is a broadcast message
}

// Then update your type aliases to use the appropriate message type
pub type ClientJoinRoom = ClientMessage<JoinRoom, JoinRoomPayload>;
pub type ClientLeaveRoom = ClientMessage<LeaveRoom, LeaveRoomPayload>;
pub type ClientSlideChange = ClientMessage<SlideChange, SlideChangePayload>;
pub type ClientSlideData = ClientMessage<SlideData, SlideDataPayload>;

pub type ServerJoinRoom = ServerMessage<JoinRoom, JoinRoomPayload>;
pub type ServerLeaveRoom = ServerMessage<LeaveRoom, LeaveRoomPayload>;

// Update the  enum to include new message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum RivaWsMessage {
    JoinRoom(ServerJoinRoom),
    LeaveRoom(ServerLeaveRoom),
    SlideChange(ClientSlideChange),
    SlideData(ClientSlideData),
}