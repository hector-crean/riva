use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


use ts_rs::TS;

use crate::RoomId;



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




#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RequestSlideChangePayload {
    slide_index: usize,
}




#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct SlideChangedPayload {
    slide_index: usize,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RoomJoinedPayload {
    user_id: String,
}



#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
pub struct RoomLeftPayload {
    user_id: String,
}


// Client-to-server message structure
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ClientMessage<T>
where
    T: TS,
{
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
pub struct ServerMessage<T>
where
    T: TS,
{
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





pub type JoinRoomMessage = ClientMessage<JoinRoomPayload>;
pub type LeaveRoomMessage = ClientMessage<LeaveRoomPayload>;
pub type RequestSlideChangeMessage = ClientMessage<RequestSlideChangePayload>;
pub type SlideChangedMessage = ServerMessage<SlideChangedPayload>;
pub type RoomJoinedMessage = ServerMessage<RoomJoinedPayload>;
pub type RoomLeftMessage = ServerMessage<RoomLeftPayload>;


// Update the  enum to include new message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
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
