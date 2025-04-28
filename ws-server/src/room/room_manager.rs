use super::{client_id::ClientId, presence::PresenceLike, room_id::RoomId, storage::StorageLike, RoomError, RoomLike, TransactionOutcome};
use crate::{message::{ClientMessageTypeLike, Message, ServerMessageTypeLike}, message_broker::MessageBroker};
use std::{
    collections::HashMap,
    sync::Arc, // Use std::sync::RwLock for the map if lookups dominate adds/removals
    // Alternatively, consider crates like dashmap for concurrent HashMaps
};
use serde::Serialize;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock}; // Use tokio locks if holding across .await

// Option 1: RwLock for map, Tokio Mutex per Room
#[derive(Clone)]
pub struct RoomManager<B: MessageBroker, R: RoomLike> {
    // Outer lock protects the HashMap structure (add/remove/lookup)
    rooms: Arc<TokioRwLock<HashMap<RoomId, Arc<TokioMutex<R>>>>>, // Using Tokio RwLock for async map access
    msg_broker: Arc<B>,
}

impl<B: MessageBroker + Send + Sync + 'static, R: RoomLike + Send + Sync + 'static> RoomManager<B, R>
where
    R::ClientMessageType: Send + Sync,
{
    pub fn new(msg_broker: B) -> Self {
        Self {
            rooms: Arc::new(TokioRwLock::new(HashMap::new())),
            msg_broker: Arc::new(msg_broker),
        }
    }

    pub async fn add_room(&self, room_id: RoomId, room: R) {
        let mut rooms_guard = self.rooms.write().await; // Acquire write lock for the map
        rooms_guard.insert(room_id, Arc::new(TokioMutex::new(room)));
        // map write lock is released when rooms_guard goes out of scope
    }

    pub async fn remove_room(&self, room_id: &RoomId) {
        let mut rooms_guard = self.rooms.write().await; // Acquire write lock for the map
        rooms_guard.remove(room_id);
        // map write lock is released here
    }

    // Method to get a room handle without holding the map lock for long
    async fn get_room_handle(&self, room_id: &RoomId) -> Option<Arc<TokioMutex<R>>> {
        let rooms_guard = self.rooms.read().await; // Acquire read lock for the map
        let room_handle = rooms_guard.get(room_id).cloned();
        // map read lock is released here
        room_handle
    }

    pub async fn get_room(&self, room_id: &RoomId) -> Result<R, RoomError> {
        let room_handle = self.get_room_handle(room_id)
            .await
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        let room = room_handle.lock().await;
        let room_snapshot = room.snapshot();
    
        Ok(room_snapshot)
    }

    /// Updates a room's state using a closure that takes a mutable reference to the room.
    /// This allows for more flexible updates, including partial updates of the room's state.
    /// 
    /// # Example
    /// ```rust
    /// room_manager.update_room_with(&room_id, |room| {
    ///     // Update specific fields of the room
    ///     room.some_field = new_value;
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn update_room_with<F>(&self, room_id: &RoomId, update_fn: F) -> Result<(), RoomError>
    where
        F: FnOnce(&mut R) -> Result<(), RoomError>,
    {
        let room_handle = self.get_room_handle(room_id)
            .await
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        let mut room_guard = room_handle.lock().await;
        update_fn(&mut *room_guard)?;
        
        Ok(())
    }

    /// Updates a room's state with a new room instance.
    /// This is useful for bulk updates or when you need to replace the entire room state.
    pub async fn update_room(&self, room_id: &RoomId, new_room: R) -> Result<(), RoomError> {
        self.update_room_with(room_id, |room| {
            *room = new_room;
            Ok(())
        }).await
    }

    pub async fn handle_client_message(
        &self, // Now takes &self because internal state uses Arc/Locks
        room_id: &RoomId,
        client_id: &ClientId,
        message: Message<R::ClientMessageType>,
    ) -> Result<(), RoomError> {

        // 1. Get the handle to the room's lock, briefly locking the map
        let room_handle = self.get_room_handle(room_id)
            .await
            .ok_or_else(|| RoomError::RoomNotFound(room_id.clone()))?;

        // 2. Lock only the specific room we need to modify
        let mut room_guard = room_handle.lock().await; // Acquire async mutex for the specific room

        // 3. Apply the message to the room's state
        //    We pass &mut *room_guard which is &mut R
        let outcome = room_guard.apply_client_message(client_id, message)?;

        // --- Optional: Release room lock early if possible ---
        // If apply_client_message is quick and doesn't need to be atomic
        // with the broadcast, you *could* potentially drop the lock here:
        // drop(room_guard);
        // However, this risks race conditions if another message comes in
        // before the broadcast completes, depending on your logic.
        // It's often safer to hold the lock during the broadcast
        // if consistency is paramount.
        // ----------------------------------------------------

        // 4. Handle the outcome using the shared communicator
        match outcome {
            TransactionOutcome::None => {}
            TransactionOutcome::Broadcast {
                message,
                exclude_sender,
            } => {
                let exclude = if exclude_sender { vec![client_id.clone()] } else { vec![] };
                // Perform the async broadcast while holding the room lock
                self.msg_broker
                    .as_ref()
                    .broadcast(room_id.as_str(), "message", &message, &exclude)
                    .await.map_err(|e| RoomError::MessageBrokerError(e))?;
            }
            TransactionOutcome::BroadcastStorageUpdate {
                diff,
                exclude_sender,
            } => {
                let exclude = if exclude_sender { vec![client_id.clone()] } else { vec![] };
                 // If this broadcasts to *all* rooms, it might be complex or slow.
                 // Usually, storage updates are per-room or global but announced differently.
                 // Assuming it's for the current room based on the pattern:
                self.msg_broker
                    .as_ref()
                    .broadcast(room_id.as_str(), "storage_update", &diff, &exclude) // Changed topic for clarity
                    .await?;
                // If it truly means ALL rooms, you'd need a different msg_broker method
                // like broadcast_globally or iterate over rooms (potentially complex with locking).
            }
            TransactionOutcome::SendTo { clients, message } => {
                 // This requires resolving ClientIds to actual connections/topics
                 // This logic might live in the MessageBroker or need lookup here
                 for target_client in clients {
                     // Assuming msg_broker has a send_direct method
                     // self.msg_broker.send_direct(target_client.id(), "message", &message).await?;
                 }
            }
            TransactionOutcome::Multiple(msgs) => {
                // Handle multiple outcomes, potentially involving multiple broadcasts/sends
                 for single_outcome in msgs {
                     // Recursively handle or inline logic similar to above
                     // Be mindful of lock duration if this becomes complex
                 }
            }
        }

        // 5. Room lock (room_guard) is released automatically when it goes out of scope here

        Ok(())
    }
}





