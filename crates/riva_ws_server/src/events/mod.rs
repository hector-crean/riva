pub mod presentation;

use chrono::{DateTime, Utc};
use presentation::{PresentationCommand, PresentationEvent};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::{room_id::RoomId, RoomEventLike};


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Message<T>
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
pub enum ClientEvent {
    Presentation(PresentationCommand),
}

pub type ClientMessage = Message<ClientEvent>;




#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(untagged)]
pub enum ServerEvent {
    RoomCreated {
        room_id: RoomId,
    },
    RoomDeleted {
        room_id: RoomId,
    },
    RoomJoined {
        room_id: RoomId,
        socket_id: String,
        user_info: Option<UserInfo>,
        entered_at: DateTime<Utc>,
    },
    RoomLeft {
        room_id: RoomId,
        socket_id: String,
    },
    StorageUpdated,
    CommentCreated,
    CommentEdited,
    CommentDeleted,
    CommentReactionAdded,
    CommentReactionRemoved,
    ThreadCreated,
    ThreadDeleted,
    ThreadMetadataUpdated,
    Notification,
    // event types associated with a specific room type. Liveblocks has quite an interesting pattern of
    // just sating stoage updated, which is just a prompt for the client to fetch the latest state.
    Presentation(PresentationEvent),

}

impl RoomEventLike for ServerEvent {
    fn event_name(&self) -> &'static str {
        match self {
            ServerEvent::Presentation(event) => event.event_name(),
            ServerEvent::RoomCreated { .. } => "RoomCreated",
            ServerEvent::RoomDeleted { .. } => "RoomDeleted",
            ServerEvent::RoomJoined { .. } => "RoomJoined",
            ServerEvent::RoomLeft { .. } => "RoomLeft",
            ServerEvent::StorageUpdated { .. } => "StorageUpdated",
            ServerEvent::CommentCreated { .. } => "CommentCreated",
            ServerEvent::CommentEdited { .. } => "CommentEdited",
            ServerEvent::CommentDeleted { .. } => "CommentDeleted",
            ServerEvent::CommentReactionAdded { .. } => "CommentReactionAdded",
            ServerEvent::CommentReactionRemoved { .. } => "CommentReactionRemoved",
            ServerEvent::ThreadCreated { .. } => "ThreadCreated",
            ServerEvent::ThreadDeleted { .. } => "ThreadDeleted",
            ServerEvent::ThreadMetadataUpdated { .. } => "ThreadMetadataUpdated",
            ServerEvent::Notification { .. } => "Notification",
        }
    }
}

pub type ServerMessage = Message<ServerEvent>;
