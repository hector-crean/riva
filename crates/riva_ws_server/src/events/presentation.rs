use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::room_id::RoomId;



// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct ChangeSlide {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationCommand {
    JoinPresentation {
        room_id: RoomId,
    },
    LeavePresentation {
        room_id: RoomId,
    },
    ChangeSlide { 
        slide_index: usize,
    },
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
