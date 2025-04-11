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
#[ts(export, type = "\"RequestSlideChange\"")]
pub struct RequestSlideChange;


#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RequestSlideChangePayload {
    slide_index: usize,
}



// Add new  types and payloads for slide management
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"SlideChanged\"")]
pub struct SlideChanged;


#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct SlideChangedPayload {
    slide_index: usize,
}



#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"RoomJoined\"")]
pub struct RoomJoined;

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RoomJoinedPayload {
    user_id: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, type = "\"RoomLeft\"")]
pub struct RoomLeft;

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RoomLeftPayload {
    user_id: String,
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





pub type JoinRoomMessage = ClientMessage<JoinRoom, JoinRoomPayload>;
pub type LeaveRoomMessage = ClientMessage<LeaveRoom, LeaveRoomPayload>;
pub type RequestSlideChangeMessage = ClientMessage<RequestSlideChange, RequestSlideChangePayload>;
pub type SlideChangedMessage = ServerMessage<SlideChanged, SlideChangedPayload>;
pub type RoomJoinedMessage = ServerMessage<RoomJoined, RoomJoinedPayload>;
pub type RoomLeftMessage = ServerMessage<RoomLeft, RoomLeftPayload>;


// Update the  enum to include new message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum PresentationRoomMessage {
    //client messages
    JoinRoom(JoinRoomMessage),
    LeaveRoom(LeaveRoomMessage),
    RequestSlideChange(RequestSlideChangeMessage),
    //server messages
    RoomJoined(RoomJoinedMessage),
    RoomLeft(RoomLeftMessage),
    SlideChanged(SlideChangedMessage),
    
}
