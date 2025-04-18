use std::{collections::HashMap, sync::Arc};

use super::{network_communicator::NetworkCommunicator, room_id::RoomId, ClientId, Room, RoomError, RoomLike, TransactionOutcome};






struct RoomManager<N: NetworkCommunicator, R: RoomLike> {
    rooms: HashMap<RoomId, R>,
    communicator: Arc<N>,
}




impl<N: NetworkCommunicator, R: RoomLike> RoomManager<N, R> {
    fn handle_client_message(&mut self, room_id: &RoomId, client_id: &ClientId, message: R::ClientMessage) -> Result<(), RoomError> {
        let room = self.rooms.get_mut(room_id).ok_or(RoomError::RoomNotFound(room_id.clone()))?;
        
        let outcome = room.apply_client_message(client_id, message)?;
        
        // Handle the outcome using the shared communicator
        match outcome {
            TransactionOutcome::None => {},
            TransactionOutcome::Broadcast { message, exclude_sender } => {
                let exclude = if exclude_sender { vec![client_id.clone()] } else { vec![] };
                room.broadcast(&message, &exclude, &self.communicator)?;
            },
            TransactionOutcome::BroadcastStorageUpdate { diff, exclude_sender } => {
                let exclude = if exclude_sender { vec![client_id.clone()] } else { vec![] };
                room.broadcast_storage_update(&diff, &exclude, &self.communicator)?;
            },
            TransactionOutcome::SendTo { clients, message } => {},
            TransactionOutcome::Multiple(msgs) => {}
            // Handle other outcomes...
        }
        
        Ok(())
    }
}