use socketioxide::extract::SocketRef;
use tracing::debug;


use super::{room_id::RoomId};


use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::{RoomCommand, RoomEvent, TypedRoom};




#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum VideoCommand {
   StartVideo {
    room_id: RoomId,
   }
}

impl RoomCommand for VideoCommand {
    const COMMAND_NAME: &'static str = "video";
    fn room_id(&self) -> RoomId {
        match self {
            Self::StartVideo { room_id } => room_id.clone(),
        }
    }

   
}




#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum VideoEvent {
    VideoStarted
}

impl RoomEvent for VideoEvent {
    fn event_name(&self) -> &'static str {
        "video"
    }
}




pub struct Video {
    pub room_id: RoomId,
    pub socket_id: String,
    pub video_url: String,
}

impl Video {
    pub fn new(room_id: RoomId, socket_id: String, video_url: String) -> Self {
        Self { room_id, socket_id, video_url }
    }
   
}
impl TypedRoom for Video {
    type Command = VideoCommand;
    type Event = VideoEvent;
    fn process_command(&mut self, command: Self::Command, socket: &SocketRef) -> Option<Self::Event> {
        todo!()
    }
    fn room_id(&self) -> RoomId {
        self.room_id.clone()
    }
    async fn emit_event(&self, socket: &SocketRef, event: Self::Event) -> Result<(), String> {
        todo!()
    }
}

