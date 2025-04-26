use crate::message::{
    ClientMessage, ClientMessageType, ClientMessageTypeLike, Message, ServerMessage, ServerMessageTypeLike
};
use crate::{
    message::ServerMessageType,
    room::{RoomLike, presence::PresenceLike, storage::StorageLike},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::extract::SocketRef;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::room::{
    RoomError, TransactionOutcome, client_id::ClientId, presence::PresenceError, room_id::RoomId,
    storage::StorageError,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresentationStorage {
    current_slide: usize,
    slide_data: Vec<Value>,
}

impl StorageLike for PresentationStorage {
    type ApplyResult = Self;
    type Diff = json_patch::Patch;

    fn storage_type_id(&self) -> &'static str {
        "presentation"
    }
    fn merge(&mut self, other: &Self) -> Result<Self::ApplyResult, StorageError> {
        todo!()
    }
    fn apply_diff(&mut self, diff: Self::Diff) -> Result<Self::ApplyResult, StorageError> {
        todo!()
    }
    fn diff(&self, other: &Self) -> Result<Self::Diff, StorageError> {
        todo!()
    }
    fn snapshot(&self) -> Result<serde_json::Value, StorageError> {
        todo!()
    }
    fn from_snapshot(snapshot: serde_json::Value) -> Result<Self, StorageError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresentationPresence {
    last_updated: DateTime<Utc>,
}

impl PresenceLike for PresentationPresence {
    fn presence_type_id(&self) -> &'static str {
        "presentation"
    }
    fn update(&mut self, data: serde_json::Value) -> Result<bool, PresenceError> {
        todo!()
    }
    fn merge(&mut self, other: &Self) -> Result<bool, PresenceError> {
        todo!()
    }
    fn default_state() -> Self {
        todo!()
    }
    fn last_updated(&self) -> DateTime<Utc> {
        todo!()
    }

    
}

#[derive(Debug, Clone)] // Add necessary derives
pub struct PresentationClientData {
    pub user_id: String,
    pub name: String,
    // other metadata
}

#[derive(Debug, Clone)]
pub struct Presentation {
    id: RoomId,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    storage: PresentationStorage,
    presence: HashMap<ClientId, PresentationPresence>,
    clients: HashMap<ClientId, PresentationClientData>, // Store metadata here
                                                        // No communicator field
}

impl Presentation {
    pub fn new(
        id: RoomId,
        created_at: DateTime<Utc>,
        last_activity: DateTime<Utc>,
        storage: PresentationStorage,
    ) -> Self {
        Self {
            id,
            created_at,
            last_activity,
            storage,
            presence: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

impl RoomLike for Presentation {
    type Storage = PresentationStorage;
    type Presence = PresentationPresence;
    type ClientMessageType = PresentationClientMessage;
    type ServerMessageType = ServerMessageType;
    type ClientMetadata = PresentationClientData;

    fn room_type(&self) -> &'static str {
        "presentation"
    }

    // --- Implement all methods from RoomLike ---

    fn id(&self) -> &RoomId {
        &self.id
    }

    fn storage(&self) -> &Self::Storage {
        &self.storage
    }
    fn storage_mut(&mut self) -> &mut Self::Storage {
        &mut self.storage
    }

    fn get_presence(&self, client_id: &ClientId) -> Option<&Self::Presence> {
        self.presence.get(client_id)
    }

    fn get_all_presence(&self) -> HashMap<ClientId, Self::Presence> {
        self.presence.clone() // Clone the map and its contents
    }

    fn get_client_metadata(&self, client_id: &ClientId) -> Option<&Self::ClientMetadata> {
        self.clients.get(client_id)
    }

    fn get_connected_clients(&self) -> Vec<ClientId> {
        self.clients.keys().cloned().collect()
    }

    fn add_client(
        &mut self,
        client_id: ClientId, // Use the actual client identifier
        metadata: Self::ClientMetadata,
        // Add socket ref or similar if needed for direct communication setup
    ) -> Result<(), RoomError> {
        self.clients.insert(client_id, metadata);
        Ok(())
    }


    fn remove_client(&mut self, client_id: &ClientId) -> Result<Self::ClientMetadata, RoomError> {
        self.clients.remove(client_id).ok_or(RoomError::ClientNotFound(client_id.clone()))
    }

    fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn last_activity_at(&self) -> DateTime<Utc> {
        self.last_activity
    }

    fn apply_client_message(
        &mut self,
        client_id: &ClientId,
        message: Message<Self::ClientMessageType>,
    ) -> Result<
        TransactionOutcome<Self::ServerMessageType, <Self::Storage as StorageLike>::Diff>,
        RoomError,
    > {
        self.last_activity = Utc::now();

        match message.payload {
           
            // Handle other client message types...
            _ => {
                println!("Unhandled message type: {:?}", message.payload.name());
                Ok(TransactionOutcome::None)
            }
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ChangeSlide {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]

#[serde(tag = "type")]
pub enum PresentationClientMessage {
    JoinPresentation,
    LeavePresentation,
    ChangeSlide { slide_index: usize },
}

impl ClientMessageTypeLike for PresentationClientMessage {
    fn name(&self) -> &'static str {
        match self {
            Self::JoinPresentation => "JoinPresentation",
            Self::LeavePresentation => "LeavePresentation",
            Self::ChangeSlide { .. } => "ChangeSlide",
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SlideChanged {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]

#[serde(tag = "type")]
pub enum PresentationServerMessage {
    SlideChanged { slide_index: usize },
}

impl ServerMessageTypeLike for PresentationServerMessage {
    fn name(&self) -> &'static str {
        match self {
            Self::SlideChanged { .. } => "SlideChanged",
        }
    }
}
