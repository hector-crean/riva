use crate::room::room_id::RoomId;
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;
use serde_json::{json, to_string};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender};

use super::{MessageBroker, MessageBrokerError};

type ClientId = String;
pub struct WebSocketBroker {
    // Maps client IDs to their message senders
    clients: Arc<RwLock<HashMap<ClientId, Sender<Message>>>>,
    // Maps room IDs to sets of client IDs
    rooms: Arc<RwLock<HashMap<RoomId, Vec<ClientId>>>>,
}

impl WebSocketBroker {
    pub fn new() -> Self {
        WebSocketBroker {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new client connection
    pub async fn register_client(&self, client_id: ClientId, socket: WebSocket) {
        let (tx, mut rx) = mpsc::channel::<Message>(100);

        // Store the sender in our clients map
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.clone(), tx);
        }

        // Spawn a task to forward messages to the client's WebSocket
        let clients_clone = self.clients.clone();
        tokio::spawn(async move {
            let mut socket_send = socket;

            while let Some(message) = rx.recv().await {
                if socket_send.send(message).await.is_err() {
                    // Connection closed, remove client
                    let mut clients = clients_clone.write().await;
                    clients.remove(&client_id);
                    break;
                }
            }
        });
    }

    /// Add a client to a room
    pub async fn join_room(&self, client_id: ClientId, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(room_id.to_string()).or_insert_with(Vec::new);
        if !room.contains(&client_id) {
            room.push(client_id);
        }
    }

    /// Remove a client from a room
    pub async fn leave_room(&self, client_id: &ClientId, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.retain(|id| id != client_id);
        }
    }
}

#[async_trait]
impl MessageBroker for WebSocketBroker {
    async fn send<P>(
        &self,
        recipients: &[ClientId],
        msg_name: &str,
        payload: P,
    ) -> Result<(), MessageBrokerError>
    where
        P: Serialize + Send + Sync,
    {
        let message_json = json!({
            "event": msg_name,
            "data": payload
        });

        let message_str =
            to_string(&message_json).map_err(|_| MessageBrokerError::MessageBrokerError)?;

        let clients = self.clients.read().await;

        for client_id in recipients {
            if let Some(sender) = clients.get(client_id) {
                if sender
                    .send(Message::Text(message_str.clone()))
                    .await
                    .is_err()
                {
                    // We could handle this error, but for now we'll just continue
                    // as the client might have disconnected
                }
            }
        }

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
        let rooms = self.rooms.read().await;

        if let Some(room_clients) = rooms.get(room_id) {
            // Filter out excluded clients
            let recipients: Vec<ClientId> = room_clients
                .iter()
                .filter(|client_id| !exclude.contains(client_id))
                .cloned()
                .collect();

            // Use the send method to deliver the message
            self.send(&recipients, msg_name, payload).await?;
        }

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
        let clients = self.clients.read().await;

        // Get all client IDs except excluded ones
        let recipients: Vec<ClientId> = clients
            .keys()
            .filter(|client_id| !exclude.contains(client_id))
            .cloned()
            .collect();

        // Use the send method to deliver the message
        self.send(&recipients, msg_name, payload).await?;

        Ok(())
    }
}
