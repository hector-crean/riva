use crate::{RoomLike, TypedRoom};
use serde_json::Value;
use socketioxide::extract::SocketRef;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{debug, trace};


use super::room_id::RoomId;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{RoomCommand, RoomEvent};

// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct ChangeSlide {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationCommand {
    JoinPresentation { room_id: RoomId },
    LeavePresentation { room_id: RoomId },
    ChangeSlide { slide_index: usize },
}

impl RoomCommand for PresentationCommand {
    const COMMAND_NAME: &'static str = "presentation";
    // fn command_name(&self) -> &'static str {
    //     match self {
    //         Self::JoinPresentation { .. } => "join_presentation",
    //         Self::LeavePresentation { .. } => "leave_presentation",
    //         Self::ChangeSlide { .. } => "change_slide",
    //     }
    // }
    fn room_id(&self) -> RoomId {
        match self {
            Self::JoinPresentation { room_id } => room_id.clone(),
            Self::LeavePresentation { room_id } => room_id.clone(),
            Self::ChangeSlide { .. } => todo!(),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct SlideChanged {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationEvent {
    PresentationJoined { socket_id: String, room_id: RoomId },
    PresentationLeft { socket_id: String, room_id: RoomId },
    SlideChanged { slide_index: usize },
}

impl RoomEvent for PresentationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::PresentationJoined { .. } => "presentation_joined",
            Self::PresentationLeft { .. } => "presentation_left",
            Self::SlideChanged { .. } => "slide_changed",
        }
    }
}

// Example implementation for a presentation room
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PresentationRoom {
    pub id: RoomId,
    pub slides: Vec<String>,
    pub current_slide: usize,
    pub connected_users: HashSet<String>,
}

impl TypedRoom for PresentationRoom {
    type Command = PresentationCommand;
    type Event = PresentationEvent;

    fn room_id(&self) -> RoomId {
        self.id.clone()
    }
    fn process_command(
        &mut self,
        command: Self::Command,
        socket: &SocketRef,
    ) -> Option<Self::Event> {
        match command {
            PresentationCommand::JoinPresentation { room_id } => {
                self.connected_users.insert(socket.id.to_string());
                Some(PresentationEvent::PresentationJoined {
                    socket_id: socket.id.to_string(),
                    room_id,
                })
            }
            PresentationCommand::LeavePresentation { room_id } => {
                self.connected_users.remove(&socket.id.to_string());
                Some(PresentationEvent::PresentationLeft {
                    socket_id: socket.id.to_string(),
                    room_id,
                })
            }
            PresentationCommand::ChangeSlide { slide_index } => {
                self.current_slide = slide_index;
                Some(PresentationEvent::SlideChanged { slide_index })
            }
        }
    }
    async fn emit_event(
        &self,
        socket: &SocketRef,
        event: Self::Event,
    ) -> Result<(), String> {
        let room_id: String = self.room_id().into();
        socket
            .within(room_id)
            .emit("message", &event)
            .await
            .map_err(|e| e.to_string())
    }
}
