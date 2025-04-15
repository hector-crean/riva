pub mod presentation;
pub mod video;

use chrono::{DateTime, Utc};
use presentation::{PresentationCommand, PresentationEvent};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::room_id::RoomId;


// Client-to-server message structure
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CommandMessage<T>
where
    T: TS,
{
    pub payload: T,
    // Client-specific fields
    pub room_id: RoomId,
}


// Server-to-client message structure
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EventMessage<T>
where
    T: TS,
{
    pub room_id: RoomId,
    pub payload: T,
    // Server-specific fields
    pub datetime: DateTime<Utc>, // Server timestamp for the
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>, // Who triggered this  (for broadcasts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>, // Correlation ID matching client request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<bool>, // Indicates if this is a broadcast message
}





#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(untagged)]
pub enum CommandType {
    Presentation(PresentationCommand),
    Video(VideoCommand),
}

pub type Command = CommandMessage<CommandType>;



#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(untagged)]
pub enum EventType {
    Presentation(PresentationEvent),
    Video(VideoEvent),
}

pub type Event = EventMessage<EventType>;







