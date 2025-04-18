use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::RoomError;

// Represents messages originating FROM the client TO the server
pub trait ClientMessageLike: for<'de> Deserialize<'de> + Send + Sync + Debug + 'static {
    fn name(&self) -> &'static str; // e.g., "updatePresence", "updateStorage"
}

// Represents messages originating FROM the server TO the client
pub trait ServerMessageLike: Serialize + Send + Sync + Debug + 'static {
    fn name(&self) -> &'static str; // e.g., "presenceUpdated", "storageUpdated"
    fn to_json(&self) -> Result<serde_json::Value, RoomError> {
         serde_json::to_value(self).map_err(RoomError::SerializationError)
    }
}