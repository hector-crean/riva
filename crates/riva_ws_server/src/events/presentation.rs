use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::{room_id::RoomId, RoomCommandLike, RoomEventLike};



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


impl RoomCommandLike for PresentationCommand {
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
    SlideChanged {
        slide_index: usize,
    },
}

impl RoomEventLike for PresentationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            PresentationEvent::SlideChanged { .. } => "SlideChanged",
        }
    }
}
