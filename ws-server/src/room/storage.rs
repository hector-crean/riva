use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// Define a potential error type for storage operations
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Merge conflict: {0}")]
    MergeConflict(String),
    #[error("Failed to apply diff: {0}")]
    ApplyDiffError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Incompatible storage types")]
    IncompatibleTypes,
    // Add other specific storage errors as needed
}

pub trait StorageLike:
    for<'de> Deserialize<'de> + Serialize + Send + Sync + Clone + Debug + 'static + TS
{
    // Consider using an associated type for Diffs if they aren't always JSON
    // type Diff: for<'de> Deserialize<'de> + Serialize + Send + Sync + Debug + 'static;
    // For simplicity with JSON communication, using Value might be okay initially
    type Diff: for<'de> Deserialize<'de> + Serialize + Send + Sync + Debug + 'static;
    type ApplyResult; // Type returned after applying a diff/merge, maybe includes changesets

    /// Returns a unique identifier for this storage type (e.g., "crdt-map", "json-doc")
    fn storage_type_id(&self) -> &'static str;

    /// Merges changes from another storage instance based on CRDT rules or other logic.
    /// This typically happens when loading persisted state or handling complex sync scenarios.
    /// It might return information about what changed.
    fn merge(&mut self, other: &Self) -> Result<Self::ApplyResult, StorageError>;

    /// Creates a diff representing changes needed to get from `self` to `other`.
    /// The nature of the diff depends on the underlying storage type (e.g., CRDT ops, patch).
    fn diff(&self, other: &Self) -> Result<Self::Diff, StorageError>;

    /// Applies a diff (e.g., received from a client or another server instance) to this storage.
    /// Returns information about the effect of applying the diff.
    fn apply_diff(&mut self, diff: Self::Diff) -> Result<Self::ApplyResult, StorageError>;

    // --- New/Revised Methods ---

    /// Applies a specific operation/mutation originating from a client command.
    /// This is often the primary way storage is modified in response to user actions.
    /// It should return the necessary information to broadcast updates (e.g., the diff/ops applied).
    /// The `Op` type would likely be part of your `ClientMessage` enum.
    // fn apply_op(&mut self, op: Self::Operation) -> Result<Self::Diff, StorageError>;
    // Note: We might handle this within RoomLike::transaction instead of directly here.

    /// Creates a serializable snapshot of the current storage state.
    /// Used for persistence or sending the full state to new clients.
    fn snapshot(&self) -> Result<serde_json::Value, StorageError>;

    /// Creates an instance from a snapshot.
    fn from_snapshot(snapshot: serde_json::Value) -> Result<Self, StorageError>
    where
        Self: Sized;

    // Optional: Explicitly track changes for persistence optimization
    // fn clear_changes_flag(&mut self);
    // fn has_changes(&self) -> bool;
}


