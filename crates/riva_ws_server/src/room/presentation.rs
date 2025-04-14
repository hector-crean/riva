use crate::{events::presentation::{PresentationCommand, PresentationEvent}, room::RoomLike};
use socketioxide::extract::SocketRef;
use serde_json::Value;
use std::collections::HashSet;
use tracing::{debug, trace};

#[derive(Debug, Clone)]
pub struct Presentation {
    name: String,
    current_slide: usize,
    slide_data: Vec<Value>,
    clients: HashSet<String>, //socket_ids
}

impl Presentation {
    pub fn new(
        name: String,
        current_slide: usize,
        slide_data: Vec<Value>,
    ) -> Self {
        Self {
            name,
            current_slide,
            slide_data,
            clients: HashSet::new(),
        }
    }
}

impl RoomLike for Presentation {
    type Command = PresentationCommand;
    type Event = PresentationEvent;
    fn transaction(&mut self, cmd: Self::Command, socket: &SocketRef) -> Option<Self::Event> {
        match cmd {
            PresentationCommand::ChangeSlide { slide_index, .. } => {
                debug!(
                    socket_id = %socket.id,
                    old_slide = self.current_slide,
                    new_slide = slide_index,
                    "Changing slide"
                );
                self.current_slide = slide_index;
                Some(PresentationEvent::SlideChanged {
                    slide_index: self.current_slide,
                })
            }
            PresentationCommand::JoinPresentation {
                room_id,
            } => {
                let room_id_str: String = room_id.clone().into();
                debug!(
                    socket_id = %socket.id,
                    room_id = %room_id_str,
                    "Client joining presentation"
                );
                socket.join(room_id_str);
                self.clients.insert(socket.id.as_str().to_string());
                Some(PresentationEvent::PresentationJoined {
                    socket_id: socket.id.as_str().to_string(),
                    room_id: room_id.clone(),
                })
            }
            PresentationCommand::LeavePresentation { room_id } => {
                let room_id_str: String = room_id.clone().into();
                // debug!(
                //     socket_id = %socket.id,
                //     client_id = %client_id,
                //     room_id = %room_id_str,
                //     "Client leaving presentation"
                // );
                socket.leave(room_id_str);
                self.clients.remove(&socket.id.as_str().to_string());
                Some(PresentationEvent::PresentationLeft  { socket_id: socket.id.as_str().to_string(), room_id: room_id.clone()})
            }
        }
    }
    fn add_client(&mut self, client_id: String, socket_id: String) -> bool {
        debug!(
            client_id = %client_id,
            socket_id = %socket_id,
            clients_count = self.clients.len() + 1,
            "Adding client to presentation"
        );
        self.clients.insert(socket_id);
        true
    }
    fn remove_client(&mut self, client_id: &str) -> bool {
        debug!(
            client_id = %client_id,
            clients_count = self.clients.len() - 1,
            "Removing client from presentation"
        );
        self.clients.remove(client_id);
        true
    }
    fn is_empty(&self) -> bool {
        trace!(clients_count = self.clients.len(), "Checking if presentation is empty");
        self.clients.is_empty()
    }
}