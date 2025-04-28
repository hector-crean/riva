use async_trait::async_trait;
use serde::Serialize;
use std::error::Error as StdError;
pub mod socket_io;

use crate::room::{client_id::ClientId, RoomError}; // Alias for clarity

#[derive(Debug, thiserror::Error)]
pub enum MessageBrokerError {
    #[error("Message broker error: {0}")]
    MessageBrokerError,
}

impl From<MessageBrokerError> for RoomError {
    fn from(error: MessageBrokerError) -> Self {
        RoomError::MessageBrokerError(error)
    }
}

#[async_trait]
pub trait MessageBroker: Send + Sync + 'static {
    // Add Send + Sync + 'static for broad usability (e.g., Arc<dyn Trait>)
    /// Associated error type for operations.

    /// Sends a message directly to one or more specific clients.
    async fn send<P>(
        &self,
        recipients: &[ClientId],
        msg_name: &str,
        payload: P,
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync; // Use generic, serializable payload

    /// Broadcasts a message to all clients in a specific room, potentially excluding some.
    async fn broadcast<P>(
        &self,
        room_id: &str,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync; // Use generic, serializable payload

    /// Broadcasts to all connected clients (might not be applicable/efficient for all backends).
    async fn broadcast_all<P>(
        &self,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync;
}
