use chrono::{DateTime, Utc};
use room_id::RoomId;
use serde::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;

pub mod presentation;
pub mod room_id;


pub trait RoomEvent: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    fn event_name(&self) -> &'static str;
}

pub trait RoomCommand: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    const COMMAND_NAME: &'static str;
    // fn room_id(&self) -> RoomId;
}


pub trait RoomLike: Send + Sync {
    const ROOM_TYPE: &'static str;
    type Command: RoomCommand;
    type Event: RoomEvent;
    fn transaction(&mut self, room_id: RoomId, command: Self::Command, socket: &SocketRef) -> Option<Self::Event>;
    fn add_client(&mut self, socket_id: &str) -> bool;
    fn remove_client(&mut self, socket_id: &str) -> bool;
    fn is_empty(&self) -> bool;
    fn created_at(&self) -> DateTime<Utc>;
    fn id(&self) -> RoomId;
}

