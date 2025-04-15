use socketioxide::extract::SocketRef;
use tracing::debug;

use crate::events::{video::{VideoCommand, VideoEvent}, CommandType, EventType};

use super::{room_id::RoomId, RoomLike, RoomMetadata};



pub struct Video {
    pub room_id: RoomId,
    pub socket_id: String,
    pub video_url: String,
}

impl Video {
    pub fn new(room_id: RoomId, socket_id: String, video_url: String) -> Self {
        Self { room_id, socket_id, video_url }
    }
    fn transaction(&mut self, command: VideoCommand, socket: &SocketRef) -> Option<VideoEvent> {
        todo!()
    }
}

impl RoomLike for Video {
    type Command = VideoCommand;
    type Event = VideoEvent;
    fn process_typed_command(&mut self, command: Self::Command, socket: &SocketRef) -> Option<Self::Event> {
        self.transaction(command, socket)
    }
    fn process_command(&mut self, command: CommandType, socket: &SocketRef) -> Option<EventType> {
        match command {
            CommandType::Video(command) => {
                self.process_typed_command(command, socket)
                    .map(EventType::Video)
            }
            _ => {
                debug!("Received non-video command for video room");
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
