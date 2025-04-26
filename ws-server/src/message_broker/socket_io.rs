use async_trait::async_trait;
use serde::Serialize;

use crate::room::{RoomError, client_id::ClientId};

use super::MessageBroker;

#[derive(Clone)]
pub struct SocketIoMessageBroker {
    // Implementation details here
}

impl SocketIoMessageBroker {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MessageBroker for SocketIoMessageBroker {
    // Add Send + Sync + 'static for broad usability (e.g., Arc<dyn Trait>)
    /// Associated error type for operations.
    type Error = RoomError;
    /// Sends a message directly to one or more specific clients.
    async fn send<P>(
        &self,
        recipients: &[ClientId],
        msg_name: &str,
        payload: P,
    ) -> Result<(), Self::Error>
    where
        P: Serialize + Send + Sync,
    {
        todo!()
    }

    /// Broadcasts a message to all clients in a specific room, potentially excluding some.
    async fn broadcast<P>(
        &self,
        room_id: &str,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), Self::Error>
    where
        P: Serialize + Send + Sync,
    {
        todo!()
    }

    /// Broadcasts to all connected clients (might not be applicable/efficient for all backends).
    async fn broadcast_all<P>(
        &self,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), Self::Error>
    where
        P: Serialize + Send + Sync,
    {
        todo!()
    }
}
