pub mod presence;
pub mod room_manager;
pub mod storage;
pub mod transaction;
use std::collections::HashMap;

use crate::message::{ClientMessageTypeLike, Message, ServerMessageTypeLike};
use crate::message_broker::{MessageBroker, MessageBrokerError};
use crate::presentation::Presentation;
use chrono::{DateTime, Utc};
use client_id::ClientId;
use presence::PresenceLike;
use room_id::RoomId;
use serde::{Deserialize, Serialize};
use storage::{StorageError, StorageLike};
use ts_rs::TS;
pub mod client_id;
pub mod room_id;

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
    RoomNotFound(RoomId), // Add other specific room errors
    #[error("Room already exists: {0}")]
    RoomAlreadyExists(RoomId),
    #[error("Message broker error: {0}")]
    MessageBrokerError(MessageBrokerError),
}

/// Describes the outcome of processing a client message (transaction).
/// This informs the room manager how to update clients.
#[derive(Debug)]
pub enum TransactionOutcome<ServerMsg: ServerMessageTypeLike, StorageDiff> {
    /// No action needed, or handled internally (e.g., direct response to sender).
    None,
    /// Broadcast a message to all clients (optionally excluding the sender).
    Broadcast {
        message: Message<ServerMsg>,
        exclude_sender: bool,
    },
    /// Broadcast a storage diff/update to all clients (optionally excluding sender).
    /// The exact content depends on the StorageLike implementation.
    BroadcastStorageUpdate {
        diff: StorageDiff,
        exclude_sender: bool,
    },
    /// Send a message to specific clients.
    SendTo {
        clients: Vec<ClientId>,
        message: Message<ServerMsg>,
    },
    /// Multiple actions required.
    Multiple(Vec<TransactionOutcome<ServerMsg, StorageDiff>>),
}

/// Represents the operational capabilities of a collaborative room.
pub trait RoomLike: Send + Sync + 'static + Default {
    // --- Associated Types ---
    type Storage: StorageLike;
    type Presence: PresenceLike;
    type ClientMessageType: ClientMessageTypeLike;
    type ServerMessageType: ServerMessageTypeLike;
    type ClientMetadata: Send + Sync + Clone + Debug + 'static; // Data per connection

    // --- Basic Properties ---
    fn id(&self) -> &RoomId;
    fn room_type(&self) -> &'static str;
    // Optional context IDs
    // fn organisation_id(&self) -> Option<&str>;
    // fn project_id(&self) -> Option<&str>;

    // --- Core State Access ---
    fn snapshot(&self) -> Self;

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
    ) -> Result<(), RoomError>;

    /// Called when a client disconnects or leaves the room instance.
    /// Should clean up presence and potentially trigger broadcasts.
    /// Returns the metadata of the removed client, if it existed.
    fn remove_client(&mut self, client_id: &ClientId) -> Result<Self::ClientMetadata, RoomError>;

    /// Checks if any clients are currently connected to this room instance.
    fn is_empty(&self) -> bool;

    /// Timestamp of when this room instance was created or loaded.
    fn created_at(&self) -> DateTime<Utc>;

    /// Timestamp of the last significant activity (e.g., message processed, client joined/left).
    fn last_activity_at(&self) -> DateTime<Utc>;

    fn apply_client_message(
        &mut self,
        client_id: &ClientId,
        message: Message<Self::ClientMessageType>,
    ) -> Result<
        TransactionOutcome<Self::ServerMessageType, <Self::Storage as StorageLike>::Diff>,
        RoomError,
    >;
}
