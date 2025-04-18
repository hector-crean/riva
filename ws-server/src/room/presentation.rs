use crate::{events::{presentation::{PresentationClientMessage, PresentationServerMessage}, ServerMessageType}, room::RoomLike};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use serde_json::Value;
use ts_rs::TS;
use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};
use std::sync::Arc;

use super::{network_communicator::NetworkCommunicator, presence::PresenceError, room_id::RoomId, storage::StorageError, ClientId, PresenceLike, RoomError, StorageLike, TransactionOutcome};

#[derive(Debug, Clone, TS, Deserialize, Serialize)]
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
   fn apply_diff(&mut self, diff: Self::Diff) -> Result<Self::ApplyResult, super::storage::StorageError> {
       todo!()
   }
   fn diff(&self, other: &Self) -> Result<Self::Diff, StorageError> {
       todo!()
   }
   fn snapshot(&self) -> Result<serde_json::Value, StorageError> {
       todo!()
   }
   fn from_snapshot(snapshot: serde_json::Value) -> Result<Self, super::storage::StorageError>
       where
           Self: Sized {
       todo!()
   }

    
}



#[derive(Debug, Clone, TS, Deserialize, Serialize)]
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
   
    fn to_network_format(&self) -> Result<Value, PresenceError> {
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


impl RoomLike for Presentation {
    const ROOM_TYPE: &'static str = "presentation";

    type Storage = PresentationStorage;
    type Presence = PresentationPresence;
    type ClientMessage = PresentationClientMessage;
    type ServerMessage = PresentationServerMessage;
    type ClientMetadata = PresentationClientData;
    type RoomError = RoomError; // Use the base one or define a custom one
    type NetworkCommunicator: NetworkCommunicator;

    // --- Implement all methods from RoomLike ---

    fn id(&self) -> &RoomId { &self.id }

    fn storage(&self) -> &Self::Storage { &self.storage }
    fn storage_mut(&mut self) -> &mut Self::Storage { &mut self.storage }

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


    fn add_client(&mut self, client_id: ClientId, metadata: Self::ClientMetadata) -> Result<(), Self::RoomError> {
        if self.clients.contains_key(&client_id) {
            // Handle error or update existing client? Depends on logic.
            // For now, let's assume it's an error or ignore.
             println!("Client {} already exists.", client_id);
             // return Err(RoomError::ClientAlreadyExists(client_id));
        }
        let default_presence = Self::Presence::default_state();
        self.clients.insert(client_id.clone(), metadata);
        self.presence.insert(client_id.clone(), default_presence);
        self.last_activity = Utc::now();
        // Potentially broadcast "userEntered" event here or return it in outcome?
        // For simplicity, let add/remove handle their own broadcasts for now.
        let enter_msg = PresentationServerMessage::UserEntered { user_id: client_id.clone() /* + metadata */ };
        self.broadcast(&enter_msg, &[client_id])?; // Broadcast to others

        Ok(())
    }

    fn remove_client(&mut self, client_id: &ClientId) -> Result<Self::ClientMetadata, Self::RoomError> {
        let metadata = self.clients.remove(client_id).ok_or_else(|| RoomError::ClientNotFound(client_id.clone()))?;
        self.presence.remove(client_id);
        self.last_activity = Utc::now();

        // Broadcast "userLeft" event
         let leave_msg = PresentationServerMessage::UserLeft { user_id: client_id.clone() };
        self.broadcast(&leave_msg, &[])?; // Broadcast to everyone remaining

        Ok(metadata)
    }

     fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn last_activity_at(&self) -> DateTime<Utc> { self.last_activity }


    fn apply_client_message(
        &mut self,
        client_id: &ClientId,
        message: Self::ClientMessage,
    ) -> Result<TransactionOutcome<Self::ServerMessage, <Self::Storage as StorageLike>::Diff>, Self::RoomError> {
        self.last_activity = Utc::now();

        match message {
            PresentationClientMessage::UpdateMyPresence { data } => {
                let presence_entry = self.presence.entry(client_id.clone())
                    .or_insert_with(Self::Presence::default_state); // Ensure presence exists

                if presence_entry.update(data)? { // Use the boolean result from update()
                    // Presence changed, create notification
                    let updated_presence_payload = presence_entry.to_network_format()?;
                     let msg = PresentationServerMessage::PresenceUpdated {
                        client_id: client_id.clone(),
                        presence: updated_presence_payload,
                    };
                    // Broadcast change to others
                    Ok(TransactionOutcome::Broadcast { message: msg, exclude_sender: true })
                } else {
                    // No change, no broadcast needed
                    Ok(TransactionOutcome::None)
                }
            }

            PresentationClientMessage::UpdateSlideContent { slide_id, content_diff } => {
                // 1. Get mutable access to storage
                let storage = self.storage_mut();

                // 2. Apply the change (assuming StorageLike has a method or internal logic)
                // This is where you'd use CRDT apply functions or similar.
                // Let's assume apply_diff returns the actual diff applied (or the input diff)
                // This might involve mapping PresentationClientMessage::UpdateSlideContent
                // to a specific Storage operation/diff format.
                // let storage_diff = storage.apply_specific_update(slide_id, content_diff)?;
                let storage_diff: <Self::Storage as StorageLike>::Diff = content_diff; // Placeholder
                let apply_result = storage.apply_diff(storage_diff)?; // Assume Diff type matches

                
                // Check apply_result if needed.

                // 3. If successful, create outcome to broadcast storage update
                // Use the actual diff that was applied/generated by the storage operation
                Ok(TransactionOutcome::BroadcastStorageUpdate { diff: storage_diff, exclude_sender: true })
            }

            // Handle other client message types...
            _ => {
                 println!("Unhandled message type: {:?}", message.name());
                 Ok(TransactionOutcome::None)
            }
        }
    }

    // --- Communication Implementation ---
     fn send_to(
       &self,
       recipients: &[ClientId],
       message: &Self::ServerMessage,
       communicator: &NetworkCommunicator,  // Added parameter
    ) -> Result<(), Self::RoomError> {
        let payload = message.to_json()?;
        communicator.send(recipients, &message.name(), payload)
    }

    fn broadcast(
        &self,
        message: &Self::ServerMessage,
        exclude_clients: &[ClientId],
        communicator: &NetworkCommunicator,  // Added parameter
    ) -> Result<(), Self::RoomError> {
        let payload = message.to_json()?;
        communicator.broadcast(self.id.as_str(), &message.name(), payload, exclude_clients)
    }

    fn broadcast_storage_update(
        &self,
        diff: &<Self::Storage as StorageLike>::Diff,
        exclude_clients: &[ClientId],
        communicator: &NetworkCommunicator,  // Added parameter
    ) -> Result<(), Self::RoomError> {
        let event_name = "storage:update";
        let payload = serde_json::to_value(diff).map_err(RoomError::SerializationError)?;
        communicator.broadcast(self.id.as_str(), event_name, payload, exclude_clients)
    }

     // --- Persistence Implementation ---
     async fn persist_state(&self) -> Result<(), Self::RoomError> {
        let snapshot = self.storage.snapshot()?;
        println!("SIMULATE PERSIST: Room: {:?}, Snapshot: {:?}", self.id(), snapshot);
        // Actual database/file write logic here
        // e.g., db::save_room_storage(self.id(), snapshot).await.map_err(RoomError::PersistenceError)
        Ok(())
    }

}
