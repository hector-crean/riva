use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::{room_id::RoomId, RoomCommand, RoomEvent};



// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct ChangeSlide {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationCommand {
    JoinPresentation,
    LeavePresentation,
    ChangeSlide {
        slide_index: usize,
    },
}


impl RoomCommand for PresentationCommand {
    const COMMAND_NAME: &'static str = "presentation";
    
}

// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct SlideChanged {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationEvent {
    PresentationJoined {
        socket_id: String,
        room_id: RoomId,
    },
    PresentationLeft {
        socket_id: String,
        room_id: RoomId,
    },
    SlideChanged {
        slide_index: usize,
    },
}

impl RoomEvent for PresentationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            PresentationEvent::PresentationJoined { .. } => "PresentationJoined",
            PresentationEvent::PresentationLeft { .. } => "PresentationLeft",
            PresentationEvent::SlideChanged { .. } => "SlideChanged",
        }
    }
}
