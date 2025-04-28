use async_trait::async_trait;
use serde::Serialize;

use crate::room::{RoomError, client_id::ClientId};

use super::{MessageBroker, MessageBrokerError};

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
    async fn send<P>(
        &self,
        recipients: &[ClientId],
        msg_name: &str,
        payload: P,
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
    async fn broadcast<P>(
        &self,
        room_id: &str,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
    async fn broadcast_all<P>(
        &self,
        msg_name: &str,
        payload: P,
        exclude: &[ClientId],
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
}
