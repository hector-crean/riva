use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug)]
pub enum PresenceError {
    #[error("Invalid presence update data: {0}")]
    InvalidUpdate(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    // Add other specific presence errors
}

// Represents the data associated with a single client's presence in the room.
pub trait PresenceLike: Serialize + Send + Sync + Clone + Debug + 'static {
    /// Returns a unique identifier for this presence data structure type.
    fn presence_type_id(&self) -> &'static str;

    /// Updates this presence based on incoming data.
    /// The `data` often comes from a client's `updatePresence` call.
    /// Returns `true` if the presence state actually changed.
    fn update(&mut self, data: serde_json::Value) -> Result<bool, PresenceError>;

    /// Merges this presence with another, typically used when resolving state
    /// or initializing. Often keeps the most recent non-null values.
    /// Returns `true` if the presence state actually changed.
    fn merge(&mut self, other: &Self) -> Result<bool, PresenceError>;

    // --- Optional but often useful ---

    /// Returns the last time this presence data was meaningfully updated.
    /// Useful for determining activity or ordering updates.
    fn last_updated(&self) -> DateTime<Utc>;

    /// Serializes the presence data to be sent over the network.
    /// Might be the same as `Serialize`, but allows for transformation if needed.
    fn to_network_format(&self) -> Result<serde_json::Value, PresenceError> {
        serde_json::to_value(self).map_err(PresenceError::SerializationError)
    }

    /// Creates a default presence state for a newly joined client.
    fn default_state() -> Self
    where
        Self: Sized;

    // Consider if an 'is_active' concept is needed here, or if that's
    // determined by the Room based on connection status + last_updated.
    // fn is_active(&self) -> bool;
}
