pub mod presence;
pub mod storage;
pub mod message;
pub mod room_manager;

pub mod transaction;
pub mod crdt;
pub mod network_communicator;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use message::{ClientMessageLike, ServerMessageLike};
use network_communicator::NetworkCommunicator;
use presence::PresenceLike;
use presentation::Presentation;
use room_id::RoomId;
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use storage::{StorageError, StorageLike};
use ts_rs::TS;

pub mod presentation;
pub mod room_id;

type ClientId = String;


use std::fmt::Debug;
use thiserror::Error; // Add thiserror for structured errors




// Define potential errors for room operations
#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Client '{0}' not found")]
    ClientNotFound(ClientId),
    #[error("Storage operation failed: {0}")]
    StorageError(#[from] storage::StorageError), // Use the specific error type
    #[error("Presence operation failed: {0}")]
    PresenceError(#[from] presence::PresenceError), // Use the specific error type
    #[error("Transaction failed: {0}")]
    TransactionError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Network/Broadcast error: {0}")]
    NetworkError(String), // Placeholder for actual network errors
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    #[error("Room not foind: {0}")]
    RoomNotFound(RoomId)
    // Add other specific room errors
}




/// Describes the outcome of processing a client message (transaction).
/// This informs the room manager how to update clients.
#[derive(Debug)]
pub enum TransactionOutcome<SrvMsg: ServerMessageLike, StorageDiff> {
    /// No action needed, or handled internally (e.g., direct response to sender).
    None,
    /// Broadcast a message to all clients (optionally excluding the sender).
    Broadcast { message: SrvMsg, exclude_sender: bool },
    /// Broadcast a storage diff/update to all clients (optionally excluding sender).
    /// The exact content depends on the StorageLike implementation.
    BroadcastStorageUpdate { diff: StorageDiff, exclude_sender: bool },
    /// Send a message to specific clients.
    SendTo { clients: Vec<ClientId>, message: SrvMsg },
    /// Multiple actions required.
    Multiple(Vec<TransactionOutcome<SrvMsg, StorageDiff>>),
}

/// Represents the operational capabilities of a collaborative room.
pub trait RoomLike: Send + Sync + 'static {
    /// Unique identifier string for this *type* of room (e.g., "presentation", "document").
    const ROOM_TYPE: &'static str;

    // --- Associated Types ---
    type Storage: StorageLike;
    type Presence: PresenceLike;
    type ClientMessage: ClientMessageLike;
    type ServerMessage: ServerMessageLike;
    type ClientMetadata: Send + Sync + Clone + Debug + 'static; // Data per connection
    type RoomError: std::error::Error + From<RoomError> + Send + Sync + 'static; // Custom error type

    // --- Basic Properties ---
    fn id(&self) -> &RoomId;
    fn room_type(&self) -> &'static str { Self::ROOM_TYPE }
    // Optional context IDs
    // fn organisation_id(&self) -> Option<&str>;
    // fn project_id(&self) -> Option<&str>;

    // --- Core State Access ---

    /// Provides immutable access to the room's persistent storage.
    fn storage(&self) -> &Self::Storage;

    /// Provides mutable access to the room's persistent storage.
    /// **Use with caution.** Prefer modifying storage via `apply_client_message`.
    /// Primarily intended for internal use (e.g., loading state, applying CRDT merges).
    fn storage_mut(&mut self) -> &mut Self::Storage;

    /// Gets the presence data for a specific client.
    fn get_presence(&self, client_id: &ClientId) -> Option<&Self::Presence>;

    /// Gets the entire presence map (client_id -> presence data).
    fn get_all_presence(&self) -> HashMap<ClientId, Self::Presence>; // Return owned map for flexibility

    /// Gets metadata associated with a specific client connection.
    fn get_client_metadata(&self, client_id: &ClientId) -> Option<&Self::ClientMetadata>;

    /// Gets IDs of all currently connected clients.
    fn get_connected_clients(&self) -> Vec<ClientId>;


    // --- Lifecycle and Client Management ---

    /// Called when a client successfully connects and joins this room instance.
    /// Should initialize presence and potentially metadata.
    /// `metadata` is the initial info provided by the client on connection.
    /// Returns `Ok(())` or an error if the client cannot be added.
    fn add_client(
        &mut self,
        client_id: ClientId, // Use the actual client identifier
        metadata: Self::ClientMetadata,
        // Add socket ref or similar if needed for direct communication setup
    ) -> Result<(), Self::RoomError>;

    /// Called when a client disconnects or leaves the room instance.
    /// Should clean up presence and potentially trigger broadcasts.
    /// Returns the metadata of the removed client, if it existed.
    fn remove_client(&mut self, client_id: &ClientId) -> Result<Self::ClientMetadata, Self::RoomError>;

    /// Checks if any clients are currently connected to this room instance.
    fn is_empty(&self) -> bool;

    /// Timestamp of when this room instance was created or loaded.
    fn created_at(&self) -> DateTime<Utc>;

    /// Timestamp of the last significant activity (e.g., message processed, client joined/left).
    fn last_activity_at(&self) -> DateTime<Utc>;

    // --- Core Operation Logic ---

    /// Processes a message received from a specific client.
    /// This is the primary entry point for handling user actions that modify room state.
    /// It should:
    /// 1. Validate the message.
    /// 2. Update presence and/or storage state.
    /// 3. Determine what notifications need to be sent to clients.
    /// `client_id` identifies the sender.
    /// Returns a `TransactionOutcome` describing broadcasts/sends needed.
    fn apply_client_message(
        &mut self,
        client_id: &ClientId,
        message: Self::ClientMessage,
    ) -> Result<TransactionOutcome<Self::ServerMessage, <Self::Storage as StorageLike>::Diff>, Self::RoomError>;


    // --- Communication Helpers (to be implemented by the concrete Room type) ---
    // These methods interact with the underlying network layer (e.g., socketioxide).
    // The trait defines the *intent*, the implementation handles the *how*.

    /// Sends a message to specific clients identified by their IDs.
    fn send_to(
       &self,
       recipients: &[ClientId],
       message: &Self::ServerMessage,
       communicator: &NetworkCommunicator
    ) -> Result<(), Self::RoomError>;

    /// Broadcasts a message to all connected clients, potentially excluding some.
    fn broadcast(
        &self,
        message: &Self::ServerMessage,
        exclude_clients: &[ClientId],
        communicator: &NetworkCommunicator

    ) -> Result<(), Self::RoomError>;

    /// Broadcasts a storage update (diff/ops) to connected clients.
    fn broadcast_storage_update(
        &self,
        diff: &<Self::Storage as StorageLike>::Diff,
        exclude_clients: &[ClientId],
        network_communicator: NetworkCommunicator
    ) -> Result<(), Self::RoomError>;


    // --- Persistence (Optional but Recommended) ---

    /// Persists the current state of the room (primarily storage) to durable storage.
    async fn persist_state(&self) -> Result<(), Self::RoomError>;

    // Loading state is often handled outside the instance, e.g., in a factory/manager
    // static fn load_state(room_id: &RoomId, ...) -> Result<Self, Self::RoomError>;

    // --- Housekeeping ---
    // Performs periodic cleanup or checks (e.g., removing inactive presence, checking expiry).
    // fn housekeeping(&mut self) -> Result<(), Self::RoomError>;
}






// --- Enum Wrapper ---
// Your Room enum remains largely the same, but now delegates calls
// to the variant which implements the refined RoomLike trait.

#[derive(Clone, Debug)] // Ensure variants are Clone if Room is Clone
pub enum Room {
    Presentation(Presentation),
    // Document(DocumentRoom), // Example of another type
}

impl Room {
     // Helper to delegate calls - macro could simplify this
    fn dispatch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn RoomLike<
            Storage = dyn StorageLike,
            Presence = dyn PresenceLike,
            ClientMessage = dyn ClientMessageLike,
            ServerMessage = dyn ServerMessageLike,
            ClientMetadata = (),
            RoomError = dyn std::error::Error,
        >) -> R,
    {
        match self {
            Room::Presentation(p) => f(p),
            // Room::Document(d) => f(d),
        }
    }

     fn dispatch_mut<F, R>(&mut self, f: F) -> R
     where
        F: FnOnce(&mut dyn RoomLike<
            Storage = dyn StorageLike,
            Presence = dyn PresenceLike,
            ClientMessage = dyn ClientMessageLike,
            ServerMessage = dyn ServerMessageLike,
            ClientMetadata = (),
            RoomError = dyn std::error::Error,
            // Add other associated types if needed for the function signature
         >) -> R,
     {
         match self {
             Room::Presentation(p) => f(p),
             // Room::Document(d) => f(d),
         }
     }

   
}

impl Room {
    // Existing dispatch methods...
    
    // Example methods that use dispatch and dispatch_mut
    
    pub fn id(&self) -> &RoomId {
        self.dispatch(|room| room.id())
    }
    
    pub fn room_type(&self) -> &'static str {
        self.dispatch(|room| room.room_type())
    }
    
    pub fn add_client(&mut self, client_id: ClientId, metadata: ()) -> Result<(), Box<dyn std::error::Error>> {
        self.dispatch_mut(|room| {
            room.add_client(client_id, metadata)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
    }
    
    pub fn apply_client_message<ClientMsg: ClientMessageLike, ServerMsg: ServerMessageLike>(
        &mut self, 
        client_id: &ClientId, 
        message: ClientMsg
    ) -> Result<TransactionOutcome<ServerMsg, Box<dyn std::any::Any>>, Box<dyn std::error::Error>> {
        self.dispatch_mut(|room| {
            room.apply_client_message(client_id, message)
                .map(|outcome| {
                    // Convert the specific TransactionOutcome to a generic one
                    // This would need proper conversion logic based on your types
                    // This is simplified for example purposes
                    match outcome {
                        TransactionOutcome::None => TransactionOutcome::None,
                        // Other conversions...
                        _ => TransactionOutcome::None // Simplified
                    }
                })
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
    }
    
    // Other methods that delegate to the underlying room implementation...
}





