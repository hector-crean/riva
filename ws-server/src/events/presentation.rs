use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::room::message::{ClientMessageLike, ServerMessageLike};




// #[derive(Debug, Clone, Serialize, Deserialize, TS)]
// pub struct ChangeSlide {
//     pub slide_index: usize,
// }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum PresentationClientMessage {
    JoinPresentation,
    LeavePresentation,
    ChangeSlide {
        slide_index: usize,
    },
}


impl ClientMessageLike for PresentationClientMessage {
    fn name(&self) -> &'static str {
        match self {
            Self::JoinPresentation => "JoinPresentation",
            Self::LeavePresentation => "LeavePresentation",
            Self::ChangeSlide { .. } => "ChangeSlide",
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
pub enum PresentationServerMessage {
    SlideChanged {
        slide_index: usize,
    },
}

impl ServerMessageLike for PresentationServerMessage {
    fn name(&self) -> &'static str {
        match self {
            Self::SlideChanged { .. } => "SlideChanged",
        }
    }
}
