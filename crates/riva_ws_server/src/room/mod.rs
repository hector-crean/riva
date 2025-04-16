pub mod presence;
pub mod storage;

pub mod transaction;
pub mod crdt;
use chrono::{DateTime, Utc};
use room_id::RoomId;
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use std::collections::HashSet;

pub mod presentation;
pub mod room_id;

pub trait RoomEventLike: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    fn event_name(&self) -> &'static str;
}

pub trait RoomCommandLike: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    const COMMAND_NAME: &'static str;
    // fn room_id(&self) -> RoomId;
}

pub trait RoomLike: Send + Sync {
    const ROOM_TYPE: &'static str;
    type Command: RoomCommandLike;
    type Event: RoomEventLike;
    fn transaction(&mut self, room_id: RoomId, command: Self::Command, socket: &SocketRef) -> Option<Self::Event>;
    fn add_client(&mut self, socket_id: &str) -> bool;
    fn remove_client(&mut self, socket_id: &str) -> bool;
    fn is_empty(&self) -> bool;
    fn created_at(&self) -> DateTime<Utc>;
    fn id(&self) -> RoomId;
    // fn get_clients(&self) -> &HashSet<String>;
    
    // // Transaction management
    // fn get_transaction_manager(&mut self) -> &mut TransactionManager;
    // fn apply_transaction(&mut self, transaction: Transaction) -> Option<Self::Event>;
    
    // // CRDT operations
    // fn merge_state(&mut self, other: &Self) where Self: Sized;
    // fn get_state_vector(&self) -> Vec<u8>;
    // fn apply_state_delta(&mut self, delta: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
    
    // // Undo/Redo operations
    // fn undo(&mut self) -> Option<ServerEvent> {
    //     if let Some(transaction) = self.get_transaction_manager().undo() {
    //         // Create a reverse transaction and apply it
    //         self.create_reverse_transaction(transaction)
    //             .and_then(|reverse| self.apply_transaction(reverse))
    //     } else {
    //         None
    //     }
    // }
    
    // fn redo(&mut self) -> Option<ServerEvent> {
    //     if let Some(transaction) = self.get_transaction_manager().redo() {
    //         self.apply_transaction(transaction)
    //     } else {
    //         None
    //     }
    // }
    
    // // Helper method to create a reverse transaction for undo
    // fn create_reverse_transaction(&self, transaction: Transaction) -> Option<Transaction>;
}

