use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    presentation::{PresentationClientMessage, PresentationServerMessage},
    room::{presence::PresenceLike, room_id::RoomId, storage::StorageLike, RoomError},
};

// Represents messages originating FROM the client TO the server
pub trait ClientMessageTypeLike: for<'de> Deserialize<'de> + Send + Sync + Debug + 'static + TS {
    fn name(&self) -> &'static str; // e.g., "updatePresence", "updateStorage"
}

// Represents messages originating FROM the server TO the client
pub trait ServerMessageTypeLike: Serialize + Send + Sync + Debug + 'static + TS{
    fn name(&self) -> &'static str; // e.g., "presenceUpdated", "storageUpdated"
    fn to_json(&self) -> Result<serde_json::Value, RoomError> {
        serde_json::to_value(self).map_err(RoomError::SerializationError)
    }
}

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





#[derive(Debug, Clone, TS)]
#[ts(export)]
pub enum ClientMessageType<Presence: PresenceLike + TS> {
    PresenceUpdated(Presence),
    StorageUpdated,
}

impl<Presence: PresenceLike> ClientMessageTypeLike for ClientMessageType<Presence> {
    fn name(&self) -> &'static str {
        match self {
            ClientMessageType::PresenceUpdated(_) => "presence",
            ClientMessageType::StorageUpdated => "storage",
        }
    }
}

pub type ClientMessage = Message<ClientMessageType>;


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
pub enum ServerMessageType {
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
}

impl ServerMessageTypeLike for ServerMessageType {
    fn name(&self) -> &'static str {
        match self {
            ServerMessageType::RoomCreated { .. } => "RoomCreated",
            ServerMessageType::RoomDeleted { .. } => "RoomDeleted",
            ServerMessageType::RoomJoined { .. } => "RoomJoined",
            ServerMessageType::RoomLeft { .. } => "RoomLeft",
            ServerMessageType::StorageUpdated { .. } => "StorageUpdated",
            ServerMessageType::CommentCreated { .. } => "CommentCreated",
            ServerMessageType::CommentEdited { .. } => "CommentEdited",
            ServerMessageType::CommentDeleted { .. } => "CommentDeleted",
            ServerMessageType::CommentReactionAdded { .. } => "CommentReactionAdded",
            ServerMessageType::CommentReactionRemoved { .. } => "CommentReactionRemoved",
            ServerMessageType::ThreadCreated { .. } => "ThreadCreated",
            ServerMessageType::ThreadDeleted { .. } => "ThreadDeleted",
            ServerMessageType::ThreadMetadataUpdated { .. } => "ThreadMetadataUpdated",
            ServerMessageType::Notification { .. } => "Notification",
        }
    }
}

pub type ServerMessage = Message<ServerMessageType>;


