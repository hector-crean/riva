use crate::{events::{presentation::{PresentationCommand, PresentationEvent}, ServerEvent}, room::RoomLike};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use serde_json::Value;
use ts_rs::TS;
use std::collections::HashSet;
use tracing::{debug, trace};

use super::room_id::RoomId;

#[derive(Debug, Clone, TS, Deserialize, Serialize)]
#[ts(export)]
pub struct Presentation {
    current_slide: usize,
    slide_data: Vec<Value>,
    clients: HashSet<String>, //socket_ids
    created_at: DateTime<Utc>,
    id: RoomId,
}

impl Presentation {
    pub fn new(
        room_name: String,
        organisation_id: String,
        current_slide: usize,
        slide_data: Vec<Value>,
    ) -> Self {
        Self {
            current_slide,
            slide_data,
            clients: HashSet::new(),
            created_at: Utc::now(),
            id: RoomId::new(&organisation_id, &room_name),
        }
    }
}

impl RoomLike for Presentation {
    const ROOM_TYPE: &'static str = "presentation";
    type Command = PresentationCommand;
    type Event = ServerEvent;
    fn transaction(&mut self, room_id: RoomId, cmd: Self::Command, socket: &SocketRef) -> Option<Self::Event> {
        match cmd {
            PresentationCommand::ChangeSlide { slide_index, .. } => {
                debug!(
                    socket_id = %socket.id,
                    old_slide = self.current_slide,
                    new_slide = slide_index,
                    "Changing slide"
                );
                self.current_slide = slide_index;
                Some(ServerEvent::Presentation(PresentationEvent::SlideChanged {
                    slide_index: self.current_slide,
                }))
            }
            PresentationCommand::JoinPresentation => {
                let socket_id = socket.id.as_str();
                let room_id_str: String = room_id.clone().into();
                
                debug!(
                    socket_id = %socket_id,
                    room_id = %room_id_str,
                    "Client joining presentation"
                );
                
                socket.join(room_id_str);
                self.add_client(socket_id);
                
                Some(ServerEvent::RoomJoined {
                    room_id: room_id.clone(),
                    socket_id: socket_id.to_string(),
                    user_info: None,
                    entered_at: Utc::now(),
                })
            }
            PresentationCommand::LeavePresentation => {
                let socket_id = socket.id.as_str().to_string();
                let room_id_str: String = room_id.clone().into();
                
                debug!(
                    socket_id = %socket_id,
                    room_id = %room_id_str,
                    "Client leaving presentation"
                );
                
                socket.leave(room_id_str);
                self.remove_client(&socket_id);
                
                Some(ServerEvent::RoomLeft { 
                    room_id: room_id.clone(),
                    socket_id: socket_id.to_string(),
                })
            }
        }
    }
    
    fn add_client(&mut self, socket_id: &str) -> bool {
        debug!(
            socket_id = %socket_id,
            clients_count = self.clients.len() + 1,
            "Adding client to presentation"
        );
        self.clients.insert(socket_id.to_string())
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
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn id(&self) -> RoomId {
        self.id.clone()
    }
}