use crate::{events::presentation::{PresentationCommand, PresentationEvent}, room::RoomLike};
use socketioxide::extract::SocketRef;
use serde_json::Value;
use std::collections::HashSet;
use tracing::{debug, trace};
use crate::events::{CommandType, EventType};
use std::collections::HashMap;

use super::RoomMetadata;


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
    fn transaction(&mut self, command: PresentationCommand, socket: &SocketRef) -> Option<PresentationEvent> {
        match command {
            PresentationCommand::NextSlide => {
                if self.current_slide < self.slide_data.len() - 1 {
                    self.current_slide += 1;
                    Some(PresentationEvent::SlideChanged {
                        slide_index: self.current_slide,
                        initiated_by: socket.id.to_string(),
                    })
                } else {
                    None
                }
            },
            PresentationCommand::PreviousSlide => {
                if self.current_slide > 0 {
                    self.current_slide -= 1;
                    Some(PresentationEvent::SlideChanged {
                        slide_index: self.current_slide,
                        initiated_by: socket.id.to_string(),
                    })
                } else {
                    None
                }
            },
            PresentationCommand::JumpToSlide(index) => {
                if index < self.slide_data.len() {
                    self.current_slide = index;
                    Some(PresentationEvent::SlideChanged {
                        slide_index: self.current_slide,
                        initiated_by: socket.id.to_string(),
                    })
                } else {
                    debug!("Attempted to jump to invalid slide index: {}", index);
                    None
                }
            },
            // Add other command handlers as needed
            _ => {
                debug!("Unhandled presentation command: {:?}", command);
                None
            }
        }
    }
}

impl RoomLike for Presentation {
    type Command = PresentationCommand;
    type Event = PresentationEvent;
    fn process_typed_command(&mut self, command: Self::Command, socket: &SocketRef) -> Option<Self::Event> {
        self.transaction(command, socket)
    }
    fn process_command(&mut self, command: CommandType, socket: &SocketRef) -> Option<EventType> {
        match command {
            CommandType::Presentation(cmd) => {
                trace!("Processing presentation command: {:?}", cmd);
                self.process_typed_command(cmd, socket)
                    .map(EventType::Presentation)
            },
            _ => {
                debug!("Received non-presentation command for presentation room");
                None
            }
        }
    }
    fn join_user(&mut self, socket_id: String) -> Option<EventType> {
       todo!()
    }
    fn leave_user(&mut self, socket_id: String) -> Option<EventType> {
       todo!()
    }
    fn get_metadata(&self) -> RoomMetadata {
       todo!()
    }
    fn room_type(&self) -> &'static str {
        todo!()
    }
    fn get_users(&self) -> Vec<String> {
        todo!()
    }
}

