use super::{RoomError, RoomLike, TransactionOutcome, client_id::ClientId, room_id::RoomId};
use crate::{message::Message, message_broker::MessageBroker};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct RoomManager<B: MessageBroker, R: RoomLike> {
    rooms: HashMap<RoomId, R>,
    msg_broker: Arc<B>,
}

impl<B: MessageBroker, R: RoomLike> RoomManager<B, R>
where
    RoomError: std::convert::From<<B as MessageBroker>::Error>,
{
    pub fn new(msg_broker: B) -> Self {
        Self {
            rooms: HashMap::new(),
            msg_broker: Arc::new(msg_broker),
        }
    }

    fn add_room(&mut self, room_id: RoomId, room: R) {
        self.rooms.insert(room_id, room);
    }

    fn remove_room(&mut self, room_id: RoomId) {
        self.rooms.remove(&room_id);
    }

    async fn handle_client_message(
        &mut self,
        room_id: &RoomId,
        client_id: &ClientId,
        message: Message<R::ClientMessageType>,
    ) -> Result<(), RoomError> {
        let room = self
            .rooms
            .get_mut(room_id)
            .ok_or(RoomError::RoomNotFound(room_id.clone()))?;

        let outcome = room.apply_client_message(client_id, message)?;

        // Handle the outcome using the shared communicator
        match outcome {
            TransactionOutcome::None => {}
            TransactionOutcome::Broadcast {
                message,
                exclude_sender,
            } => {
                let exclude = if exclude_sender {
                    vec![client_id.clone()]
                } else {
                    vec![]
                };
                self.msg_broker
                    .as_ref()
                    .broadcast(room_id.as_str(), "message", &message, &exclude)
                    .await?;
            }
            TransactionOutcome::BroadcastStorageUpdate {
                diff,
                exclude_sender,
            } => {
                let exclude = if exclude_sender {
                    vec![client_id.clone()]
                } else {
                    vec![]
                };
                self.msg_broker
                    .as_ref()
                    .broadcast_all("message", &diff, &exclude)
                    .await?;
            }
            TransactionOutcome::SendTo { clients, message } => {}
            TransactionOutcome::Multiple(msgs) => {} // Handle other outcomes...
        }

        Ok(())
    }
}
